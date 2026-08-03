//! Inyección transitoria de encabezado y pie de página para la previsualización
//! y conversión a PDF.
//!
//! Cuando el usuario elige "Reemplazar", el contenido del encabezado y del pie
//! NO está hardcodeado en el código: se lee de dos archivos independientes y
//! editables, creados automáticamente la primera vez que se usa:
//!
//! - `{DATA_DIR}/header_footer/header.xml`  → contenido interno de `<w:hdr>`
//! - `{DATA_DIR}/header_footer/footer.xml`  → contenido interno de `<w:ftr>`
//!
//! Esos archivos pueden contener los marcadores dinámicos:
//! - `{{titulo_documento}}` → nombre del documento
//! - `{{link_servidor}}`    → URL pública del backend
//! - `{{qr_documento}}`     → imagen QR con el id del documento (solo Word)
//!
//! Las etiquetas `{{key}}` del sistema (p. ej. `{{fecha_actual}}`) también se
//! resuelven dentro del encabezado/pie porque `tag_service` procesa las partes
//! `word/header*.xml` y `word/footer*.xml` después de esta inyección.

use std::collections::HashSet;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::str::FromStr;

use serde::Deserialize;
use regex::Regex;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

use crate::helpers::config;
use crate::models::Document;

/// Modo de tratamiento del encabezado/pie de página al previsualizar o convertir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderFooterMode {
    /// No se inyecta nada: se respeta el encabezado/pie que traiga el archivo.
    Preserve,
    /// Se inyecta el encabezado/pie definido en los archivos editables.
    Replace,
}

impl Default for HeaderFooterMode {
    fn default() -> Self {
        HeaderFooterMode::Preserve
    }
}

impl FromStr for HeaderFooterMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "replace" => Ok(HeaderFooterMode::Replace),
            // Tolerante: cualquier valor desconocido se comporta como `preserve`.
            _ => Ok(HeaderFooterMode::Preserve),
        }
    }
}

/// Query param `?header_footer=preserve|replace` (opcional, default `preserve`).
#[derive(Deserialize, Default)]
pub struct HeaderFooterQuery {
    pub header_footer: Option<String>,
}

impl HeaderFooterQuery {
    pub fn mode(&self) -> HeaderFooterMode {
        self.header_footer
            .as_deref()
            .and_then(|s| HeaderFooterMode::from_str(s).ok())
            .unwrap_or_default()
    }
}

const HDR_OPEN: &str = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">"#;
const HDR_CLOSE: &str = "</w:hdr>";
const FTR_OPEN: &str = r#"<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#;
const FTR_CLOSE: &str = "</w:ftr>";

/// Contenido por defecto del encabezado (creado la primera vez).
const DEFAULT_HEADER: &str = r#"<w:p>
  <w:pPr><w:jc w:val="right"/></w:pPr>
  <w:r><w:rPr><w:b/><w:sz w:val="20"/></w:rPr><w:t xml:space="preserve">{{titulo_documento}}</w:t></w:r>
  <w:r><w:rPr><w:sz w:val="20"/></w:rPr><w:t xml:space="preserve">  </w:t></w:r>
  {{qr_documento}}
</w:p>"#;

/// Contenido por defecto del pie de página (creado la primera vez).
const DEFAULT_FOOTER: &str = r#"<w:p>
  <w:pPr><w:jc w:val="center"/></w:pPr>
  <w:r><w:rPr><w:sz w:val="18"/></w:rPr><w:t xml:space="preserve">{{link_servidor}} · {{fecha_actual}} · Página </w:t></w:r>
  <w:fldSimple w:instr=" PAGE "><w:r><w:rPr><w:sz w:val="18"/></w:rPr><w:t>1</w:t></w:r></w:fldSimple>
  <w:r><w:rPr><w:sz w:val="18"/></w:rPr><w:t xml:space="preserve"> de </w:t></w:r>
  <w:fldSimple w:instr=" NUMPAGES "><w:r><w:rPr><w:sz w:val="18"/></w:rPr><w:t>1</w:t></w:r></w:fldSimple>
</w:p>"#;

