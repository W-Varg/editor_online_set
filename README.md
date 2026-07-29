# Editor Online

Aplicacion de edicion colaborativa con:

- Frontend Vue 3
- Backend Rust/Axum
- SQLite local
- ONLYOFFICE Docs
- Collabora CODE

## Arquitectura Y Responsabilidades

El proyecto sigue una arquitectura cliente-servidor con editores externos integrados:

- `Frontend Vue`: capa de presentacion. Maneja login, listado de documentos, formularios y la apertura de editores.
- `Backend Rust`: API principal. Autentica usuarios, crea documentos, sirve metadatos, persiste contenido, coordina WOPI y genera las URL que consumen los editores.
- `SQLite`: capa de persistencia local. Guarda usuarios, documentos y estados del sistema.
- `ONLYOFFICE Docs`: editor colaborativo para documentos Office. Consume la configuracion que entrega el backend y devuelve cambios al callback.
- `Collabora CODE`: editor colaborativo basado en WOPI. Abre el documento mediante `WOPISrc`, bloquea el archivo cuando es necesario y persiste los cambios en el backend.

### Flujo De Arquitectura

```text
Navegador
  -> Frontend Vue
    -> Backend Rust
      -> SQLite / archivos locales
      -> ONLYOFFICE Docs
      -> Collabora CODE
```

### Que Hace Cada Componente

- `Frontend Vue`
  - muestra la interfaz
  - ejecuta el login
  - lista documentos
  - abre los editores
  - dispara crear, borrar y convertir a PDF
- `Backend Rust`
  - valida JWT
  - expone la API REST
  - crea documentos en blanco
  - persiste contenido en disco
  - atiende el contrato WOPI de Collabora
  - genera la configuracion de ONLYOFFICE
- `SQLite`
  - almacena usuarios y documentos
  - conserva fechas, estado, tamaño y editor asociado
- `ONLYOFFICE Docs`
  - permite edicion colaborativa en tiempo real
  - usa `document.key` para identificar la version del archivo
  - notifica al backend cuando hay guardado
- `Collabora CODE`
  - permite edicion colaborativa via WOPI
  - consulta `CheckFileInfo`
  - obtiene el archivo con `GetFile`
  - guarda con `PutFile`
  - usa bloqueos para evitar conflictos

### Que Es WOPI

WOPI significa `Web Application Open Platform Interface`. Es un protocolo que permite que un editor web, como Collabora, abra y guarde archivos que vive en otro servidor.

En este proyecto, el backend Rust actua como servidor WOPI y le entrega a Collabora:

- la informacion del archivo con `CheckFileInfo`
- el contenido del archivo con `GetFile`
- los cambios guardados con `PutFile`
- el control de bloqueo con `LOCK`, `UNLOCK` y `REFRESH_LOCK`

Gracias a WOPI, Collabora puede editar documentos que en realidad se guardan en la base local y en el directorio `data/` del proyecto.

## Requisitos previos

Instala esto antes de arrancar el proyecto:

- `git`
- `Docker` y `Docker Compose`
- `Rust` con `cargo`
- `Node.js` 18 o superior
- `yarn`

Opcional, pero recomendado:

- `curl`
- `jq`
- un navegador Chromium, Brave o Firefox

## Clonar el repositorio

```bash
git clone <URL-del-repositorio>
cd editor-online
```

## Estructura

```text
editor-online/
  backend-rust/
  frontend-vue/
  data/
  docker-compose.yml
  planning.md
  README.md
```

## Puertos

| Servicio | Puerto |
|----------|--------|
| Frontend Vue | 8090 |
| Backend Rust | 8091 |
| ONLYOFFICE Docs | 8092 |
| Collabora CODE | 8093 |

## Levantar los servicios

### 1. Levantar los documentos

```bash
docker compose up -d
```

Esto inicia:

- `collabora/code:latest`
- `onlyoffice/documentserver:latest`

Antes de levantar Collabora, define el `extra_params` completo para el CSP y los orígenes permitidos. Por ejemplo:

