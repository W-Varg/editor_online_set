# Plugin Sharing - Plan de Implementación

## Estado Actual

Backend Rust plano (4 archivos), frontend Vue básico con ONLYOFFICE y Collabora.

## 1. Reestructuración del Backend Rust

### Estructura objetivo

```
src/
├── main.rs                    # Router + startup
├── config.rs                  # Env vars tipadas
├── models/                    # Domain models
│   ├── mod.rs
│   ├── document.rs
│   └── user.rs
├── dto/                       # Request/Response
│   ├── mod.rs
│   ├── auth.rs
│   ├── document.rs
│   ├── collabora.rs
│   ├── onlyoffice.rs
│   └── sharing.rs
├── db/                        # DB connection + migrations
│   ├── mod.rs
│   └── migrations.rs
├── repos/                     # Data access
│   ├── mod.rs
│   ├── user_repo.rs
│   └── document_repo.rs
├── services/                  # Business logic
│   ├── mod.rs
│   ├── auth_service.rs
│   ├── document_service.rs
│   ├── collabora_service.rs
│   ├── onlyoffice_service.rs
│   └── sharing_service.rs
├── controllers/               # Axum handlers
│   ├── mod.rs
│   ├── auth_controller.rs
│   ├── document_controller.rs
│   ├── collabora_controller.rs
│   ├── onlyoffice_controller.rs
│   └── sharing_controller.rs
├── helpers/                   # Cross-cutting
│   ├── mod.rs
│   ├── jwt.rs
│   ├── url.rs
│   └── wopi.rs
└── templates.rs               # DOCX/XLSX/PDF
```

## 2. Base de Datos

### Schema actual → nuevo

```sql
-- Usuarios extendidos
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    name TEXT NOT NULL,
    dni TEXT UNIQUE,
    cargo TEXT,
    active INTEGER DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Documentos con owner
CREATE TABLE documents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    ext TEXT NOT NULL,
    mime TEXT NOT NULL,
    editor TEXT NOT NULL,
    size INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'draft',
    owner_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (owner_id) REFERENCES users(id)
);

-- Compartidos
CREATE TABLE document_shares (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    shared_by TEXT NOT NULL,
    permission TEXT NOT NULL DEFAULT 'edit',
    created_at TEXT NOT NULL,
    FOREIGN KEY (document_id) REFERENCES documents(id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (shared_by) REFERENCES users(id),
    UNIQUE(document_id, user_id)
);
```

### Reglas de sharing

| Acción | Comportamiento |
|--------|---------------|
| Crear documento | `owner_id = user_id`, dueño tiene acceso implícito |
| Compartir con B | INSERT en document_shares |
| B ve documentos | Propios (owner_id = B) + compartidos (document_shares.user_id = B) |
| B "elimina" | DELETE document_shares WHERE user_id = B AND document_id = X |
| A (owner) "elimina" | DELETE document_shares + si no quedan shares, DELETE documento |

El DELETE siempre es "quitarme el acceso". El archivo se borra solo cuando NADIE tiene acceso.

## 3. API - Endpoints

### Nuevos
```
GET    /api/users/search?q={query}     → Buscar usuarios (autenticado)
POST   /api/documents/:id/share        → { user_id }
DELETE /api/documents/:id/share/:userId → Revocar
GET    /api/documents/:id/shares       → Lista de usuarios con acceso
```

### Modificados
```
GET    /api/documents                  → owner_id = user_id OR id IN (shares)
DELETE /api/documents/:id              → Remove own access, cleanup if last
```

### Response de documento extendido
```json
{
  "id": "...",
  "name": "...",
  "ext": "docx",
  "mime": "...",
  "editor": "onlyoffice",
  "size": 1337,
  "status": "draft",
  "created_at": "...",
  "updated_at": "...",
  "owner_id": "...",
  "owner_name": "User 1",
  "shared": true,
  "shared_by": "User 2",
  "shared_by_name": "User 2"
}
```

## 4. Plugin ONLYOFFICE - Sidebar Compartir

### Arquitectura

El plugin se sirve como archivo estático desde el backend Rust.
ONLYOFFICE lo carga via `config.editorConfig.plugins`.

### Archivos del plugin (en backend-rust/public/plugins/share/)

```
public/plugins/share/
├── plugin.xml       → Config (botón en toolbar, sidebar)
├── plugin.js        → Lógica JS
└── style.css        → Estilos del sidebar
```

### plugin.xml
```xml
<?xml version="1.0" encoding="UTF-8"?>
<plugin id="restringida-share" name="Compartir" version="1.0">
    <description>Compartir documento con otros usuarios</description>
    <icon>data:image/svg+xml;base64,...</icon>
    <methods>
        <method>onDocumentReady</method>
    </methods>
    <buttons>
        <button>Compartir</button>
    </buttons>
</plugin>
```

### Config que se inyecta en el EditorConfig
```json
{
  "plugins": {
    "autostart": false,
    "plugins": [
      {
        "id": "restringida-share",
        "src": "{backendUrl}/plugins/share/plugin.js"
      }
    ]
  },
  "customization": {
    "pluginsData": [
      "DOCUMENT_ID",
      "ACCESS_TOKEN",
      "BACKEND_URL"
    ]
  }
}
```

### Flujo del plugin
1. Se inicia el editor ONLYOFFICE con configuración que incluye el plugin
2. El plugin registra un botón en la toolbar
3. Usuario hace click → se abre sidebar
4. Sidebar tiene campo de búsqueda + resultados + lista de compartidos
5. Usuario busca por nombre/DNI → GET /api/users/search?q=
6. Selecciona usuario → se agrega a lista temporal
7. Click "Compartir" → POST /api/documents/:id/share
8. Se recarga la lista de compartidos → GET /api/documents/:id/shares
9. Puede revocar acceso → DELETE /api/documents/:id/share/:userId

## 5. Plugin Collabora

Collabora NO tiene sistema de plugins. Alternativa:

- Botón "Compartir" en la toolbar de la app (FUERA del iframe de Collabora)
- Modal/sidebar que aparece dentro de la página del editor Collabora
- Usa el mismo backend que el plugin de ONLYOFFICE

En EditorCollabora.vue:
- Agregar botón "Compartir" en la toolbar (al lado del botón Volver)
- Modal con búsqueda de usuarios y lista de compartidos
- Llamadas directas a la API

## 6. Frontend - Vistas

### App.vue
- Navbar con tabs: "Mis Documentos" | "Compartidos conmigo"

### HomeView.vue
- Tab "Míos": documentos donde owner_id = user_id
- Tab "Compartidos": documentos de document_shares
- Cada doc muestra badge "Compartido" si tiene shares
- Al hacer click en "Editar" → navega al editor correspondiente

### Tipos nuevos
```typescript
interface DocumentExtended extends Document {
  owner_id: string
  owner_name: string
  shared: boolean
  shared_by?: string
  shared_by_name?: string
}

interface ShareInfo {
  id: string
  user_id: string
  user_name: string
  shared_by: string
  shared_by_name: string
  permission: string
  created_at: string
}
```

## 7. Orden de Implementación

1. planning_plugin.md ✅ (este archivo)
2. Reestructurar backend Rust (modulos)
3. Migrar DB y seed
4. Implementar repos + services
5. Implementar controllers + endpoints sharing
6. Frontend: types + api + tabs + compartidos
7. Plugin ONLYOFFICE
8. Botón compartir Collabora
9. Pruebas integrales
