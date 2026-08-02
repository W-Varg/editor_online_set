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
   }
   ```

   - `dir` es la carpeta que creaste en `public/plugins/`.
   - `editors` filtra los tipos de documento donde estará disponible.
   - `autostart: true` hace que el plugin arranque solo al abrir el editor;
     con `false` queda disponible en la pestaña "Plugins".

4. Reinicia el backend. El servicio `onlyoffice_service` genera las URLs de
   `pluginsData` automáticamente.

## Plugins actuales

| Carpeta   | GUID                                        | Editores          | Autostart | Estado               |
|-----------|---------------------------------------------|-------------------|-----------|----------------------|
| `saludar` | `asc.{8f2a1c40-7b3d-4e21-9a6f-000000000002}` | word, cell, slide | sí        | Activo (ejemplo)     |
| `compartir` | `asc.{8f2a1c40-7b3d-4e21-9a6f-000000000001}` | word, cell, slide | no        | No registrado aún    |

`compartir` queda en el repositorio como referencia (panel para administrar
accesos), pero hoy solo está activo `saludar`. Para activarlo, agrégalo a
`CUSTOM_PLUGINS` igual que el anterior.

## Verificación

```bash
# El plugin debe ser accesible públicamente
curl -I http://localhost:8091/plugins/saludar/config.json

# La config del editor debe incluir pluginsData con la URL del plugin
curl -s http://localhost:8091/api/onlyoffice/config/<document-id> \
  -H "Authorization: Bearer <jwt>" | jq '.editorConfig.plugins'
```
