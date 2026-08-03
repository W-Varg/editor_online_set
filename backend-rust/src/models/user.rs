use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Usuario registrado en el sistema.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    /// Identificador único del usuario.
    #[schema(example = "1")]
    pub id: String,
    /// Nombre de usuario para iniciar sesión.
    #[schema(example = "admin")]
    pub username: String,
    /// Nombre visible del usuario.
    #[schema(example = "Juan Pérez")]
    pub name: String,
    /// DNI del usuario (opcional).
    #[schema(example = "12345678", value_type = String)]
    pub dni: Option<String>,
    /// Cargo o puesto del usuario (opcional).
    #[schema(example = "Jefe de área", value_type = String)]
    pub cargo: Option<String>,
    /// Correo electrónico del usuario (opcional).
    #[schema(example = "juan@example.com", value_type = String)]
    pub email: Option<String>,
}
