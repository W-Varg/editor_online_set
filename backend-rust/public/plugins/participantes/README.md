# Plugin Participantes para ONLYOFFICE

Este plugin agrega un boton en el panel lateral de ONLYOFFICE. Al hacer clic, abre un sidebar con:

- un `input` de texto
- un boton `Mostrar`
- un dialogo modal que enseña el texto capturado

## Estructura

La implementacion vive en dos sitios:

- `frontend-vue/src/plugins/participantes/`
- `backend-rust/public/plugins/participantes/`

ONLYOFFICE necesita servir los archivos desde una URL publica accesible por el Document Server en Docker. En este proyecto esa URL sale desde el backend, por eso la copia servida realmente es la de `backend-rust/public/plugins/participantes/`.

## Archivos del plugin

- `plugin.xml`: registra el plugin y su boton
- `index.html`: contenedor del sidebar
- `style.css`: estilos visuales
- `plugin.js`: logica del sidebar y del dialogo

## Como integrarlo con ONLYOFFICE en Docker

### 1. Asegura que el backend pueda servir el plugin

El backend Rust ya expone estaticos en `/plugins` desde `backend-rust/public/plugins`. Debes tener el plugin tambien en esa ruta con la misma estructura:

```text
backend-rust/public/plugins/participantes/
  index.html
  plugin.js
  plugin.xml
  style.css
```

### 2. Registra el plugin en la configuracion del editor

El backend que genera la configuracion de ONLYOFFICE debe incluir el plugin en `editorConfig.plugins.plugins`.

La URL debe apuntar a tu backend publico, por ejemplo:

```text
http://host.docker.internal:8091/plugins/participantes/plugin.js
```

o, si estas en intranet:

```text
http://172.27.38.53:8091/plugins/participantes/plugin.js
```

En este repositorio, el backend ya usa `PUBLIC_BACKEND_URL` para construir la URL publica.

### 3. Reinicia los servicios

```bash
docker compose down
docker compose up -d
```

Luego reinicia el backend si lo ejecutas con `cargo run`.

### 4. Verifica que ONLYOFFICE lo cargue

Abre un documento en ONLYOFFICE y revisa que aparezca el boton del plugin `Participantes`. Al pulsarlo:

1. se abre el sidebar
2. escribes un texto
3. presionas `Mostrar`
4. aparece el dialogo con el texto
5. puedes cerrarlo con `Cerrar`, con la `x`, o con `Esc`

## Configuracion de entorno recomendada

Define en tu `.env`:

```bash
PUBLIC_BACKEND_URL=http://host.docker.internal:8091
JWT_SECRET=my-secret-key
```

Si usas IP de intranet, cambia `PUBLIC_BACKEND_URL` a la IP real.

## Nota importante

El `src` del plugin debe ser accesible desde el contenedor de ONLYOFFICE, no solo desde tu navegador. Si el Document Server no puede descargar `plugin.js`, el plugin no aparecera.
