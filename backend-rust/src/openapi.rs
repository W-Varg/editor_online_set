use utoipa::OpenApi;
use crate::{__path_health_handler, __path_root_handler, __path_serve_api_docs};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Editor Online Backend",
        description = "API del gestor de documentos en línea. Permite autenticación, \
            gestión de documentos y plantillas, edición con ONLYOFFICE y Collabora \
            (vía WOPI), uso compartido entre usuarios, conversión a PDF y el catálogo \
            de etiquetas reemplazables en el contenido.\n\n\
            Todos los endpoints que operan sobre datos de usuario requieren el header \
            `Authorization: Bearer <token>` obtenido desde `POST /api/auth/login`.",
        version = env!("CARGO_PKG_VERSION")
    ),
    servers(
        (url = "/", description = "Servidor local")
    ),
    paths(
        root_handler,
        health_handler,
        serve_api_docs,
        crate::controllers::auth_controller::login,
        crate::controllers::document_controller::list,
        crate::controllers::document_controller::create,
        crate::controllers::document_controller::get,
        crate::controllers::document_controller::delete,
        crate::controllers::document_controller::download,
        crate::controllers::document_controller::content,
        crate::controllers::document_controller::convert_to_pdf,
        crate::controllers::document_controller::preview,
        crate::controllers::document_controller::get_pdf,
        crate::controllers::sharing_controller::search_users,
        crate::controllers::sharing_controller::search_document_users,
        crate::controllers::sharing_controller::create,
        crate::controllers::sharing_controller::list,
        crate::controllers::sharing_controller::remove,
        crate::controllers::sharing_controller::sync,
        crate::controllers::collabora_controller::session,
        crate::controllers::collabora_controller::template_session,
        crate::controllers::collabora_controller::check_file_info,
        crate::controllers::collabora_controller::get_file,
        crate::controllers::collabora_controller::file_ops,
        crate::controllers::collabora_controller::put_file,
        crate::controllers::collabora_controller::template_check_file_info,
        crate::controllers::collabora_controller::template_get_file,
        crate::controllers::collabora_controller::template_file_ops,
        crate::controllers::collabora_controller::template_put_file,
        crate::controllers::onlyoffice_controller::config,
        crate::controllers::onlyoffice_controller::callback,
        crate::controllers::tag_controller::list_tags,
        crate::controllers::tag_controller::preview_source,
        crate::controllers::auth_controller::list_users,
        crate::controllers::template_controller::list,
        crate::controllers::template_controller::create,
        crate::controllers::template_controller::get,
        crate::controllers::template_controller::rename,
        crate::controllers::template_controller::delete,
        crate::controllers::template_controller::content,
        crate::controllers::template_controller::preview,
        crate::controllers::template_controller::config,
        crate::controllers::template_controller::callback
    ),
    components(
        schemas(
            crate::dto::auth::LoginRequest,
            crate::dto::auth::AuthResponse,
            crate::dto::document::CreateDocument,
            crate::dto::document::DocumentResponse,
            crate::dto::document::DeleteResponse,
            crate::dto::document::ConvertResponse,
            crate::dto::collabora::CollaboraSession,
            crate::dto::collabora::CheckFileInfo,
            crate::dto::collabora::JwtClaims,
            crate::dto::onlyoffice::OnlyOfficeConfig,
            crate::dto::onlyoffice::OnlyOfficeDocument,
            crate::dto::onlyoffice::OnlyOfficePermissions,
            crate::dto::onlyoffice::OnlyOfficeEditorConfig,
            crate::dto::onlyoffice::OnlyOfficeCustomization,
            crate::dto::onlyoffice::OnlyOfficePlugins,
            crate::dto::onlyoffice::OnlyOfficeUser,
            crate::dto::tag::TagDefinition,
            crate::controllers::sharing_controller::SharePayload,
            crate::dto::sharing::ShareResponse,
            crate::dto::sharing::UserSearchResult,
            crate::dto::sharing::ShareSyncRequest,
            crate::dto::sharing::ShareSearchData,
            crate::dto::sharing::ShareSearchResponse,
            crate::dto::system::RootResponse,
            crate::dto::system::HealthResponse,
            crate::models::User,
            crate::models::Document,
            crate::models::Template,
            crate::dto::template::CreateTemplate,
            crate::dto::template::RenameTemplate,
            crate::dto::template::TemplateResponse
        )
    )
)]
pub struct ApiDoc;
