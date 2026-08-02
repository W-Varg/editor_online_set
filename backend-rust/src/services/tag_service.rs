//! Etiquetas dinámicas (`{{key}}`) dentro de documentos Word/Excel.
//!
//! El plugin de ONLYOFFICE inserta etiquetas como texto literal (p. ej.
//! `{{fecha_actual}}`) y estas quedan guardadas sin resolver en `data/{id}.bin`.
//! Este servicio, al previsualizar o convertir a PDF, reemplaza esas etiquetas
//! por los valores correspondientes del usuario que genera la previsualización
//! (nombre, cargo, DNI, email) y de la fecha actual del servidor.

use std::io::{Cursor, Read, Write};

use regex::Regex;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

use crate::dto::TagDefinition;
use crate::models::User;

/// Definición de una etiqueta disponible para insertar en el editor.
pub struct TagDef {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

/// Catálogo de etiquetas. La API `GET /api/tags` lo expone tal cual; el plugin
/// lo usa para listar las opciones al usuario.
pub const TAG_DEFINITIONS: &[TagDef] = &[
    TagDef {
        key: "fecha_actual",
        label: "Fecha actual",
        description: "Fecha del día en que se previsualiza (dd/mm/aaaa HH:mm).",
    },
    TagDef {
        key: "nombre_usuario",
        label: "Nombre del usuario",
        description: "Nombre del usuario que previsualiza el documento.",
    },
    TagDef {
        key: "cargo_usuario",
        label: "Cargo del usuario",
        description: "Cargo del usuario que previsualiza el documento.",
    },
    TagDef {
        key: "dni",
        label: "DNI del usuario",
        description: "DNI del usuario que previsualiza el documento.",
    },
    TagDef {
        key: "email",
        label: "Email del usuario",
        description: "Email del usuario que previsualiza el documento.",
    },
];

/// Devuelve el listado público de etiquetas para la API.
pub fn list() -> Vec<TagDefinition> {
    TAG_DEFINITIONS
        .iter()
        .map(|t| TagDefinition {
            key: t.key.to_string(),
            label: t.label.to_string(),
            description: t.description.to_string(),
        })
        .collect()
}

/// Calcula el valor de una etiqueta para el usuario indicado.
pub fn value_for(key: &str, user: &User) -> Option<String> {
    match key {
        "fecha_actual" => Some(chrono::Local::now().format("%d/%m/%Y %H:%M").to_string()),
        "nombre_usuario" => Some(user.name.clone()),
        "cargo_usuario" => Some(user.cargo.clone().unwrap_or_default()),
        "dni" => Some(user.dni.clone().unwrap_or_default()),
        "email" => Some(user.email.clone().unwrap_or_default()),
        _ => None,
    }
}

/// Reemplaza las etiquetas `{{key}}` del contenido binario de un documento
/// Word/Excel por los valores del usuario.
///
/// Devuelve `None` si el documento no contiene etiquetas (o no es un archivo
/// OOXML editable por este proyecto), para no alterar el flujo normal.
pub fn resolve(content: &[u8], user: &User, ext: &str) -> Option<Vec<u8>> {
    let ext = ext.to_ascii_lowercase();
    if !matches!(ext.as_str(), "docx" | "doc" | "xlsx" | "xls") {
        return None;
    }

    let reader = Cursor::new(content.to_vec());
    let mut archive = match zip::ZipArchive::new(reader) {
        Ok(a) => a,
        Err(_) => return None,
    };

    let mut entries: Vec<(String, CompressionMethod, Vec<u8>)> = Vec::new();
    let mut changed = false;

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let name = file.name().to_string();
        let method = file.compression();
        let mut bytes = Vec::new();
        if file.read_to_end(&mut bytes).is_err() {
            continue;
        }

        if is_text_part(&ext, &name) {
            if let Ok(text) = String::from_utf8(bytes.clone()) {
                let resolved = resolve_xml_part(&ext, &text, user);
                if resolved != text {
                    changed = true;
                    bytes = resolved.into_bytes();
                }
            }
        }
        entries.push((name, method, bytes));
    }

    if !changed {
        return None;
    }

    let writer = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(writer);
    for (name, method, bytes) in entries {
        let options = SimpleFileOptions::default().compression_method(method);
        if zip.start_file(name, options).is_err() {
            return None;
        }
        if zip.write_all(&bytes).is_err() {
            return None;
        }
    }
    zip.finish().ok().map(|w| w.into_inner())
}

/// Decide si una parte del ZIP contiene texto de usuario reemplazable.
fn is_text_part(ext: &str, name: &str) -> bool {
    if ext.starts_with("xls") {
        name == "xl/sharedStrings.xml"
            || (name.starts_with("xl/worksheets/") && name.ends_with(".xml"))
    } else {
        (name.starts_with("word/") && name.ends_with(".xml"))
            || name == "word/document.xml"
    }
}

