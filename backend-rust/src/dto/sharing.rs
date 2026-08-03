use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ShareRequest {
    pub user_id: String,
}

/// Compartición de un documento con otro usuario.
#[derive(Debug, Serialize, ToSchema)]
pub struct ShareResponse {
    /// Identificador único de la compartición.
    #[schema(example = "5")]
    pub id: String,
    /// ID del documento compartido.
    #[schema(example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")]
    pub document_id: String,
    /// ID del usuario con el que se compartió.
    #[schema(example = "2")]
    pub user_id: String,
    /// Nombre del usuario con el que se compartió.
    #[schema(example = "Ana García")]
    pub user_name: String,
    /// ID del usuario propietario que otorgó el acceso.
    #[schema(example = "1")]
    pub shared_by: String,
    /// Nombre del usuario que otorgó el acceso.
    #[schema(example = "Juan Pérez")]
    pub shared_by_name: String,
    /// Nivel de permiso otorgado: `read` o `write`.
    #[schema(example = "write")]
    pub permission: String,
    /// Fecha en que se creó la compartición (ISO 8601).
    #[schema(example = "2026-08-02T15:10:00Z")]
    pub created_at: String,
}

/// Resultado de búsqueda de usuarios.
#[derive(Debug, Serialize, ToSchema)]
pub struct UserSearchResult {
    /// Identificador único del usuario.
    #[schema(example = "2")]
    pub id: String,
    /// Nombre de usuario para iniciar sesión.
    #[schema(example = "ana")]
    pub username: String,
    /// Nombre visible del usuario.
    #[schema(example = "Ana García")]
    pub name: String,
    /// DNI del usuario (si está registrado).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "12345678", value_type = String)]
    pub dni: Option<String>,
    /// Cargo o puesto del usuario (si está registrado).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Contadora", value_type = String)]
    pub cargo: Option<String>,
}

/// Cambios a aplicar en lote sobre las comparticiones de un documento.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ShareSyncRequest {
    /// IDs de usuario a los que se otorgará acceso.
    #[schema(example = json!(["3", "4"]), value_type = Vec<String>)]
    pub add: Vec<String>,
    /// IDs de usuario a los que se revocará el acceso.
    #[schema(example = json!(["5"]), value_type = Vec<String>)]
    pub remove: Vec<String>,
}

/// Agrupación de los resultados de la búsqueda de comparticiones.
#[derive(Debug, Serialize, ToSchema)]
pub struct ShareSearchData {
    /// Usuarios que ya tienen acceso al documento.
    pub compartidos: Vec<UserSearchResult>,
    /// Usuarios candidatos encontrados que aún no tienen acceso.
    pub encontrados: Vec<UserSearchResult>,
}

/// Respuesta de la búsqueda de comparticiones de un documento.
#[derive(Debug, Serialize, ToSchema)]
pub struct ShareSearchResponse {
    /// Datos con los usuarios ya compartidos y los encontrados.
    pub data: ShareSearchData,
}