/// Inyecta el encabezado/pie definido por el usuario según el modo.
///
/// `Preserve` → `None` (no se modifica nada). `Replace` → reescribe el ZIP del
/// `.docx`/`.xlsx` y devuelve los bytes modificados; si el archivo no es
/// OOXML editable o algo falla, devuelve `None` para no romper el flujo.
pub fn inject(
    content: &[u8],
    doc: &Document,
    mode: HeaderFooterMode,
    data_dir: &Path,
) -> Option<Vec<u8>> {
    match mode {
        HeaderFooterMode::Preserve => return None,
        HeaderFooterMode::Replace => {}
    }
    match doc.ext.to_ascii_lowercase().as_str() {
        "docx" => inject_docx(content, doc, data_dir),
        "xlsx" => inject_xlsx(content, doc),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// DOCX (Word)
// ---------------------------------------------------------------------------

fn inject_docx(content: &[u8], doc: &Document, data_dir: &Path) -> Option<Vec<u8>> {
    let (header_inner, footer_inner) = ensure_part_files(data_dir)?;
    let qr_bytes = qr_png(&doc.id)?;

    let reader = Cursor::new(content.to_vec());
    let mut archive = zip::ZipArchive::new(reader).ok()?;
    let mut entries: Vec<(String, CompressionMethod, Vec<u8>)> = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).ok()?;
        let name = file.name().to_string();
        let method = file.compression();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).ok()?;
        entries.push((name, method, bytes));
    }

    if !entries.iter().any(|(n, _, _)| n == "word/document.xml") {
        return None;
    }

    let existing: HashSet<String> = entries.iter().map(|(n, _, _)| n.clone()).collect();
    let header_part = find_free_part(&existing, "word/header", ".xml");
    let footer_part = find_free_part(&existing, "word/footer", ".xml");
    let media_part = find_free_part(&existing, "word/media/header_qr", ".png");
    let header_rels = format!(
        "word/_rels/{}.rels",
        header_part.strip_prefix("word/").unwrap_or(&header_part)
    );

    // rIds únicos para las relaciones header/footer del documento.
    let old_rels = entries
        .iter()
        .find(|(n, _, _)| n == "word/_rels/document.xml.rels")
        .and_then(|(_, _, b)| String::from_utf8(b.clone()).ok());
    let (hdr_rid, ftr_rid) = old_rels
        .as_deref()
        .map(find_free_r_ids)
        .unwrap_or_else(|| ("rId1".to_string(), "rId2".to_string()));

    // Contenido de las partes.
    let title = xml_escape(&doc.name);
    let link = xml_escape(&config::public_backend_url(8091));
    let qr_drawing = qr_drawing_xml();
    let header_content = header_inner
        .replace("{{titulo_documento}}", &title)
        .replace("{{link_servidor}}", &link)
        .replace("{{qr_documento}}", &qr_drawing);
    let footer_content = footer_inner
        .replace("{{titulo_documento}}", &title)
        .replace("{{link_servidor}}", &link)
        .replace("{{qr_documento}}", "");
    let header_xml = format!("{HDR_OPEN}{header_content}{HDR_CLOSE}");
    let footer_xml = format!("{FTR_OPEN}{footer_content}{FTR_CLOSE}");

    // document.xml: referencias en el sectPr final + namespace r.
    let doc_xml = entries
        .iter()
        .find(|(n, _, _)| n == "word/document.xml")
        .and_then(|(_, _, b)| String::from_utf8(b.clone()).ok())?;
    let new_doc_xml = inject_sectpr(&ensure_r_ns(&doc_xml), &hdr_rid, &ftr_rid);

    // document.xml.rels: relaciones header/footer.
    let rels_add = format!(
        "<Relationship Id=\"{h}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/header\" Target=\"{hp}\"/><Relationship Id=\"{f}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer\" Target=\"{fp}\"/>",
        h = hdr_rid,
        f = ftr_rid,
        hp = header_part.strip_prefix("word/").unwrap_or(&header_part),
        fp = footer_part.strip_prefix("word/").unwrap_or(&footer_part)
    );
    let new_rels = match &old_rels {
        Some(rels) => rels.replacen("</Relationships>", &format!("{rels_add}</Relationships>"), 1),
        None => format!(
            "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">{rels_add}</Relationships>"
        ),
    };

    // [Content_Types].xml: overrides + png.
    let types_add = format!(
        "<Override PartName=\"/{hp}\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml\"/><Override PartName=\"/{fp}\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml\"/>",
        hp = header_part,
        fp = footer_part
    );
    let png_default = r#"<Default Extension="png" ContentType="image/png"/>"#;
    let old_types = entries
        .iter()
        .find(|(n, _, _)| n == "[Content_Types].xml")
        .and_then(|(_, _, b)| String::from_utf8(b.clone()).ok());
    let new_types = match &old_types {
        Some(t) => {
            let base = t.replacen("</Types>", &format!("{types_add}</Types>"), 1);
            if t.contains("Extension=\"png\"") {
                base
            } else {
                base.replacen("</Types>", &format!("{png_default}</Types>"), 1)
            }
        }
        None => format!(
            "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">{types_add}{png_default}</Types>"
        ),
    };

    // Relaciones de la imagen dentro de la parte header.
    let header_rels_xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n  <Relationship Id=\"rIdImg\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"{media}\"/>\n</Relationships>",
        media = media_part.strip_prefix("word/").unwrap_or(&media_part)
    );

    let mut out: Vec<(String, CompressionMethod, Vec<u8>)> = Vec::new();
    for (name, method, bytes) in entries {
        let bytes = if name == "word/document.xml" {
            new_doc_xml.clone().into_bytes()
        } else if name == "word/_rels/document.xml.rels" {
            new_rels.clone().into_bytes()
        } else if name == "[Content_Types].xml" {
            new_types.clone().into_bytes()
        } else {
            bytes
        };
        out.push((name, method, bytes));
    }
    out.push((header_part, CompressionMethod::Deflated, header_xml.into_bytes()));
    out.push((footer_part, CompressionMethod::Deflated, footer_xml.into_bytes()));
    out.push((header_rels, CompressionMethod::Deflated, header_rels_xml.into_bytes()));
    out.push((media_part, CompressionMethod::Deflated, qr_bytes));

    rewrite_zip(out)
}

