use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Datos necesarios para crear una plantilla.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTemplate {
    /// Nombre de la plantilla (sin extensión).
    #[schema(example = "Acta de reunión")]
    pub name: String,
    /// Extensión del archivo (docx, xlsx, odt, etc.).
    #[schema(example = "docx")]
    pub ext: String,
    /// Editor preferido para abrir la plantilla: `onlyoffice` o `collabora`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "onlyoffice", value_type = String)]
    pub editor: Option<String>,
    /// ID de un documento existente del que copiar el contenido inicial.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789", value_type = String)]
    pub source_document_id: Option<String>,
}

/// Datos para renombrar una plantilla.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RenameTemplate {
    /// Nuevo nombre de la plantilla (sin extensión).
    #[schema(example = "Acta de reunión v2")]
    pub name: String,
}

/// Representación de una plantilla en listas y consultas.
#[derive(Debug, Serialize, ToSchema)]
pub struct TemplateResponse {
    /// Identificador único de la plantilla (UUID).
    #[schema(example = "7f8e9d0a-1b2c-3d4e-5f6a-7b8c9d0e1f2a")]
    pub id: String,
    /// Nombre de la plantilla (sin extensión).
    #[schema(example = "Acta de reunión")]
    pub name: String,
    /// Extensión del archivo (docx, xlsx, odt, etc.).
    #[schema(example = "docx")]
    pub ext: String,
    /// Tipo MIME del contenido de la plantilla.
    #[schema(example = "application/vnd.openxmlformats-officedocument.wordprocessingml.document")]
    pub mime: String,
    /// Editor asociado: `onlyoffice` o `collabora`.
    #[schema(example = "onlyoffice")]
    pub editor: String,
    /// Tamaño del archivo en bytes.
    #[schema(example = 18432)]
    pub size: u64,
    /// ID del usuario propietario de la plantilla.
    #[schema(example = "1")]
    pub owner_id: String,
    /// Nombre visible del propietario.
    #[schema(example = "Juan Pérez")]
    pub owner_name: String,
    /// Fecha de creación (ISO 8601).
    #[schema(example = "2026-08-01T10:00:00Z")]
    pub created_at: String,
    /// Fecha de última modificación (ISO 8601).
    #[schema(example = "2026-08-02T12:00:00Z")]
    pub updated_at: String,
}
