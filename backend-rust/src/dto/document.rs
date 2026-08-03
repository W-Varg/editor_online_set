use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Datos necesarios para crear un nuevo documento.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDocument {
    /// Nombre de archivo del documento (sin extensión).
    #[schema(example = "Contrato de servicios")]
    pub name: String,
    /// Extensión del archivo (docx, xlsx, odt, etc.).
    #[schema(example = "docx")]
    pub ext: String,
    /// Editor con el que se abrirá el documento: `onlyoffice` o `collabora`.
    #[schema(example = "onlyoffice")]
    pub editor: String,
    /// ID de la plantilla a usar como base. Si se omite, se crea en blanco.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "0a1b2c3d-...", value_type = String)]
    pub template_id: Option<String>,
}

/// Representación de un documento en las listas y consultas.
#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentResponse {
    /// Identificador único del documento (UUID).
    #[schema(example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789")]
    pub id: String,
    /// Nombre de archivo del documento (sin extensión).
    #[schema(example = "Contrato de servicios")]
    pub name: String,
    /// Extensión del archivo (docx, xlsx, odt, etc.).
    #[schema(example = "docx")]
    pub ext: String,
    /// Tipo MIME del contenido (ej. `application/vnd.openxmlformats-officedocument.wordprocessingml.document`).
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
    /// Nombre visible del propietario.
    #[schema(example = "Juan Pérez")]
    pub owner_name: String,
    /// Fecha de creación en formato ISO 8601.
    #[schema(example = "2026-08-02T14:00:00Z")]
    pub created_at: String,
    /// Fecha de última modificación en formato ISO 8601.
    #[schema(example = "2026-08-02T18:30:00Z")]
    pub updated_at: String,
    /// `true` si el documento está compartido con otro usuario.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = true)]
    pub shared: Option<bool>,
    /// ID del usuario que compartió el documento (presente en la pestaña "shared").
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "2", value_type = String)]
    pub shared_by: Option<String>,
    /// Nombre del usuario que compartió el documento.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Ana García", value_type = String)]
    pub shared_by_name: Option<String>,
}

#[allow(dead_code)]
/// Resultado de la eliminación de un documento.
#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteResponse {
    /// `true` si el documento fue eliminado correctamente.
    #[schema(example = true)]
    pub deleted: bool,
}

/// Resultado de una conversión a PDF.
#[derive(Debug, Serialize, ToSchema)]
pub struct ConvertResponse {
    /// Identificador interno del PDF generado (id del documento + `.pdf`).
    #[schema(example = "0a1b2c3d-4e5f-6789-abcd-ef0123456789.pdf")]
    pub pdf_id: String,
    /// URL pública para descargar o incrustar el PDF.
    #[schema(example = "http://localhost:8091/api/documents/0a1b2c3d.../pdf")]
    pub pdf_url: String,
    /// Estado de la conversión: `converted`, `already_converted`...
    #[schema(example = "converted")]
    pub status: String,
}
