# Plugin "Participantes" para ONLYOFFICE

Plugin que agrega un botón en la barra lateral derecha (junto a "Paragraph settings").
Al hacer clic abre un panel lateral con:

- Un campo de texto.
- Un botón **"Mostrar"**.
- Al presionar "Mostrar", el texto capturado se muestra en un **dialog/modal** que puede cerrarse con la `X`, el botón "Cerrar" o haciendo clic fuera del cuadro.

---

## 1. Estructura de archivos

```
participantes/
├── config.json              # Configuración/metadatos del plugin (obligatorio)
├── index.html                # Vista del panel lateral
├── plugin_code.js            # Lógica (input -> dialog)
├── README.md
└── resources/
    ├── css/
    │   └── style.css
    └── img/
        ├── icon.png           # 24x24 (icono normal en la barra)
        └── icon@2x.png        # 48x48 (icono retina)
```

> **Nota:** `index.html` referencia `../plugin.js`. Ese archivo **no se incluye** porque es parte del propio ONLYOFFICE Document Server (es el bridge de la API `window.Asc.plugin`). Se carga automáticamente cuando el plugin corre dentro del editor, así que no debes crearlo ni modificarlo — solo debe existir la ruta relativa correcta (ONLYOFFICE lo resuelve internamente al montar los plugins en `/sdkjs-plugins` o `/web-apps/apps/.../plugin.js` según la versión).

---

## 2. ¿Cómo funciona el botón "similar al de Paragraph settings"?

En el editor de ONLYOFFICE, cada plugin instalado con `"isVisual": true` y sin `buttons` en `variations` (visor tipo panel) **agrega automáticamente un ícono en la barra lateral derecha**, debajo de los íconos nativos (Párrafo ¶, Tabla, Imagen, Forma, Texto, Firma). Esto es controlado por la propiedad `isInsideMode: true` en `config.json`.

No necesitas maquetar tú mismo el botón: ONLYOFFICE renderiza el ícono (`resources/img/icon.png` / `icon@2x.png`) automáticamente en esa barra, en la posición que corresponde según el orden de instalación del plugin. Al hacer clic, abre el panel (`index.html`) en un `<iframe>` lateral, tal como se ve en tu segunda captura.

---

## 3. Cómo integrar el plugin a tu ONLYOFFICE en Docker (puerto 8092)

Existen **2 formas**. La recomendada para desarrollo/pruebas es el **volumen montado**; para producción, es mejor **construir una imagen propia**.

### Opción A — Montar el plugin como volumen (rápido, recomendado para probar)

1. Copia la carpeta `participantes/` a tu proyecto, por ejemplo:
   ```
   /home/dev/Documents/restringida/dev_proyects/editor_online/frontend-vue/src/plugins/participantes
   ```

2. Ubica cómo estás levantando el contenedor de Document Server. Si usas `docker run`, agrega un bind mount hacia la carpeta de plugins internos del contenedor, que normalmente es:

   ```
   /var/www/onlyoffice/documentserver/web-apps/apps/api/documents/plugins/participantes
   ```

   Ejemplo de comando (ajusta el nombre del contenedor/imagen si ya lo tienes corriendo):

   ```bash
   docker run -d \
     -p 8092:80 \
     --name onlyoffice-ds \
     -v /home/dev/Documents/restringida/dev_proyects/editor_online/frontend-vue/src/plugins/participantes:/var/www/onlyoffice/documentserver/web-apps/apps/api/documents/plugins/participantes \
     onlyoffice/documentserver
   ```

3. Si **ya tienes el contenedor corriendo** (no quieres recrearlo), usa `docker cp` para copiar el plugin dentro y luego reinicia el servicio de nginx/document server dentro del contenedor:

   ```bash
   docker cp /home/dev/Documents/restringida/dev_proyects/editor_online/frontend-vue/src/plugins/participantes \
     onlyoffice-ds:/var/www/onlyoffice/documentserver/web-apps/apps/api/documents/plugins/participantes

   docker exec -it onlyoffice-ds supervisorctl restart all
   ```

   > Reemplaza `onlyoffice-ds` por el nombre real de tu contenedor. Puedes verlo con `docker ps`.

4. **docker-compose** (si usas `docker-compose.yml`), agrega el volumen dentro del servicio del Document Server:

   ```yaml
   services:
     onlyoffice-documentserver:
       image: onlyoffice/documentserver
       ports:
         - "8092:80"
       volumes:
         - /home/dev/Documents/restringida/dev_proyects/editor_online/frontend-vue/src/plugins/participantes:/var/www/onlyoffice/documentserver/web-apps/apps/api/documents/plugins/participantes
   ```

   Luego:
   ```bash
   docker-compose up -d --force-recreate onlyoffice-documentserver
   ```