/// Añade los `w:headerReference`/`w:footerReference` al sectPr final (el último
/// del documento, que controla la última sección y el caso más común de una
/// sola sección), eliminando antes las referencias existentes.
fn inject_sectpr(document_xml: &str, hdr_rid: &str, ftr_rid: &str) -> String {
    let re = Regex::new(r"(?s)<w:sectPr\b[^>]*/>|<w:sectPr\b.*?</w:sectPr>").unwrap();
    let mut last: Option<(usize, usize)> = None;
    for caps in re.captures_iter(document_xml) {
        let m = caps.get(0).unwrap();
        last = Some((m.start(), m.end()));
    }
    let (start, end) = match last {
        Some(v) => v,
        None => return document_xml.to_string(),
    };
    let sectpr = &document_xml[start..end];

    let refs = format!(
        "<w:headerReference w:type=\"default\" r:id=\"{h}\"/><w:headerReference w:type=\"first\" r:id=\"{h}\"/><w:headerReference w:type=\"even\" r:id=\"{h}\"/><w:footerReference w:type=\"default\" r:id=\"{f}\"/><w:footerReference w:type=\"first\" r:id=\"{f}\"/><w:footerReference w:type=\"even\" r:id=\"{f}\"/>",
        h = hdr_rid,
        f = ftr_rid
    );

    let new_sectpr = if sectpr.ends_with("/>") {
        format!("<w:sectPr>{refs}</w:sectPr>")
    } else {
        let re_ref = Regex::new(
            r"<w:(?:headerReference|footerReference)\b[^>]*/>|<w:(?:headerReference|footerReference)\b.*?</w:(?:headerReference|footerReference)>",
        )
        .unwrap();
        let cleaned = re_ref.replace_all(sectpr, "");
        cleaned.replacen("</w:sectPr>", &format!("{refs}</w:sectPr>"), 1)
    };

    let mut out = String::with_capacity(document_xml.len() + 128);
    out.push_str(&document_xml[..start]);
    out.push_str(&new_sectpr);
    out.push_str(&document_xml[end..]);
    out
}

