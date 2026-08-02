# Plugins personalizados de ONLYOFFICE

El backend sirve estos plugins como archivos estáticos desde la ruta
`/plugins/<carpeta>/` y los inyecta en la configuración que envía a ONLYOFFICE.

## Cómo agregar un plugin nuevo

1. Crea la carpeta `public/plugins/<carpeta>/` con los archivos del plugin
   (mínimo `config.json`, `index.html` y el JS de la lógica).
2. El `config.json` debe tener un `guid` único con el formato
   `asc.{8f2a1c40-7b3d-4e21-9a6f-XXXXXXXXXXXX}`.
3. Registra el plugin en `src/helpers/plugins.rs` agregando una entrada a
   `CUSTOM_PLUGINS`:

   ```rust
   CustomPlugin {
       id: "asc.{...}",
       name: "Mi plugin",
       dir: "mi-plugin",
       editors: &["word", "cell", "slide"],
       autostart: false,
       requires_owner: false,
       options: None,
   }
   ```

   - `dir` es la carpeta que creaste en `public/plugins/`.
   - `editors` filtra los tipos de documento donde estará disponible.
   - `autostart: true` hace que el plugin arranque solo al abrir el editor;
     con `false` queda disponible en la pestaña "Plugins" (el sidebar no se
     abre solo).
   - `requires_owner: true` solo inyecta el plugin cuando el usuario es el
     propietario del documento.
   - `options` es un generador opcional `fn(&PluginContext) -> Value` que
     produce `editorConfig.plugins.options[guid]`; el plugin lo lee como
     `window.Asc.plugin.info.options`. Útil para pasar el documento, el JWT y
     la URL del backend, como hace `compartir`.

4. Reinicia el backend. El servicio `onlyoffice_service` genera las URLs de
   `pluginsData`, el `autostart`, el filtro por propietario y las `options`
   automáticamente.

## Plugins actuales

| Carpeta   | GUID                                        | Editores          | Autostart | Owner | Estado            |
|-----------|---------------------------------------------|-------------------|-----------|-------|-------------------|
| `saludar`   | `asc.{8f2a1c40-7b3d-4e21-9a6f-000000000002}` | word, cell, slide | no        | no    | Activo (ejemplo)  |
| `compartir` | `asc.{8f2a1c40-7b3d-4e21-9a6f-000000000001}` | word, cell, slide | no        | sí    | Activo            |
| `etiquetas` | `asc.{8f2a1c40-7b3d-4e21-9a6f-000000000003}` | word, cell        | no        | no    | Activo            |

## Etiquetas dinámicas (`{{key}}`)

El plugin `etiquetas` inserta texto literal `{{key}}` en el documento (en la
posición del cursor). Las etiquetas quedan guardadas **sin resolver** en
`data/{id}.bin` y el backend las sustituye por sus valores reales al
previsualizar (`GET /api/documents/{id}/preview`) o convertir a PDF
(`POST /api/documents/{id}/convert`).

- Catálogo y resolución: `src/services/tag_service.rs` (constante
  `TAG_DEFINITIONS` + `resolve(content, user, ext)`).
- API: `GET /api/tags` devuelve `[{key, label, description}]`.
- El contenido resuelto se sirve una sola vez en
  `GET /api/preview-source/{token}` (TTL 60s) porque el convertidor de
  ONLYOFFICE fetchea el documento por URL y no recibe bytes.
- Los valores se calculan con los datos del **usuario que previsualiza**
  (nombre, cargo, DNI, email) y la fecha actual del servidor.
- Soporta `.docx` (`word/document.xml` + headers/footers) y `.xlsx`
  (`xl/sharedStrings.xml` + hojas).

## Verificación

```bash
# Los archivos del plugin deben ser accesibles públicamente
curl -I http://localhost:8091/plugins/saludar/config.json
curl -I http://localhost:8091/plugins/compartir/config.json

# La config del editor debe incluir pluginsData, autostart y options
curl -s http://localhost:8091/api/onlyoffice/config/<document-id> \
  -H "Authorization: Bearer <jwt>" | jq '.editorConfig.plugins'
```
