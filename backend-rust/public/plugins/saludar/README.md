# Plugin "Saludar"

Plugin de ejemplo para ONLYOFFICE: el usuario escribe un texto en el **panel
lateral** y, al pulsar **Mostrar saludo**, se abre una **ventana modal** con el
texto. Replica el patrón de previsualización del plugin `doc2md` de la comunidad.

## Cómo se carga

Este proyecto **no instala plugins dentro del contenedor** de DocumentServer.
En su lugar, el backend sirve el plugin como archivo estático y lo inyecta en la
configuración del editor:

1. El backend genera la config de ONLYOFFICE (`/api/onlyoffice/config/:id`).
2. El registro de plugins (`src/helpers/plugins.rs`) declara este plugin con su
   GUID, carpeta y editores soportados.
3. El servicio agrega `editorConfig.plugins.pluginsData` =
   `http://<host>:8091/plugins/saludar/config.json` (queda disponible en la
   pestaña "Plugins"). El plugin **no arranca solo**: el sidebar se abre al
   hacer clic en él (ver nota de `autostart` en `src/helpers/plugins.rs`).

La URL pública se construye con el host de la petición, por lo que funciona tanto
en `localhost` como por IP de intranet (p. ej. `http://172.27.38.53:8091`).

## Estructura

```text
saludar/
├── config.json     → registro del plugin (GUID, variación de sidebar, iconos)
├── index.html      → panel lateral: input, "Mostrar saludo" y footer con
│                     "Cancelar" / "Cerrar" (cierran el sidebar)
├── plugin.js       → lógica del panel, cierre del sidebar y apertura del modal
├── saludo.html     → ventana modal que muestra el saludo
├── saludo.js       → lógica de la ventana modal
└── resources/
    ├── light/      → iconos del tema claro
    └── dark/       → iconos del tema oscuro
```

## SDK de plugins

El plugin carga el SDK desde el CDN oficial (`https://onlyoffice.github.io/
sdkjs-plugins/v1/`) para no depender de los recursos internos de DocumentServer.
Eso permite servirlo externamente desde cualquier host.

> **Entorno sin internet:** si la red no tiene salida a Internet, copia los
> archivos `plugins.js`, `plugins-ui.js` y `plugins.css` del contenedor
> (`/var/www/onlyoffice/documentserver/sdkjs-plugins/v1/`) a
> `backend-rust/public/sdkjs-plugins/v1/` y cambia las URLs de los `<script>`/`<link>`
> de `index.html` y `saludo.html` a rutas relativas `/sdkjs-plugins/v1/...`.

## Comunicación entre el panel y el modal

1. `plugin.js` abre el modal con `new window.Asc.PluginWindow()` + `show(variation)`.
2. El modal (`saludo.js`) avisa `sendToPlugin('onSaludoReady')`.
3. El panel (`plugin.js`) lo escucha con `previewWindow.attachEvent('onSaludoReady', ...)`
   y responde `previewWindow.command('onSaludoData', { texto })`.
4. El modal recibe `onSaludoData` con `attachEvent` y pinta el texto.

## Cierre del panel

Los botones **Cancelar** y **Cerrar** del footer invocan
`window.Asc.plugin.executeCommand('close', '')`, que cierra el sidebar.

## Cambios

- `v2.1.0`: el plugin se carga disponible pero **no abre el sidebar al iniciar**
  (`autostart: false`); se agregan los botones "Cancelar" y "Cerrar" que cierran
  el panel.
- `v2.0.0`: reescrito con buenas prácticas (sin CSS propio, SDK por CDN,
  formato de config moderno, registro declarativo en el backend).
