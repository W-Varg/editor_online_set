use serde::Serialize;
use utoipa::ToSchema;

/// Etiqueta dinámica que puede insertarse en el contenido de un documento y que
/// el backend resuelve por su valor real al previsualizar.
#[derive(Debug, Serialize, ToSchema)]
pub struct TagDefinition {
    /// Clave única de la etiqueta (se inserta en el documento entre `{{ }}`).
    #[schema(example = "fecha_actual")]
    pub key: String,
    /// Etiqueta visible en la interfaz del plugin.
    #[schema(example = "Fecha actual")]
    pub label: String,
    /// Descripción de la etiqueta para el usuario.
    #[schema(example = "Se reemplaza por la fecha del día en que se previsualiza.")]
    pub description: String,
}