/// Garantiza que `word/document.xml` declare el namespace `xmlns:r` (necesario
/// para las referencias `r:id` en el sectPr).
fn ensure_r_ns(document_xml: &str) -> String {
    let re = Regex::new(r"<w:document\b[^>]*>").unwrap();
    let m = match re.find(document_xml) {
        Some(m) => m,
        None => return document_xml.to_string(),
    };
    let open = m.as_str();
    if open.contains("xmlns:r") {
        return document_xml.to_string();
    }
    let mut out = document_xml.to_string();
    let idx = m.start() + open.len() - 1;
    out.insert_str(
        idx,
        " xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"",
    );
    out
}

// ---------------------------------------------------------------------------
// XLSX (Excel)
// ---------------------------------------------------------------------------

fn inject_xlsx(content: &[u8], doc: &Document) -> Option<Vec<u8>> {
    let reader = Cursor::new(content.to_vec());
    let mut archive = zip::ZipArchive::new(reader).ok()?;
    let mut entries: Vec<(String, CompressionMethod, Vec<u8>)> = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).ok()?;
        let name = file.name().to_string();
        let method = file.compression();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).ok()?;
        entries.push((name, method, bytes));
    }

    let mut changed = false;
    for (name, _, bytes) in entries.iter_mut() {
        if !name.starts_with("xl/worksheets/") || !name.ends_with(".xml") {
            continue;
        }
        let xml = match String::from_utf8(bytes.clone()) {
            Ok(x) => x,
            Err(_) => continue,
        };
        if xml.contains("<headerFooter") {
            continue;
        }
        // Excel no admite imágenes en encabezados de impresión: solo texto.
        let hf = format!(
            "<headerFooter><oddHeader>&amp;L{title}&amp;R{link}</oddHeader><oddFooter>&amp;L{link}&amp;C{date}&amp;RPágina &amp;P de &amp;N</oddFooter></headerFooter>",
            title = xml_escape(&doc.name),
            link = xml_escape(&config::public_backend_url(8091)),
            date = chrono::Local::now().format("%d/%m/%Y")
        );
        let new_xml = insert_header_footer(&xml, &hf);
        if new_xml != xml {
            *bytes = new_xml.into_bytes();
            changed = true;
        }
    }

    if !changed {
        return None;
    }
    rewrite_zip(entries)
}

/// Inserta `<headerFooter>` en la posición correcta del esquema: después de
/// `</pageSetup>` si existe, si no tras `</pageMargins>`, si no tras
/// `</sheetData>` y en último caso antes de `</worksheet>`.
fn insert_header_footer(xml: &str, hf: &str) -> String {
    for marker in ["</pageSetup>", "</pageMargins>", "</sheetData>", "</worksheet>"] {
        if let Some(pos) = xml.rfind(marker) {
            let mut s = xml.to_string();
            s.insert_str(pos + marker.len(), hf);
            return s;
        }
    }
    xml.to_string()
}

// ---------------------------------------------------------------------------
// Archivos editables
// ---------------------------------------------------------------------------

/// Garantiza que existan `header.xml` y `footer.xml` editables y devuelve su
/// contenido (parte interna de `<w:hdr>` / `<w:ftr>`).
fn ensure_part_files(data_dir: &Path) -> Option<(String, String)> {
    let dir = data_dir.join("header_footer");
    std::fs::create_dir_all(&dir).ok()?;
    let hdr_path = dir.join("header.xml");
    let ftr_path = dir.join("footer.xml");
    if !hdr_path.exists() {
        std::fs::write(&hdr_path, DEFAULT_HEADER).ok()?;
        tracing::info!(
            "Creado {} (editable: contenido del encabezado a inyectar)",
            hdr_path.display()
        );
    }
    if !ftr_path.exists() {
        std::fs::write(&ftr_path, DEFAULT_FOOTER).ok()?;
        tracing::info!(
            "Creado {} (editable: contenido del pie de página a inyectar)",
            ftr_path.display()
        );
    }
    let header = std::fs::read_to_string(&hdr_path).ok()?;
    let footer = std::fs::read_to_string(&ftr_path).ok()?;
    Some((header, footer))
}

