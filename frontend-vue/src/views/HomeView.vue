<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listDocuments, createDocument, deleteDocument, convertToPdf } from '@/services/api'
import type { Document } from '@/services/types'

const router = useRouter()
const auth = useAuthStore()

const docs = ref<Document[]>([])
const loading = ref(true)
const error = ref('')
const formName = ref('')
const formExt = ref<'docx' | 'xlsx'>('docx')
const formEditor = ref<'onlyoffice' | 'collabora'>('onlyoffice')
const creating = ref(false)
const converting = ref<string | null>(null)
const activeTab = ref<'mine' | 'shared'>('mine')

onMounted(() => loadDocs())

async function loadDocs() {
  loading.value = true
  error.value = ''
  try {
    docs.value = await listDocuments(activeTab.value)
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function switchTab(tab: 'mine' | 'shared') {
  activeTab.value = tab
  loadDocs()
}

async function handleCreate() {
  if (!formName.value.trim()) return
  creating.value = true
  error.value = ''
  try {
    await createDocument({
      name: formName.value.trim(),
      ext: formExt.value,
      editor: formEditor.value,
    })
    formName.value = ''
    if (activeTab.value === 'shared') switchTab('mine')
    else await loadDocs()
  } catch (e) {
    error.value = 'Error al crear: ' + String(e)
  } finally {
    creating.value = false
  }
}

async function handleDelete(id: string, name: string) {
  const isOwner = docs.value.find(d => d.id === id)?.owner_id === auth.user?.id
  const msg = isOwner
    ? `Eliminar "${name}"?\nSe quitará tu acceso. Si otros usuarios tienen acceso, el documento se conservará para ellos.`
    : `Quitar acceso a "${name}"?`
  if (!confirm(msg)) return
  try {
    await deleteDocument(id)
    await loadDocs()
  } catch (e) {
    error.value = 'Error al eliminar: ' + String(e)
  }
}

async function handleConvert(id: string) {
  converting.value = id
  error.value = ''
  try {
    const result = await convertToPdf(id)
    if (result.pdf_url) {
      window.open(result.pdf_url, '_blank')
    } else {
      error.value = 'Error en la conversión'
    }
    await loadDocs()
  } catch (e) {
    error.value = 'Error al convertir: ' + String(e)
  } finally {
    converting.value = null
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

function editorLabel(e: string): string {
  return e === 'onlyoffice' ? 'ONLYOFFICE' : 'Collabora'
}

function statusLabel(s: string): string {
  return s === 'final' ? 'Convertido a PDF' : 'Borrador'
}

function editUrl(doc: Document): string {
  return doc.editor === 'onlyoffice'
    ? `/editor/onlyoffice/${doc.id}`
    : `/editor/collabora/${doc.id}`
}
</script>

<template>
  <div class="home">
    <header>
      <h1>Editor Online</h1>
      <p class="subtitle">Gestión de documentos colaborativos</p>
    </header>

    <section class="create-form">
      <h2>Nuevo documento</h2>
      <form @submit.prevent="handleCreate">
        <div class="form-row">
          <div class="field">
            <label>Nombre</label>
            <input v-model="formName" placeholder="Nombre del documento" required />
          </div>
          <div class="field">
            <label>Tipo</label>
            <select v-model="formExt">
              <option value="docx">Word (.docx)</option>
              <option value="xlsx">Excel (.xlsx)</option>
            </select>
          </div>
          <div class="field">
            <label>Editor</label>
            <select v-model="formEditor">
              <option value="onlyoffice">ONLYOFFICE</option>
              <option value="collabora">Collabora Online</option>
            </select>
          </div>
          <div class="field action">
            <button type="submit" :disabled="creating || !formName.trim()">
              {{ creating ? 'Creando...' : 'Crear documento' }}
            </button>
          </div>
        </div>
      </form>
    </section>

    <div v-if="error" class="error-banner">{{ error }}</div>

    <section class="doc-list">
      <div class="tabs">
        <button
          :class="['tab', { active: activeTab === 'mine' }]"
          @click="switchTab('mine')"
        >Mis documentos</button>
        <button
          :class="['tab', { active: activeTab === 'shared' }]"
          @click="switchTab('shared')"
        >Compartidos conmigo</button>
      </div>

      <div v-if="loading" class="state-msg">Cargando...</div>
      <div v-else-if="docs.length === 0" class="state-msg">
        {{ activeTab === 'mine' ? 'No hay documentos. Cree uno usando el formulario de arriba.' : 'No hay documentos compartidos con usted.' }}
      </div>

      <table v-else>
        <thead>
          <tr>
            <th>Nombre</th>
            <th>Tipo</th>
            <th>Editor</th>
            <th>Propietario</th>
            <th>Tamaño</th>
            <th>Estado</th>
            <th>Actualizado</th>
            <th>Acciones</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="doc in docs" :key="doc.id">
            <td class="doc-name">
              {{ doc.name }}.{{ doc.ext }}
              <span v-if="doc.shared" class="badge shared-badge">Compartido</span>
            </td>
            <td>
              <span class="badge" :class="doc.ext">
                {{ doc.ext === 'docx' ? 'WORD' : 'EXCEL' }}
              </span>
            </td>
            <td>
              <span class="badge" :class="doc.editor">
                {{ editorLabel(doc.editor) }}
              </span>
            </td>
            <td class="owner-cell">
              {{ doc.owner_name }}
              <span v-if="doc.shared_by_name" class="shared-by">
                <br><small>por {{ doc.shared_by_name }}</small>
              </span>
            </td>
            <td>{{ formatSize(doc.size) }}</td>
            <td>
              <span class="badge" :class="doc.status">
                {{ statusLabel(doc.status) }}
              </span>
            </td>
            <td class="date">{{ new Date(doc.updated_at).toLocaleString('es-BO') }}</td>
            <td class="actions">
              <button class="btn btn-edit" @click="router.push(editUrl(doc))">Editar</button>
              <button
                class="btn btn-pdf"
                :disabled="converting === doc.id"
                @click="handleConvert(doc.id)"
              >
                {{ converting === doc.id ? 'Convirtiendo...' : 'PDF' }}
              </button>
              <button class="btn btn-delete" @click="handleDelete(doc.id, doc.name)">
                {{ doc.shared ? 'Quitar acceso' : 'Eliminar' }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </section>
  </div>
</template>

<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: system-ui, -apple-system, sans-serif; background: #f5f7fa; color: #1a1a2e; }
</style>

<style scoped>
.home {
  max-width: 1100px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}
header { margin-bottom: 2rem; }
h1 { font-size: 1.75rem; margin-bottom: 0.25rem; }
.subtitle { color: #666; font-size: 0.95rem; }

.create-form {
  background: white;
  border-radius: 10px;
  padding: 1.5rem;
  margin-bottom: 2rem;
  box-shadow: 0 1px 4px rgba(0,0,0,0.06);
}
.create-form h2 { font-size: 1.1rem; margin-bottom: 1rem; }
.form-row { display: flex; gap: 1rem; align-items: flex-end; flex-wrap: wrap; }
.field { flex: 1; min-width: 150px; }
.field label { display: block; font-size: 0.8rem; font-weight: 600; color: #555; margin-bottom: 0.35rem; }
.field input, .field select {
  width: 100%; padding: 0.55rem 0.75rem;
  border: 1px solid #d0d5dd; border-radius: 6px;
  font-size: 0.9rem;
}
.field.action { flex: 0 0 auto; min-width: auto; }
.field.action button {
  padding: 0.55rem 1.25rem;
  background: #2563eb; color: white;
  border: none; border-radius: 6px;
  font-weight: 600; font-size: 0.9rem;
  cursor: pointer; white-space: nowrap;
}
.field.action button:disabled { opacity: 0.5; cursor: not-allowed; }

.error-banner {
  background: #fef2f2; color: #dc2626;
  padding: 0.75rem 1rem; border-radius: 6px;
  margin-bottom: 1rem; font-size: 0.9rem;
}

.doc-list { }
.state-msg { padding: 2rem; text-align: center; color: #888; }

/* Tabs */
.tabs {
  display: flex; gap: 0;
  margin-bottom: 1rem;
  background: white;
  border-radius: 10px 10px 0 0;
  overflow: hidden;
  box-shadow: 0 1px 4px rgba(0,0,0,0.06);
}
.tab {
  flex: 1;
  padding: 0.75rem 1rem;
  border: none;
  background: #f8fafc;
  font-size: 0.9rem;
  font-weight: 600;
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s;
  border-bottom: 2px solid transparent;
}
.tab:hover { background: #f1f5f9; }
.tab.active {
  background: white;
  color: #2563eb;
  border-bottom-color: #2563eb;
}

table {
  width: 100%; border-collapse: collapse;
  background: white; border-radius: 0 0 10px 10px; overflow: hidden;
  box-shadow: 0 1px 4px rgba(0,0,0,0.06);
}
th, td { padding: 0.7rem 0.85rem; text-align: left; font-size: 0.875rem; }
th { background: #f8fafc; font-weight: 600; color: #475569; border-bottom: 2px solid #e2e8f0; }
td { border-bottom: 1px solid #f1f5f9; }
tr:last-child td { border-bottom: none; }
.doc-name { font-weight: 500; }
.owner-cell { font-size: 0.85rem; }
.shared-by { color: #888; }
.date { font-size: 0.8rem; color: #666; white-space: nowrap; }

.badge {
  display: inline-block; padding: 0.15rem 0.55rem;
  border-radius: 4px; font-size: 0.75rem;
  font-weight: 600; text-transform: uppercase;
}
.badge.docx { background: #dbeafe; color: #1e40af; }
.badge.xlsx { background: #dcfce7; color: #166534; }
.badge.onlyoffice { background: #e6f7ec; color: #1d7a3c; }
.badge.collabora { background: #fff7ed; color: #9a3412; }
.badge.draft { background: #f1f5f9; color: #475569; }
.badge.final { background: #fef3c7; color: #92400e; }
.shared-badge { background: #f3e8ff; color: #7c3aed; margin-left: 0.5rem; font-size: 0.65rem; }

.actions { display: flex; gap: 0.4rem; flex-wrap: wrap; }
.btn {
  padding: 0.35rem 0.7rem; border-radius: 5px;
  font-size: 0.78rem; font-weight: 500;
  border: none; cursor: pointer; text-decoration: none;
  transition: opacity 0.15s;
}
.btn:hover { opacity: 0.8; }
.btn:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-edit { background: #2563eb; color: white; }
.btn-pdf { background: #0891b2; color: white; }
.btn-delete { background: #ef4444; color: white; }
</style>
