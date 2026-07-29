use crate::db::DbConn;
use crate::dto::{LoginRequest, AuthResponse};
use crate::helpers::{config, jwt};
use crate::repos::user_repo;

pub fn login(db: &DbConn, req: &LoginRequest) -> Option<AuthResponse> {
    let user = user_repo::authenticate(db, &req.username, &req.password)?;
    let token = jwt::create(&config::jwt_secret(), &user.id, &user.name, "", 86400);
    Some(AuthResponse { token, user })
}
