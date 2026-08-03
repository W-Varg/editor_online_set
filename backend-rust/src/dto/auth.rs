use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::User;

/// Credenciales de inicio de sesión.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// Nombre de usuario registrado en el sistema.
    #[schema(example = "user1")]
    pub username: String,
    /// Contraseña en texto plano del usuario.
    #[schema(example = "Admin123@")]
    pub password: String,
}

/// Respuesta exitosa de autenticación.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    /// Token JWT de acceso. Debe enviarse como `Authorization: Bearer <token>`.
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    /// Datos del usuario autenticado.
    pub user: User,
}