```bash
export JWT_SECRET='my-secret-key'
export PUBLIC_BACKEND_URL='http://host.docker.internal:8091'
export COLLABORA_EXTRA_PARAMS="--o:ssl.enable=false --o:net.content_security_policy=default-src 'self'; frame-ancestors http://localhost:* http://127.0.0.1:* http://172.27.38.53:* http://host.docker.internal:*; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-eval'; connect-src 'self' ws: wss:; img-src 'self' data: blob:; font-src 'self' data:; object-src 'none'; base-uri 'self';"
```

Si prefieres usar un archivo `.env`, copia primero el perfil que te sirva:

```bash
cp .env.local.example .env
```

Si vas a abrir el proyecto desde otra máquina en la red, usa el perfil de intranet:

```bash
cp .env.intranet.example .env
```

El backend y el compose leerán ese `.env` automáticamente.

### 2. Levantar el backend

Desde la carpeta raíz del proyecto:

```bash
cd backend-rust
DATA_DIR=../data cargo run --release
```

Si quieres usar otra IP o un proxy, puedes ajustar variables como:

- `PORT`
- `DATA_DIR`
- `BACKEND_URL`
- `PUBLIC_BACKEND_URL`
- `COLLABORA_URL`
- `PUBLIC_COLLABORA_URL`
- `JWT_SECRET`

### 3. Levantar el frontend

```bash
cd frontend-vue
yarn install
yarn dev --host
```

Si necesitas fijar la URL pública de ONLYOFFICE en el frontend, crea `frontend-vue/.env` o usa `frontend-vue/.env.example` con `VITE_ONLYOFFICE_URL`.

Con `--host`, Vite expone la app en la red local.

## URLs de acceso

Si tu IP de intranet es `172.27.38.53`, usa:

- Frontend: `http://172.27.38.53:8090`
- Backend: `http://172.27.38.53:8091`
- ONLYOFFICE: `http://172.27.38.53:8092`
- Collabora: `http://172.27.38.53:8093`

## Flujo de uso

1. Abre el frontend.
2. Inicia sesión con un usuario de prueba:
   - `user1`
   - `user2`
   - `user3`
   - `user4`
   - `user5`
3. Crea un documento nuevo:
   - `docx` para Word
   - `xlsx` para Excel
4. Abre el documento con:
   - ONLYOFFICE
   - Collabora
5. Edita y guarda.
6. Los cambios se persisten en SQLite y en `data/`.

## Usuarios de prueba

Contraseña común:

```text
Admin123@
```

Usuarios disponibles:

- `user1`
- `user2`
- `user3`
- `user4`
- `user5`

## Crear un documento con `curl`

Primero inicia sesión:

```bash
TOKEN=$(curl -sS -X POST http://172.27.38.53:8091/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"user1","password":"Admin123@"}' | jq -r .token)
```

Luego crea un documento en blanco:

```bash
curl -sS -X POST http://172.27.38.53:8091/api/documents \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  --data-raw '{"name":"mi-documento","ext":"docx","editor":"collabora"}'
```

Para Excel:

```bash
curl -sS -X POST http://172.27.38.53:8091/api/documents \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  --data-raw '{"name":"mi-planilla","ext":"xlsx","editor":"onlyoffice"}'
```

## Verificar persistencia

Los archivos locales quedan en:

- `data/editor.db`
- `data/{id}.bin`
- `data/{id}.pdf`

## Acceso desde intranet

Si vas a abrir la app desde otra maquina de la red, usa siempre la IP real del host:

- `http://172.27.38.53:8090`

No uses `localhost` desde otra computadora, porque `localhost` apunta a la propia maquina cliente.

## Reiniciar servicios

### Docker

```bash
docker compose down
docker compose up -d
```

### Backend Rust

```bash
cd backend-rust
DATA_DIR=../data cargo run --release
```

### Frontend Vue

```bash
cd frontend-vue
yarn dev --host
```

## Notas

- Para desarrollo local, el frontend debe abrirse con `--host`.
- Collabora requiere que `frame-ancestors` acepte la IP de la intranet.
- El backend ya responde con URLs absolutas usando el host real de acceso.
- La colaboracion en Collabora usa WOPI y persiste el archivo localmente.
