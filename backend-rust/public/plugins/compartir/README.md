# Plugin Compartir

Plugin visual de ONLYOFFICE para administrar los usuarios que tienen acceso al documento actual.

## Comportamiento

- Abre un panel lateral con un campo de búsqueda (DNI o nombre) y un botón de búsqueda.
- Busca al presionar Enter o el botón.
- Muestra los usuarios encontrados y los que ya tienen acceso, con un checkbox para cada uno.
- Los usuarios con acceso ya marcados aparecen como compartidos; desmarcarlos revoca el acceso.
- `Compartir` abre una ventana modal de confirmación que resume los cambios (añadir/quitar).
- Al aceptar, se sincronizan únicamente los cambios con el backend; al rechazar, no se envía nada.
- Escucha `onThemeChanged` para adaptar el panel al tema de ONLYOFFICE.
- El botón del footer cierra el panel.

## Contrato del backend

El plugin recibe los datos de sesión inyectados por el backend en
`editorConfig.plugins.options["asc.{GUID}"]`, que el SDK expone como
`window.Asc.plugin.info.options`:

```json
{
  "docId": "document-id",
  "token": "jwt-del-usuario",
  "backendUrl": "http://backend:8091"
}
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

## Archivos

```text
compartir/
├── config.json        registro del plugin (guid asc.{8f2a1c40-7b3d-4e21-9a6f-000000000001})
├── index.html         panel lateral (buscador + listado + pie con botones)
├── plugin.js          lógica del panel, llamadas a la API y apertura del modal
├── confirmar.html     ventana modal de confirmación
├── confirmar.js       lógica del modal (resumen + Aceptar/Rechazar)
├── resources/
│   ├── light/         iconos para tema claro (100% a 200%)
│   └── dark/          iconos para tema oscuro (100% a 200%)
└── README.md
```

No se usa `style.css` propio: el plugin reutiliza las clases nativas del SDK
(`plugins.css`, `.form-control`, `.btn-text-default`, `.button_wrapper`).

## Integración

Esta es la ubicación canónica y única del plugin:

```text
backend-rust/public/plugins/compartir/
```

El backend sirve el paquete desde:

```text
/plugins/compartir/
```

El plugin está registrado en `src/helpers/plugins.rs` como `CUSTOM_PLUGINS`
con `autostart: false`, `requires_owner: true` (solo el propietario del
documento puede usarlo) y `options`, que genera el objeto
`{ docId, token, backendUrl }` por documento. No es necesario instalarlo desde
el administrador de complementos de DocumentServer ni copiar archivos dentro
del contenedor `onlyoffice/documentserver`.

El flujo es:

1. El frontend solicita al backend la configuración del editor.
2. El backend agrega la entrada a `editorConfig.plugins.plugins`, genera la
   `pluginsData`, el `autostart` y las `options` por plugin.
3. El navegador descarga `plugin.js` desde el backend y ONLYOFFICE muestra el
   botón del plugin en la pestaña "Plugins".
4. El plugin lee `window.Asc.plugin.info.options` para obtener el documento,
   el JWT y la URL del backend.

La URL del plugin se construye con el host de la solicitud al backend. Por eso,
si accedes al frontend usando `http://localhost:8090`, el editor recibirá
normalmente `http://localhost:8091`; si accedes usando la IP de intranet,
recibirá esa misma IP en el puerto `8091`. La URL debe ser accesible desde el
navegador.

Para las URLs de documentos y callbacks del backend, define `PUBLIC_BACKEND_URL`.
En un entorno local puede ser:

```env
PUBLIC_BACKEND_URL=http://localhost:8091
```

Para acceder desde otros equipos de la intranet, usa la IP del equipo que
ejecuta el backend, por ejemplo:

```env
PUBLIC_BACKEND_URL=http://172.27.38.53:8091
```

Si DocumentServer necesita resolver el backend desde Docker, usa
`host.docker.internal` o la IP de la intranet. El `docker-compose.yml` ya
declara `host.docker.internal:host-gateway` para DocumentServer.

## Verificación

Reinicia el backend para que tome el código actualizado:

```bash
cd backend-rust
cargo run
```

Comprueba que los archivos sean públicos:

```bash
curl -I http://localhost:8091/plugins/compartir/config.json
curl -I http://localhost:8091/plugins/compartir/plugin.js
```

Comprueba la configuración que recibe el editor y confirma que `src` apunta al
backend:

```bash
curl -s http://localhost:8091/api/onlyoffice/config/<document-id> \
  -H "Authorization: Bearer <jwt>" | jq '.editorConfig.plugins'
```

Si el plugin no aparece después de actualizarlo, cierra el editor y abre una
nueva sesión para evitar que el navegador reutilice el JavaScript cacheado.
DocumentServer no necesita reiniciarse cuando el plugin se sirve externamente
desde el backend.