### Opción B — Reconstruir imagen con el plugin incluido (producción)

Crea un `Dockerfile` que extienda la imagen oficial:

```dockerfile
FROM onlyoffice/documentserver:latest

COPY ./participantes /var/www/onlyoffice/documentserver/web-apps/apps/api/documents/plugins/participantes

# Asegura permisos correctos (el proceso corre como usuario "www-data" u "onlyoffice" según la versión)
RUN chown -R www-data:www-data /var/www/onlyoffice/documentserver/web-apps/apps/api/documents/plugins/participantes
```

Construye y levanta:

```bash
docker build -t onlyoffice-with-participantes .
docker run -d -p 8092:80 --name onlyoffice-ds onlyoffice-with-participantes
```

---

## 4. Registrar el plugin (si tu Document Server requiere el listado explícito)

Algunas versiones de ONLYOFFICE requieren que el plugin esté referenciado en el archivo de configuración de plugins instalados. Verifica si existe:

```
/var/www/onlyoffice/documentserver/web-apps/apps/api/documents/plugins.json
```

Si existe, edítalo (dentro o fuera del contenedor, luego copia/monta igual que arriba) y agrega una entrada como:

```json
{
  "plugins": [
    "participantes",
    ...otros plugins existentes...
  ]
}
```

Guarda, y reinicia el contenedor:

```bash
docker restart onlyoffice-ds
```

> Si tu Document Server es **v7+**, normalmente basta con la carpeta dentro de `plugins/` y ONLYOFFICE la detecta automáticamente al reiniciar — no siempre necesitas tocar `plugins.json`.

---

## 5. Verificar que cargó correctamente

1. Abre un documento en tu editor Vue (que consume el iframe de ONLYOFFICE en el puerto 8092).
2. En la barra lateral derecha deberías ver un nuevo ícono (el de "Participantes", con las dos cabezas) debajo de los íconos nativos (¶, tabla, imagen, forma, texto, firma).
3. Haz clic → se abre el panel con el input y el botón "Mostrar".
4. Escribe un texto, haz clic en "Mostrar" (o presiona Enter) → aparece el dialog centrado con el texto.
5. Cierra el dialog con la `X`, el botón "Cerrar" o haciendo clic fuera del cuadro.

Si el ícono no aparece:

- Revisa los logs del contenedor:
  ```bash
  docker logs onlyoffice-ds --tail 100
  ```
- Verifica que la ruta dentro del contenedor sea exactamente:
  ```bash
  docker exec -it onlyoffice-ds ls /var/www/onlyoffice/documentserver/web-apps/apps/api/documents/plugins/
  ```
  y que `participantes/config.json` esté presente.
- Limpia caché del navegador o prueba en modo incógnito (ONLYOFFICE cachea agresivamente los assets de plugins).
- Verifica en la consola del navegador (F12) si hay errores 404 al pedir `.../plugins/participantes/config.json`.

---

## 6. Notas técnicas sobre el código del plugin

- **`config.json`**: define el `guid` único (cámbialo si vas a tener múltiples plugins con nombres parecidos), qué editores lo soportan (`"EditorsSupport": ["word", "cell", "slide"]`) y los íconos.
- **`plugin_code.js`**: usa `window.Asc.plugin.init` (API estándar del SDK de plugins de ONLYOFFICE) para inicializar los listeners una vez que el panel/iframe está listo.
- El dialog **no usa `window.Asc.plugin.executeMethod`** ni interactúa con el documento — es un modal 100% HTML/CSS/JS dentro del propio iframe del plugin, tal como pediste (input → botón "Mostrar" → dialog con el texto capturado).
- Si más adelante quieres que el texto se **inserte en el documento** en vez de (o además de) mostrarse en el dialog, se puede agregar con:
  ```js
  window.Asc.plugin.executeMethod("PasteText", [texto]);
  ```
  dentro de `onMostrarClick`, pero no está incluido porque el requerimiento actual es solo mostrarlo en el dialog.

---

## 7. Personalización rápida

| Quieres cambiar...                     | Dónde                                             |
|-----------------------------------------|----------------------------------------------------|
| Texto del panel / placeholder          | `index.html`                                      |
| Colores / tamaño del panel y dialog    | `resources/css/style.css`                         |
| Comportamiento (validaciones, eventos) | `plugin_code.js`                                  |
| Ícono de la barra lateral               | `resources/img/icon.png` y `icon@2x.png`          |
| Editores donde aparece (Word/Excel/PPT)| `config.json` → `EditorsSupport`                  |