/// Procesa una parte XML (document.xml, sharedStrings.xml, hojas, cabeceras...)
/// reemplazando las etiquetas en los nodos de texto.
fn resolve_xml_part(ext: &str, xml: &str, user: &User) -> String {
    let re_tag = Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*)\}\}").unwrap();

    // Primer pase: etiqueta completa dentro de un solo nodo <w:t> / <t>.
    let mut out = String::new();
    let mut last = 0;
    if ext.starts_with("xls") {
        let re_t = Regex::new(r"(?s)<t\b([^>]*)>(.*?)</t>").unwrap();
        for caps in re_t.captures_iter(xml) {
            let m = caps.get(0).unwrap();
            out.push_str(&xml[last..m.start()]);
            let attrs = caps.get(1).unwrap().as_str();
            let inner = caps.get(2).unwrap().as_str();
            let replaced = replace_text(inner, &re_tag, user);
            out.push_str(&format!("<t{}>{}</t>", attrs, replaced));
            last = m.end();
        }
    } else {
        let re_t = Regex::new(r"(?s)<w:t\b([^>]*)>(.*?)</w:t>").unwrap();
        for caps in re_t.captures_iter(xml) {
            let m = caps.get(0).unwrap();
            out.push_str(&xml[last..m.start()]);
            let attrs = caps.get(1).unwrap().as_str();
            let inner = caps.get(2).unwrap().as_str();
            let replaced = replace_text(inner, &re_tag, user);
            let space = if attrs.contains("xml:space") {
                String::new()
            } else if replaced != inner && replaced.trim() != replaced {
                " xml:space=\"preserve\"".to_string()
            } else {
                String::new()
            };
            out.push_str(&format!("<w:t{}{}>{}</w:t>", attrs, space, replaced));
            last = m.end();
        }
    }
    out.push_str(&xml[last..]);

    // Segundo pase (Word): etiquetas partidas entre varios <w:r> (Word suele
    // separar los runs al guardar). Si tras el primer pase sigue habiendo
    // `{{` dentro de un párrafo, se fusionan sus runs y se re-emite el texto.
    if !ext.starts_with("xls") && out.contains("{{") {
        out = merge_split_runs(&out, &re_tag, user);
    }
    out
}

/// Reemplaza etiquetas en un fragmento de texto puro (sin marcas XML).
fn replace_text(text: &str, re_tag: &Regex, user: &User) -> String {
    re_tag
        .replace_all(text, |caps: &regex::Captures| {
            let key = &caps[1];
            match value_for(key, user) {
                Some(value) => xml_escape(&value),
                None => caps[0].to_string(),
            }
        })
        .into_owned()
}

/// Fusiona los runs de los párrafos que aún contienen `{{`, resolviendo las
/// etiquetas que quedaron partidas entre nodos de texto consecutivos.
fn merge_split_runs(xml: &str, re_tag: &Regex, user: &User) -> String {
    let re_p = Regex::new(r"(?s)<w:p\b[^>]*>.*?</w:p>").unwrap();
    let re_popen = Regex::new(r"^<w:p\b[^>]*>").unwrap();
    let re_ppr = Regex::new(r"(?s)<w:pPr\b[^>]*>.*?</w:pPr>").unwrap();
    let re_rpr = Regex::new(r"(?s)<w:rPr\b[^>]*>.*?</w:rPr>").unwrap();
    let re_t = Regex::new(r"(?s)<w:t\b[^>]*>(.*?)</w:t>").unwrap();

    let mut out = String::new();
    let mut last = 0;
    for caps in re_p.captures_iter(xml) {
        let m = caps.get(0).unwrap();
        out.push_str(&xml[last..m.start()]);

        let para = caps.get(0).unwrap().as_str();
        if !para.contains("{{") {
            out.push_str(para);
        } else {
            // Texto concatenado de todos los runs del párrafo.
            let mut text = String::new();
            for t in re_t.captures_iter(para) {
                text.push_str(t.get(1).unwrap().as_str());
            }
            let resolved = replace_text(&text, re_tag, user);

            // Conserva el pPr y el rPr del primer run para no perder el formato.
            let open = re_popen.find(para).map(|m| m.as_str().to_string()).unwrap_or_else(|| "<w:p>".to_string());
            let ppr = re_ppr.find(para).map(|m| m.as_str().to_string()).unwrap_or_default();
            let rpr = re_rpr.find(para).map(|m| m.as_str().to_string()).unwrap_or_default();
            out.push_str(&open);
            out.push_str(&ppr);
            out.push_str(&format!(
                "<w:r>{0}<w:t xml:space=\"preserve\">{1}</w:t></w:r></w:p>",
                rpr,
                xml_escape(&resolved)
            ));
        }
        last = m.end();
    }
    out.push_str(&xml[last..]);
    out
}