// ---------------------------------------------------------------------------
// QR
// ---------------------------------------------------------------------------

fn qr_png(data: &str) -> Option<Vec<u8>> {
    let code = qrcode::QrCode::new(data).ok()?;
    let image = code
        .render::<image::Rgba<u8>>()
        .min_dimensions(320, 320)
        .dark_color(image::Rgba([0, 0, 0, 255]))
        .light_color(image::Rgba([255, 255, 255, 255]))
        .build();
    let mut buf = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
        .ok()?;
    Some(buf)
}

/// XML del `<w:drawing>` que muestra la imagen QR dentro del encabezado.
fn qr_drawing_xml() -> String {
    r#"<w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="1050000" cy="1050000"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:docPr id="1" name="qr"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="0" name="qr.png"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rIdImg"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="1050000" cy="1050000"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"#.to_string()
}

// ---------------------------------------------------------------------------
// Utilidades
// ---------------------------------------------------------------------------

/// Primer nombre de parte `word/...{n}...` que no exista aún en el ZIP.
fn find_free_part(existing: &HashSet<String>, prefix: &str, suffix: &str) -> String {
    let mut n = 1;
    loop {
        let name = format!("{prefix}{n}{suffix}");
        if !existing.contains(&name) {
            return name;
        }
        n += 1;
    }
}

