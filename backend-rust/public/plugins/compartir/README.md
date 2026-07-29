# Plugin Compartir

Plugin visual de ONLYOFFICE para administrar los usuarios que tienen acceso al documento actual.

## Comportamiento

- Abre un panel lateral con el campo `dni o nombre` y un botón de búsqueda.
- Busca al presionar Enter o el botón con el icono de lupa.
- Marca con checkbox los usuarios que ya tienen acceso.
- Permite seleccionar usuarios encontrados y desmarcar usuarios compartidos.
- `Guardar` muestra una confirmación y sincroniza únicamente los cambios.
- `Cancelar` restaura la selección visible sin enviar cambios.
- Escucha `onThemeChanged` para adaptar el panel al tema de ONLYOFFICE.

## Contrato del backend

El plugin recibe mediante `pluginsData` el documento, el JWT del usuario y la URL pública del backend:

```json
["document-id", "jwt-del-usuario", "http://backend:8091"]
```

Búsqueda:

```http
GET /api/documents/{id}/shares/search?q=dni%20o%20nombre
Authorization: Bearer <jwt>
```

Respuesta:

```json
{
  "data": {
    "compartidos": [],
    "encontrados": []
  }
}
```

Sincronización:

```http
PUT /api/documents/{id}/shares/sync
Authorization: Bearer <jwt>
Content-Type: application/json
```

```json
{
  "add": ["user-id-to-add"],
  "remove": ["user-id-to-remove"]
}
```

El backend aplica los deltas y conserva los accesos que no pertenecen a la búsqueda actual.

## Integración

Esta es la ubicación canónica y única del plugin:

```text
backend-rust/public/plugins/compartir/
```

El backend sirve el paquete desde:

```text
/plugins/compartir/plugin.js
```

La configuración de ONLYOFFICE la registra automáticamente con el identificador `compartir` mediante `editorConfig.plugins`. No es necesario instalarlo desde el administrador de complementos de DocumentServer ni copiar archivos dentro del contenedor `onlyoffice/documentserver`.

El flujo es:

1. El frontend solicita al backend la configuración del editor.
2. El backend agrega esta entrada a `editorConfig.plugins.plugins`:

   ```json
   {
     "autostart": false,
      "plugins": [
        {
          "id": "compartir",
          "src": "http://<backend-publico>:8091/plugins/compartir/plugin.js"
        }
     ]
   }
   ```

3. El navegador descarga `plugin.js` desde el backend y ONLYOFFICE muestra el botón del plugin.

La URL del plugin se construye con el host de la solicitud al backend. Por eso, si accedes al frontend usando `http://localhost:8090`, el editor recibirá normalmente `http://localhost:8091`; si accedes usando la IP de intranet, recibirá esa misma IP en el puerto `8091`. La URL debe ser accesible desde el navegador.

Para las URLs de documentos y callbacks del backend, define `PUBLIC_BACKEND_URL`. En un entorno local puede ser:

```env
PUBLIC_BACKEND_URL=http://localhost:8091
```

Para acceder desde otros equipos de la intranet, usa la IP del equipo que ejecuta el backend, por ejemplo:

```env
PUBLIC_BACKEND_URL=http://172.27.38.53:8091
```

Si DocumentServer necesita resolver el backend desde Docker, usa `host.docker.internal` o la IP de la intranet. El `docker-compose.yml` ya declara `host.docker.internal:host-gateway` para DocumentServer.

## Verificación

Reinicia el backend para que tome el código actualizado:

```bash
cd backend-rust
cargo run
```

Comprueba que el archivo sea público:

```bash
curl -I http://localhost:8091/plugins/compartir/plugin.js
```

Comprueba la configuración que recibe el editor y confirma que `src` apunta al backend:

```bash
curl -s http://localhost:8091/api/onlyoffice/config/<document-id> \
  -H "Authorization: Bearer <jwt>" | jq '.editorConfig.plugins'
```

Si el plugin no aparece después de actualizarlo, cierra el editor y abre una nueva sesión para evitar que el navegador reutilice el JavaScript cacheado. DocumentServer no necesita reiniciarse cuando el plugin se sirve externamente desde el backend.

El paquete conserva `config.json`, `index.html`, `style.css` e iconos para mantener una estructura compatible con instalaciones manuales, pero la carga utilizada por este proyecto es la URL registrada en `editorConfig.plugins`.
