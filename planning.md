# Editor Online - Estado Actual

## Resumen

Proyecto de editor colaborativo con:

- Frontend en Vue 3
- Backend en Rust con Axum
- Persistencia local en SQLite
- Edición con ONLYOFFICE y Collabora CODE
- Acceso por `localhost` y por IP de intranet

## Puertos

| Servicio | Puerto | Exposición |
|----------|--------|------------|
| Frontend Vue | 8090 | `http://<IP>:8090` |
| Backend Rust | 8091 | `http://<IP>:8091` |
| ONLYOFFICE Docs | 8092 | Docker |
| Collabora CODE | 8093 | Docker |

Ejemplo de intranet actual:

- Frontend: `http://172.27.38.53:8090`
- Backend: `http://172.27.38.53:8091`
- ONLYOFFICE: `http://172.27.38.53:8092`
- Collabora: `http://172.27.38.53:8093`

## Arquitectura

```text
Navegador
  |
  +--> Vue frontend :8090
          |
          +--> Rust backend :8091
                   |
                   +--> SQLite en ./data/editor.db
                   +--> Archivos binarios ./data/{id}.bin
                   +--> PDFs ./data/{id}.pdf
                   +--> ONLYOFFICE Docs :8092
                   +--> Collabora CODE :8093
```

## Backend Rust

### Responsabilidades

- Autenticación de usuarios
- CRUD de documentos
- Creación de documentos en blanco
- Persistencia del contenido local
- Conversión a PDF
- Integración con ONLYOFFICE
- Integración WOPI con Collabora

### Base de datos

- SQLite en `data/editor.db`
- Tabla `documents`:
  - `id`
  - `name`
  - `ext`
  - `mime`
  - `editor`
  - `size`
  - `status`
  - `created_at`
  - `updated_at`
- Tabla `users`:
  - `id`
  - `username`
  - `password`
  - `name`
  - `created_at`

### Archivos locales

- Documento original: `data/{id}.bin`
- PDF convertido: `data/{id}.pdf`

### Cambios implementados

- Se eliminó el auto-seed de documentos al iniciar
- `POST /api/documents` crea archivos realmente en blanco
- Collabora usa WOPI con:
  - `CheckFileInfo`
  - `GetFile`
  - `PutFile`
  - `LOCK`
  - `UNLOCK`
  - `REFRESH_LOCK`
  - `GET_LOCK`
- Se corrigieron URLs para usar el host/IP de la intranet
- OnlyOffice usa una `key` estable basada en:
  - `documentId + hash del contenido`
- Collabora recibe `frame-ancestors` compatibles con la intranet

### Endpoints

| Método | Ruta | Propósito |
|--------|------|-----------|
| POST | `/api/auth/login` | Iniciar sesión |
| GET | `/api/documents` | Listar documentos |
| POST | `/api/documents` | Crear documento |
| GET | `/api/documents/:id` | Obtener metadata |
| DELETE | `/api/documents/:id` | Eliminar documento |
| POST | `/api/documents/:id/convert` | Convertir a PDF |
| GET | `/api/documents/:id/pdf` | Descargar PDF |
| GET | `/api/documents/:id/content` | Descargar binario |
| GET | `/api/collabora/session/:id` | Obtener iframe de Collabora |
| GET | `/api/onlyoffice/config/:id` | Obtener config de OnlyOffice |
| GET | `/wopi/files/:id` | WOPI CheckFileInfo |
| POST | `/wopi/files/:id` | WOPI LOCK / UNLOCK / REFRESH_LOCK / GET_LOCK |
| GET | `/wopi/files/:id/contents` | WOPI GetFile |
| POST | `/wopi/files/:id/contents` | WOPI PutFile |
| GET | `/download/:id` | Descarga para OnlyOffice |
| POST | `/callback/onlyoffice/:id` | Callback de OnlyOffice |

## Frontend Vue

### Rutas

- `/` - Home
- `/login` - Login
- `/editor/collabora/:id` - Editor Collabora
- `/editor/onlyoffice/:id` - Editor OnlyOffice

### Cambios implementados

- El frontend construye la API usando el host actual del navegador
- El script de ONLYOFFICE se carga desde el mismo host de intranet
- El iframe de Collabora permite la colaboración dentro del sandbox necesario

### Funcionalidades

- Login con usuarios de prueba
- Lista de documentos
- Crear documentos Word o Excel
- Abrir documento en Collabora o ONLYOFFICE
- Eliminar documento
- Convertir a PDF

## ONLYOFFICE

- Se usa para edición en línea
- La config se genera desde el backend
- La `key` del documento cambia solo si cambia el contenido
- El callback persiste el archivo en `data/{id}.bin`

## Collabora CODE

- Se usa por WOPI
- El iframe se construye con la IP/host actual
- La política de `frame-ancestors` fue ajustada para intranet
- El backend persiste el archivo local cuando recibe `PutFile`

## Plantillas de documentos

- DOCX en blanco: ZIP Office Open XML mínimo
- XLSX en blanco: ZIP Office Open XML mínimo
- PDF: generación nativa desde Rust

## Flujo de uso

1. Iniciar frontend
2. Iniciar backend
3. Levantar Collabora y ONLYOFFICE con Docker
4. Iniciar sesión
5. Crear un documento nuevo
6. Abrirlo en el editor correspondiente
7. Guardar cambios
8. Verificar persistencia en SQLite y en `data/`

## Comandos principales

```bash
docker compose up -d

cd backend-rust
DATA_DIR=../data cargo run --release

cd frontend-vue
yarn dev --host
```

## Nota de intranet

Si se usa otra IP de LAN, hay que ajustar:

- `docker-compose.yml`
- `PUBLIC_COLLABORA_URL` si se usa variable fija
- `BACKEND_URL` si se despliega detrás de un proxy
- la URL de acceso del frontend y backend
