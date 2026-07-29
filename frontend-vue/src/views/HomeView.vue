<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listDocuments, createDocument, deleteDocument, convertToPdf } from '@/services/api'
import type { Document } from '@/services/types'
import DocumentsTable from '@/components/DocumentsTable.vue'

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
const previewing = ref<string | null>(null)
const activeTab = ref<'mine' | 'shared'>('mine')
const toast = ref('')
let toastTimer: ReturnType<typeof setTimeout> | undefined

onMounted(() => loadDocs())
onBeforeUnmount(() => {
  if (toastTimer) clearTimeout(toastTimer)
})

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

async function handleDelete(doc: Document) {
  const isOwner = doc.owner_id === auth.user?.id
  const msg = isOwner
    ? `Eliminar "${doc.name}"?\nSe quitará tu acceso. Si otros usuarios tienen acceso, el documento se conservará para ellos.`
    : `Quitar acceso a "${doc.name}"?`
  if (!confirm(msg)) return
  try {
    await deleteDocument(doc.id)
    await loadDocs()
  } catch (e) {
    error.value = 'Error al eliminar: ' + String(e)
  }
}

async function handleConvert(id: string) {
  converting.value = id
  error.value = ''
  try {
    await convertToPdf(id)
    await loadDocs()
    showToast('Documento convertido a PDF correctamente')
  } catch (e) {
    error.value = 'Error al convertir: ' + String(e)
  } finally {
    converting.value = null
  }
}

function showToast(message: string) {
  toast.value = message
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => {
    toast.value = ''
  }, 4000)
}

function editUrl(doc: Document): string {
  return doc.editor === 'onlyoffice'
    ? `/editor/onlyoffice/${doc.id}`
    : `/editor/collabora/${doc.id}`
}

function previewDocument(id: string) {
  previewing.value = id
  router.push(`/preview/${id}`).finally(() => {
    previewing.value = null
  })
}
</script>

<template>
  <div class="home">
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
    <div v-if="toast" class="success-toast" role="status">{{ toast }}</div>

    <DocumentsTable
      :docs="docs"
      :loading="loading"
      :active-tab="activeTab"
      :converting="converting"
      :previewing="previewing"
      @switch-tab="switchTab"
      @edit="router.push(editUrl($event))"
      @convert="handleConvert"
      @preview="previewDocument"
      @delete="handleDelete"
    />
  </div>
</template>

<style scoped>
.home {
  max-width: 1100px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.create-form {
  background: white;
  border-radius: 10px;
  padding: 1.5rem;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.06);
}

.create-form h2 {
  font-size: 1.1rem;
  margin-bottom: 1rem;
}

.form-row {
  display: flex;
  gap: 1rem;
  align-items: flex-end;
  flex-wrap: wrap;
}

.field {
  flex: 1;
  min-width: 150px;
}

.field label {
  display: block;
  font-size: 0.8rem;
  font-weight: 600;
  color: #555;
  margin-bottom: 0.35rem;
}

.field input,
.field select {
  width: 100%;
  padding: 0.55rem 0.75rem;
  border: 1px solid #d0d5dd;
  border-radius: 6px;
  font-size: 0.9rem;
  background: white;
}

.field.action {
  flex: 0 0 auto;
}

.field.action button {
  padding: 0.65rem 1rem;
  border: none;
  border-radius: 6px;
  background: #2563eb;
  color: white;
  font-size: 0.9rem;
  cursor: pointer;
}

.field.action button:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}

.error-banner {
  padding: 0.85rem 1rem;
  background: #fef2f2;
  border: 1px solid #fecaca;
  color: #b91c1c;
  border-radius: 8px;
}

.success-toast {
  position: fixed;
  right: 1.25rem;
  bottom: 1.25rem;
  z-index: 20;
  padding: 0.85rem 1rem;
  border: 1px solid #86efac;
  border-radius: 8px;
  background: #f0fdf4;
  color: #166534;
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.14);
  font-size: 0.9rem;
}
</style>
