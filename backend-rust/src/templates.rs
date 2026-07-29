use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub fn generate_sample_docx() -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let store = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("mimetype", store).unwrap();
    zip.write_all(b"application/vnd.openxmlformats-officedocument.wordprocessingml.document").unwrap();

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">").unwrap();
    zip.write_all(b"<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>").unwrap();
    zip.write_all(b"<Default Extension=\"xml\" ContentType=\"application/xml\"/>").unwrap();
    zip.write_all(b"<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>").unwrap();
    zip.write_all(b"</Types>").unwrap();

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">").unwrap();
    zip.write_all(b"<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>").unwrap();
    zip.write_all(b"</Relationships>").unwrap();

    zip.start_file("word/_rels/document.xml.rels", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">").unwrap();
    zip.write_all(b"</Relationships>").unwrap();

    zip.start_file("word/document.xml", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">").unwrap();
    zip.write_all(b"<w:body>").unwrap();
    zip.write_all(b"<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:rPr><w:b/><w:sz w:val=\"36\"/><w:color w:val=\"2E4057\"/></w:rPr><w:t>Informe de Comision</w:t></w:r></w:p>").unwrap();
    zip.write_all(b"<w:p><w:r><w:rPr><w:sz w:val=\"24\"/></w:rPr><w:t>Este es un documento de prueba generado automaticamente.</w:t></w:r></w:p>").unwrap();
    zip.write_all(b"<w:p><w:r><w:t>Puede ser editado colaborativamente con ONLYOFFICE o Collabora Online.</w:t></w:r></w:p>").unwrap();
    zip.write_all(b"<w:p><w:r><w:rPr><w:b/><w:sz w:val=\"28\"/></w:rPr><w:t>Contenido del Informe</w:t></w:r></w:p>").unwrap();
    zip.write_all(b"<w:p><w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr><w:t>Este documento demuestra las capacidades de edicion colaborativa en tiempo real.</w:t></w:r></w:p>").unwrap();
    zip.write_all(b"<w:p><w:r><w:t>Los usuarios pueden editar simultaneamente este documento desde diferentes ubicaciones.</w:t></w:r></w:p>").unwrap();
    zip.write_all(b"<w:p><w:r><w:t>Al finalizar, el documento puede ser convertido a PDF.</w:t></w:r></w:p>").unwrap();
    zip.write_all(b"</w:body>").unwrap();
    zip.write_all(b"</w:document>").unwrap();

    zip.finish().unwrap().into_inner()
}

pub fn generate_blank_docx() -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let store = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("mimetype", store).unwrap();
    zip.write_all(b"application/vnd.openxmlformats-officedocument.wordprocessingml.document").unwrap();

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">").unwrap();
    zip.write_all(b"<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>").unwrap();
    zip.write_all(b"<Default Extension=\"xml\" ContentType=\"application/xml\"/>").unwrap();
    zip.write_all(b"<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>").unwrap();
    zip.write_all(b"</Types>").unwrap();

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">").unwrap();
    zip.write_all(b"<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>").unwrap();
    zip.write_all(b"</Relationships>").unwrap();

    zip.start_file("word/_rels/document.xml.rels", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">").unwrap();
    zip.write_all(b"</Relationships>").unwrap();

    zip.start_file("word/document.xml", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">").unwrap();
    zip.write_all(b"<w:body>").unwrap();
    zip.write_all(b"<w:p/>").unwrap();
    zip.write_all(b"<w:sectPr/>").unwrap();
    zip.write_all(b"</w:body>").unwrap();
    zip.write_all(b"</w:document>").unwrap();

    zip.finish().unwrap().into_inner()
}