/// Escapa un valor para incrustarlo como contenido XML.
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

    fn user() -> User {
        User {
            id: "u1".to_string(),
            username: "user1".to_string(),
            name: "User 1".to_string(),
            dni: Some("12345678".to_string()),
            cargo: Some("Director".to_string()),
            email: Some("user1@empresa.local".to_string()),
        }
    }

    #[test]
    fn replaces_known_tags_in_plain_text() {
        let re = Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*)\}\}").unwrap();
        assert_eq!(replace_text("Hola {{nombre_usuario}}", &re, &user()), "Hola User 1");
        assert_eq!(replace_text("{{dni}}", &re, &user()), "12345678");
        assert_eq!(replace_text("{{desconocida}}", &re, &user()), "{{desconocida}}");
        assert_eq!(replace_text("{{email}}", &re, &user()), "user1@empresa.local");
    }

    #[test]
    fn value_for_returns_user_fields() {
        let u = user();
        assert_eq!(value_for("nombre_usuario", &u).unwrap(), "User 1");
        assert_eq!(value_for("cargo_usuario", &u).unwrap(), "Director");
        assert_eq!(value_for("dni", &u).unwrap(), "12345678");
        assert_eq!(value_for("email", &u).unwrap(), "user1@empresa.local");
        assert!(value_for("fecha_actual", &u).is_some());
        let fecha = value_for("fecha_actual", &u).unwrap();
        let re = Regex::new(r"^\d{2}/\d{2}/\d{4} \d{2}:\d{2}$").unwrap();
        assert!(re.is_match(&fecha), "fecha_actual debe tener formato dd/mm/yyyy HH:mm, era {fecha}");
        assert!(value_for("nope", &u).is_none());
    }

    #[test]
    fn xml_escape_protects_special_chars() {
        assert_eq!(xml_escape("a&b<c>\"d\'"), "a&amp;b&lt;c&gt;&quot;d&apos;");
    }

    #[test]
    fn resolves_tags_inside_docx_document() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>
<w:p><w:r><w:t>Hola {{nombre_usuario}}, hoy es {{fecha_actual}}</w:t></w:r></w:p>
</w:body>
</w:document>"#;
        let resolved = resolve_xml_part("docx", xml, &user());
        assert!(!resolved.contains("{{nombre_usuario}}"));
        assert!(resolved.contains("User 1"));
        assert!(resolved.contains("Hola User 1, hoy es"));
        assert!(resolved.contains("</w:t>"));
    }

    #[test]
    fn merges_tags_split_across_runs() {
        let xml = r#"<w:p><w:r><w:t>Hola {{nombre_usu</w:t></w:r><w:r><w:t>ario}}</w:t></w:r></w:p>"#;
        let resolved = resolve_xml_part("docx", xml, &user());
        assert!(resolved.contains("User 1"));
        assert!(!resolved.contains("{{"));
    }

    #[test]
    fn non_ooxml_returns_none() {
        assert!(resolve(b"not a zip", &user(), "docx").is_none());
    }

    /// Construye un ZIP mínimo con la estructura de un docx y un `document.xml`
    /// que contiene una etiqueta, para validar la reescritura completa del ZIP.
    fn build_docx_with_tag() -> Vec<u8> {
        use std::io::Write as _;
        let cursor = Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(cursor);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zip.start_file("[Content_Types].xml", opts).unwrap();
        zip.write_all(b"<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/></Types>").unwrap();
        zip.start_file("word/document.xml", opts).unwrap();
        zip.write_all(br#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>
<w:p><w:r><w:t>Sr. {{nombre_usuario}}, DNI {{dni}}</w:t></w:r></w:p>
</w:body>
</w:document>"#).unwrap();
        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn resolves_tags_in_real_docx_zip() {
        let docx = build_docx_with_tag();
        let resolved = resolve(&docx, &user(), "docx").expect("debería resolver");
        assert_ne!(resolved, docx);

        // Vuelve a abrir el ZIP resuelto y comprueba el contenido del document.xml.
        let mut archive = zip::ZipArchive::new(Cursor::new(resolved)).unwrap();
        let mut xml = String::new();
        {
            let mut file = archive.by_name("word/document.xml").unwrap();
            file.read_to_string(&mut xml).unwrap();
        }
        assert!(!xml.contains("{{"));
        assert!(xml.contains("User 1"));
        assert!(xml.contains("12345678"));
    }

    #[test]
    fn zip_without_tags_returns_none() {
        let docx = build_docx_with_tag();
        let plain = {
            let mut archive = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
            let mut xml = String::new();
            archive.by_name("word/document.xml").unwrap().read_to_string(&mut xml).unwrap();
            xml.replace("{{nombre_usuario}}", "Usuario").replace("{{dni}}", "9999")
        };
        let no_tags = {
            use std::io::Write as _;
            let cursor = Cursor::new(Vec::new());
            let mut zip = zip::ZipWriter::new(cursor);
            let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            zip.start_file("[Content_Types].xml", opts).unwrap();
            zip.write_all(b"<Types/>").unwrap();
            zip.start_file("word/document.xml", opts).unwrap();
            zip.write_all(plain.as_bytes()).unwrap();
            zip.finish().unwrap().into_inner()
        };
        assert!(resolve(&no_tags, &user(), "docx").is_none());
    }
}
