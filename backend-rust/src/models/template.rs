use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Plantilla de documento reutilizable (metadatos, el contenido se guarda en disco).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Template {
    /// Identificador único de la plantilla (UUID).
    #[schema(example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a")]
    pub id: String,
    /// Nombre de la plantilla (sin extensión).
    #[schema(example = "Acta de reunión")]
    pub name: String,
    /// Extensión del archivo (docx, xlsx, odt, etc.).
    #[schema(example = "docx")]
    pub ext: String,
    /// Tipo MIME del contenido.
    #[schema(example = "application/vnd.openxmlformats-officedocument.wordprocessingml.document")]
    pub mime: String,
    /// Editor asociado: `onlyoffice` o `collabora`.
    #[schema(example = "onlyoffice")]
    pub editor: String,
    /// Tamaño del archivo en bytes.
    #[schema(example = 18432)]
    pub size: u64,
    /// ID del usuario propietario.
    #[schema(example = "1")]
    pub owner_id: String,
    /// Fecha de creación (ISO 8601).
    #[schema(example = "2026-08-01T10:00:00Z")]
    pub created_at: String,
    /// Fecha de última modificación (ISO 8601).
    #[schema(example = "2026-08-02T12:00:00Z")]
    pub updated_at: String,
}