/// Dos `Id="rIdN"` libres, por encima del máximo existente en las relaciones.
fn find_free_r_ids(rels_xml: &str) -> (String, String) {
    let re = Regex::new(r#"Id="rId(\d+)""#).unwrap();
    let mut max = 0u32;
    for caps in re.captures_iter(rels_xml) {
        if let Ok(v) = caps[1].parse::<u32>() {
            if v > max {
                max = v;
            }
        }
    }
    (format!("rId{}", max + 1), format!("rId{}", max + 2))
}

fn rewrite_zip(entries: Vec<(String, CompressionMethod, Vec<u8>)>) -> Option<Vec<u8>> {
    let writer = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(writer);
    for (name, method, bytes) in entries {
        let options = SimpleFileOptions::default().compression_method(method);
        zip.start_file(name, options).ok()?;
        zip.write_all(&bytes).ok()?;
    }
    zip.finish().ok().map(|w| w.into_inner())
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::User;

    fn doc() -> Document {
        Document {
            id: "doc-123".to_string(),
            name: "Informe Final".to_string(),
            ext: "docx".to_string(),
            mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            editor: "onlyoffice".to_string(),
            size: 0,
            status: "draft".to_string(),
            owner_id: "u1".to_string(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn tmp_data_dir() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("hf_test_{}", uuid::Uuid::new_v4()))
    }

    fn build_docx(sectpr: &str, rels: &str) -> Vec<u8> {
        use std::io::Write as _;
        let cursor = Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(cursor);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zip.start_file("[Content_Types].xml", opts).unwrap();
        zip.write_all(b"<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/></Types>").unwrap();
        zip.start_file("_rels/.rels", opts).unwrap();
        zip.write_all(b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"/>").unwrap();
        zip.start_file("word/_rels/document.xml.rels", opts).unwrap();
        zip.write_all(rels.as_bytes()).unwrap();
        zip.start_file("word/document.xml", opts).unwrap();
        zip.write_all(
            format!(
                r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:p><w:r><w:t>Contenido</w:t></w:r></w:p>{sectpr}</w:body></w:document>"#
            )
            .as_bytes(),
        )
        .unwrap();
        zip.finish().unwrap().into_inner()
    }

    fn zip_names(bytes: &[u8]) -> Vec<String> {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).unwrap();
        (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect()
    }

    fn zip_read(bytes: &[u8], name: &str) -> String {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).unwrap();
        let mut s = String::new();
        archive.by_name(name).unwrap().read_to_string(&mut s).unwrap();
        s
    }

    #[test]
    fn preserve_returns_none() {
        let dir = tmp_data_dir();
        let docx = build_docx("<w:sectPr/>", r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="t" Target="word/document.xml"/></Relationships>"#);
        assert!(inject(&docx, &doc(), HeaderFooterMode::Preserve, &dir).is_none());
    }

    #[test]
    fn replace_injects_header_footer_parts() {
        let dir = tmp_data_dir();
        let docx = build_docx("<w:sectPr/>", r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="t" Target="word/document.xml"/></Relationships>"#);
        let out = inject(&docx, &doc(), HeaderFooterMode::Replace, &dir).expect("debe inyectar");

        assert!(dir.join("header_footer/header.xml").exists());
        assert!(dir.join("header_footer/footer.xml").exists());

        let names = zip_names(&out);
        assert!(names.iter().any(|n| n.starts_with("word/header")), "falta header part: {names:?}");
        assert!(names.iter().any(|n| n.starts_with("word/footer")), "falta footer part");
        assert!(names.iter().any(|n| n.starts_with("word/media/header_qr")), "falta QR media");

        let doc_xml = zip_read(&out, "word/document.xml");
        assert!(doc_xml.contains("headerReference"));
        assert!(doc_xml.contains("footerReference"));

        let types = zip_read(&out, "[Content_Types].xml");
        assert!(types.contains("header+xml"));
        assert!(types.contains("footer+xml"));
        assert!(types.contains("Extension=\"png\""));
    }

    #[test]
    fn replace_replaces_existing_refs() {
        let dir = tmp_data_dir();
        let docx = build_docx(
            r#"<w:sectPr><w:headerReference w:type="default" r:id="rId9"/><w:footerReference w:type="default" r:id="rId10"/></w:sectPr>"#,
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="t" Target="word/document.xml"/></Relationships>"#,
        );
        let out = inject(&docx, &doc(), HeaderFooterMode::Replace, &dir).unwrap();
        let doc_xml = zip_read(&out, "word/document.xml");
        assert!(!doc_xml.contains("rId9"), "referencia previa no eliminada");
        assert!(!doc_xml.contains("rId10"));
        assert!(doc_xml.contains("headerReference"));
    }

    #[test]
    fn tags_inside_footer_resolve_after_inject() {
        let dir = tmp_data_dir();
        std::fs::create_dir_all(dir.join("header_footer")).unwrap();
        std::fs::write(
            dir.join("header_footer/footer.xml"),
            "<w:p><w:r><w:t>{{fecha_actual}} / {{desconocida}}</w:t></w:r></w:p>",
        )
        .unwrap();
        let docx = build_docx("<w:sectPr/>", r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="t" Target="word/document.xml"/></Relationships>"#);
        let out = inject(&docx, &doc(), HeaderFooterMode::Replace, &dir).unwrap();

        let user = User {
            id: "u1".to_string(),
            username: "u1".to_string(),
            name: "User 1".to_string(),
            dni: None,
            cargo: None,
            email: None,
        };
        let resolved = crate::services::tag_service::resolve(&out, &user, "docx").unwrap_or(out.clone());
        let names = zip_names(&resolved);
        let ftr_name = names
            .iter()
            .find(|n| n.starts_with("word/footer"))
            .cloned()
            .unwrap();
        let ftr = zip_read(&resolved, &ftr_name);
        assert!(!ftr.contains("{{fecha_actual}}"), "fecha_actual sin resolver: {ftr}");
        assert!(ftr.contains("/20"), "fecha no resuelta: {ftr}");
    }

    #[test]
    fn xlsx_gets_header_footer() {
        let dir = tmp_data_dir();
        let xlsx = crate::templates::generate_blank_xlsx();
        let mut d = doc();
        d.ext = "xlsx".to_string();
        let out = inject(&xlsx, &d, HeaderFooterMode::Replace, &dir).expect("debe inyectar xlsx");
        let ws = zip_read(&out, "xl/worksheets/sheet1.xml");
        assert!(ws.contains("<headerFooter"));
        assert!(ws.contains("&amp;P"));
        assert!(ws.contains("&amp;N"));
    }

    #[test]
    fn non_ooxml_returns_none() {
        let dir = tmp_data_dir();
        assert!(inject(b"not a zip", &doc(), HeaderFooterMode::Replace, &dir).is_none());
    }
}
