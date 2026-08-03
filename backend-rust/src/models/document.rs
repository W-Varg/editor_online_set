use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Documento almacenado en el sistema (metadatos, el contenido se guarda en disco).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Document {
    /// Identificador único del documento (UUID).
    #[schema(example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")]
    pub id: String,
    /// Nombre del documento (sin extensión).
    #[schema(example = "Contrato de servicios")]
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
    #[schema(example = 24576)]
    pub size: u64,
    /// Estado del documento: `draft`, `converted`, `final`, `converting`...
    #[schema(example = "draft")]
    pub status: String,
    /// ID del usuario propietario.
    #[schema(example = "1")]
    pub owner_id: String,
    /// Fecha de creación (ISO 8601).
    #[schema(example = "2026-08-02T14:00:00Z")]
    pub created_at: String,
    /// Fecha de última modificación (ISO 8601).
    #[schema(example = "2026-08-02T18:30:00Z")]
    pub updated_at: String,
}
