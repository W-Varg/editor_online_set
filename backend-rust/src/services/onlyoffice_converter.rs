use std::path::Path;

use reqwest::Client;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::helpers::{config, url};
use crate::models::Document;

fn converter_url() -> String {
    let base = std::env::var("ONLYOFFICE_CONVERTER_URL")
        .or_else(|_| std::env::var("ONLYOFFICE_URL"))
        .unwrap_or_else(|_| "http://localhost:8092".to_string());
    let base = base.trim_end_matches('/');
    let base = base.strip_suffix("/healthcheck").unwrap_or(base);
    if base.ends_with("/ConvertService.ashx") || base.ends_with("/converter") {
        base.to_string()
    } else {
        format!("{base}/converter")
    }
}

fn file_type(ext: &str) -> &str {
    match ext.to_ascii_lowercase().as_str() {
        "doc" => "doc",
        "docx" => "docx",
        "xls" => "xls",
        "xlsx" => "xlsx",
        "ppt" => "ppt",
        "pptx" => "pptx",
        _ => ext,
    }
}

fn conversion_key(doc: &Document, content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(doc.id.as_bytes());
    hasher.update(content);
    format!("{}-{:x}", doc.id, hasher.finalize())
}

pub async fn to_pdf(doc: &Document, content: &[u8], source_url: Option<String>) -> Result<Vec<u8>, String> {
    let backend_url = config::public_backend_url(8091);
    let source_url = source_url.unwrap_or_else(|| format!("{}/download/{}", backend_url, doc.id));
    let request = json!({
        "async": false,
        "filetype": file_type(&doc.ext),
        "outputtype": "pdf",
        "url": source_url,
        "title": format!("{}.{}", doc.name, doc.ext),
        "key": conversion_key(doc, content)
    });
    let jwt_secret = std::env::var("ONLYOFFICE_JWT_SECRET")
        .or_else(|_| std::env::var("JWT_SECRET"))
        .unwrap_or_else(|_| "my-secret-key".to_string());
    let token = encode(
        &Header::default(),
        &json!({ "payload": request }),
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ).map_err(|e| format!("No se pudo firmar la solicitud de conversión: {e}"))?;
    let key = request.get("key").and_then(Value::as_str).unwrap_or_default();

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("No se pudo crear el cliente ONLYOFFICE: {e}"))?;

    let response = client
        .post(format!("{}?shardkey={}", converter_url(), key))
        .bearer_auth(&token)
        .header(reqwest::header::ACCEPT, "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("No se pudo contactar a ONLYOFFICE: {e}"))?;

    let status = response.status();
    let raw_body = response
        .text()
        .await
        .map_err(|e| format!("No se pudo leer la respuesta de ONLYOFFICE ({status}): {e}"))?;
    let body = parse_conversion_response(&raw_body)
        .map_err(|e| format!("Respuesta inválida de ONLYOFFICE ({status}): {e}"))?;

    if !status.is_success() {
        return Err(format!("ONLYOFFICE rechazó la conversión: {body}"));
    }

    if body.get("error").and_then(Value::as_i64).unwrap_or(0) != 0 {
        return Err(format!("ONLYOFFICE devolvió un error de conversión: {body}"));
    }
    if body.get("endConvert").and_then(Value::as_bool) == Some(false) {
        return Err(format!("ONLYOFFICE aún no terminó la conversión: {body}"));
    }

    let file_url = body
        .get("fileUrl")
        .or_else(|| body.get("fileurl"))
        .and_then(Value::as_str)
        .ok_or_else(|| format!("ONLYOFFICE no devolvió fileUrl: {body}"))?;

    let pdf_response = client
        .get(file_url)
        .send()
        .await
        .map_err(|e| format!("No se pudo descargar el PDF de ONLYOFFICE: {e}"))?;
    if !pdf_response.status().is_success() {
        return Err(format!("ONLYOFFICE no permitió descargar el PDF: {}", pdf_response.status()));
    }

    let pdf = pdf_response
        .bytes()
        .await
        .map_err(|e| format!("No se pudo leer el PDF de ONLYOFFICE: {e}"))?
        .to_vec();
    if pdf.len() < 5 || &pdf[..5] != b"%PDF-" {
        return Err("ONLYOFFICE devolvió un archivo que no es PDF".to_string());
    }
    Ok(pdf)
}

fn parse_conversion_response(body: &str) -> Result<Value, String> {
    if let Ok(json) = serde_json::from_str::<Value>(body) {
        return Ok(json);
    }

    let value = |tag: &str| -> Option<String> {
        let open = format!("<{tag}>");
        let close = format!("</{tag}>");
        let start = body.find(&open)? + open.len();
        let end = body[start..].find(&close)? + start;
        Some(body[start..end].trim().to_string())
    };

    if let Some(error) = value("Error") {
        return Ok(json!({ "error": error.parse::<i64>().unwrap_or(-1) }));
    }
    if let Some(file_url) = value("FileUrl") {
        return Ok(json!({
            "endConvert": value("EndConvert").as_deref() == Some("True"),
            "fileUrl": file_url,
            "percent": value("Percent").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0)
        }));
    }

    Err(format!("cuerpo no JSON/XML: {}", body.chars().take(500).collect::<String>()))
}

pub fn is_supported(doc: &Document) -> bool {
    matches!(doc.ext.to_ascii_lowercase().as_str(), "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx")
}

#[allow(dead_code)]
pub fn source_url(doc_id: &str) -> String {
    format!("{}/download/{}", config::public_backend_url(8091), url::urlencoding(doc_id))
}

#[allow(dead_code)]
pub fn is_pdf(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("pdf")
}
