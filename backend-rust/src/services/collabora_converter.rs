use reqwest::{multipart, Client};

use crate::models::Document;

fn converter_url() -> String {
    let base = std::env::var("COLLABORA_CONVERTER_URL")
        .or_else(|_| std::env::var("COLLABORA_URL"))
        .unwrap_or_else(|_| "http://localhost:8093".to_string());
    let base = base.trim_end_matches('/');
    if base.ends_with("/cool/convert-to/pdf") {
        base.to_string()
    } else {
        format!("{base}/cool/convert-to/pdf")
    }
}

pub fn is_supported(doc: &Document) -> bool {
    matches!(doc.ext.to_ascii_lowercase().as_str(), "doc" | "docx" | "xls" | "xlsx")
}

pub async fn to_pdf(doc: &Document, content: &[u8]) -> Result<Vec<u8>, String> {
    let file = multipart::Part::bytes(content.to_vec())
        .file_name(format!("{}.{}", doc.name, doc.ext))
        .mime_str(&doc.mime)
        .map_err(|e| format!("No se pudo preparar el archivo para Collabora: {e}"))?;
    let form = multipart::Form::new().part("data", file);
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("No se pudo crear el cliente Collabora: {e}"))?;

    let response = client
        .post(converter_url())
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("No se pudo contactar a Collabora CODE: {e}"))?;
    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("No se pudo leer la respuesta de Collabora ({status}): {e}"))?
        .to_vec();
    if !status.is_success() {
        return Err(format!("Collabora rechazó la conversión ({status}): {}", String::from_utf8_lossy(&bytes)));
    }
    if bytes.len() < 5 || &bytes[..5] != b"%PDF-" {
        return Err("Collabora devolvió una respuesta que no es PDF".to_string());
    }
    Ok(bytes)
}
