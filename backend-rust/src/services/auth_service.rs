use crate::db::DbConn;
use crate::dto::{LoginRequest, AuthResponse};
use crate::helpers::jwt;
use crate::repos::user_repo;

const JWT_SECRET: &str = "secreto-jwt-editor-online-2024";

pub fn login(db: &DbConn, req: &LoginRequest) -> Option<AuthResponse> {
    let user = user_repo::authenticate(db, &req.username, &req.password)?;
    let token = jwt::create(JWT_SECRET, &user.id, &user.name, "", 86400);
    Some(AuthResponse { token, user })
}
