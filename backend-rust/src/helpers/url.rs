use axum::http::HeaderMap;

pub fn urlencoding(s: &str) -> String {
    s.as_bytes().iter().map(|&c| {
        match c {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (c as char).to_string(),
            b' ' => "+".to_string(),
            _ => format!("%{:02X}", c),
        }
    }).collect()
}

pub fn hostname(headers: &HeaderMap) -> Option<String> {
    let host = headers.get("host")?.to_str().ok()?;
    Some(host.split(':').next().unwrap_or(host).to_string())
}

pub fn public_service_url(headers: &HeaderMap, port: u16) -> String {
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("http");
    let host = hostname(headers).unwrap_or_else(|| "localhost".to_string());
    format!("{}://{}:{}", scheme, host, port)
}