pub fn generate_sample_xlsx() -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let store = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("mimetype", store).unwrap();
    zip.write_all(b"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet").unwrap();

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">").unwrap();
    zip.write_all(b"<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>").unwrap();
    zip.write_all(b"<Default Extension=\"xml\" ContentType=\"application/xml\"/>").unwrap();
    zip.write_all(b"<Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>").unwrap();
    zip.write_all(b"<Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>").unwrap();
    zip.write_all(b"</Types>").unwrap();

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">").unwrap();
    zip.write_all(b"<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/>").unwrap();
    zip.write_all(b"</Relationships>").unwrap();

    zip.start_file("xl/_rels/workbook.xml.rels", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">").unwrap();
    zip.write_all(b"<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/>").unwrap();
    zip.write_all(b"</Relationships>").unwrap();

    zip.start_file("xl/workbook.xml", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">").unwrap();
    zip.write_all(b"<sheets><sheet name=\"Gastos\" sheetId=\"1\" r:id=\"rId1\"/></sheets>").unwrap();
    zip.write_all(b"</workbook>").unwrap();

    zip.start_file("xl/worksheets/sheet1.xml", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">").unwrap();
    zip.write_all(b"<sheetData>").unwrap();
    zip.write_all(b"<row r=\"1\"><c r=\"A1\" t=\"inlineStr\"><is><t>Item</t></is></c><c r=\"B1\" t=\"inlineStr\"><is><t>Monto (Bs)</t></is></c><c r=\"C1\" t=\"inlineStr\"><is><t>Observaciones</t></is></c></row>").unwrap();
    zip.write_all(b"<row r=\"2\"><c r=\"A2\" t=\"inlineStr\"><is><t>Pasajes aereos</t></is></c><c r=\"B2\" t=\"n\"><v>1500</v></c><c r=\"C2\" t=\"inlineStr\"><is><t>Ida y vuelta La Paz - Cochabamba</t></is></c></row>").unwrap();
    zip.write_all(b"<row r=\"3\"><c r=\"A3\" t=\"inlineStr\"><is><t>Hospedaje</t></is></c><c r=\"B3\" t=\"n\"><v>3200</v></c><c r=\"C3\" t=\"inlineStr\"><is><t>Hotel 4 noches</t></is></c></row>").unwrap();
    zip.write_all(b"<row r=\"4\"><c r=\"A4\" t=\"inlineStr\"><is><t>Viaticos</t></is></c><c r=\"B4\" t=\"n\"><v>1800</v></c><c r=\"C4\" t=\"inlineStr\"><is><t>3 dias de comision</t></is></c></row>").unwrap();
    zip.write_all(b"<row r=\"5\"><c r=\"A5\" t=\"inlineStr\"><is><t>Material de escritorio</t></is></c><c r=\"B5\" t=\"n\"><v>450</v></c><c r=\"C5\" t=\"inlineStr\"><is><t>Papeleria y fotocopias</t></is></c></row>").unwrap();
    zip.write_all(b"<row r=\"6\"><c r=\"A6\" t=\"inlineStr\"><is><t>Transporte local</t></is></c><c r=\"B6\" t=\"n\"><v>320</v></c><c r=\"C6\" t=\"inlineStr\"><is><t>Taxis y micros</t></is></c></row>").unwrap();
    zip.write_all(b"<row r=\"7\"><c r=\"A7\" t=\"inlineStr\"><is><t>Total</t></is></c><c r=\"B7\" t=\"n\"><v>7270</v></c><c r=\"C7\"/></row>").unwrap();
    zip.write_all(b"</sheetData>").unwrap();
    zip.write_all(b"</worksheet>").unwrap();

    zip.finish().unwrap().into_inner()
}

pub fn generate_blank_xlsx() -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let store = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("mimetype", store).unwrap();
    zip.write_all(b"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet").unwrap();

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">").unwrap();
    zip.write_all(b"<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>").unwrap();
    zip.write_all(b"<Default Extension=\"xml\" ContentType=\"application/xml\"/>").unwrap();
    zip.write_all(b"<Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>").unwrap();
    zip.write_all(b"<Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>").unwrap();
    zip.write_all(b"</Types>").unwrap();

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">").unwrap();
    zip.write_all(b"<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/>").unwrap();
    zip.write_all(b"</Relationships>").unwrap();

    zip.start_file("xl/_rels/workbook.xml.rels", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">").unwrap();
    zip.write_all(b"<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/>").unwrap();
    zip.write_all(b"</Relationships>").unwrap();

    zip.start_file("xl/workbook.xml", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">").unwrap();
    zip.write_all(b"<sheets><sheet name=\"Sheet1\" sheetId=\"1\" r:id=\"rId1\"/></sheets>").unwrap();
    zip.write_all(b"</workbook>").unwrap();

    zip.start_file("xl/worksheets/sheet1.xml", options).unwrap();
    zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>").unwrap();
    zip.write_all(b"<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">").unwrap();
    zip.write_all(b"<sheetData/>").unwrap();
    zip.write_all(b"</worksheet>").unwrap();

    zip.finish().unwrap().into_inner()
}

pub fn generate_pdf(title: &str, text: &str) -> Vec<u8> {
    let mut buf = Vec::new();

    let safe_title = title.escape_default().to_string();
    let safe_text = text.escape_default().to_string();

    let content = format!(
        "%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj

2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj

3 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792]
   /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>
endobj

4 0 obj
<< /Length 236 >>
stream
BT
/F1 24 Tf
50 750 Td
({}) Tj
ET
BT
/F1 12 Tf
50 700 Td
(Documento generado el: {}) Tj
ET
BT
/F1 11 Tf
50 650 Td
({}) Tj
ET
BT
/F1 10 Tf
50 600 Td
(--- Este documento ha sido convertido a PDF desde el editor online ---) Tj
ET
endstream
endobj

5 0 obj
<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>
endobj

xref
0 6
0000000000 65535 f 
0000000009 00000 n 
0000000058 00000 n 
0000000115 00000 n 
0000000266 00000 n 
0000000553 00000 n 

trailer
<< /Size 6 /Root 1 0 R >>
startxref
590
%%EOF",
        safe_title,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        safe_text
    );

    buf.extend_from_slice(content.as_bytes());
    buf
}
