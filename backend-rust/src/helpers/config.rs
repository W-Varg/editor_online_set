pub fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "my-secret-key".to_string())
}

pub fn public_backend_url(port: u16) -> String {
    std::env::var("PUBLIC_BACKEND_URL")
        .or_else(|_| std::env::var("BACKEND_URL"))
        .unwrap_or_else(|_| format!("http://host.docker.internal:{}", port))
}
