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

El backend sirve la copia ejecutable desde:

```text
/plugins/compartir/plugin.js
```

La configuración de ONLYOFFICE la registra con el identificador `compartir`. La carpeta fuente del plugin se mantiene en `frontend-vue/src/plugins/customs/compartir`; después de cambiar `plugin_code.js`, sincroniza la copia en `backend-rust/public/plugins/compartir/plugin.js` y reinicia el backend.
