<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { createTemplate, deleteTemplate, listTemplates, renameTemplate } from '@/services/api'
import type { Template } from '@/services/types'
import TemplatePreviewModal from '@/components/TemplatePreviewModal.vue'

const router = useRouter()
const templates = ref<Template[]>([])
const loading = ref(true)
const error = ref('')
const toast = ref('')
const formName = ref('')
const formExt = ref<'docx' | 'xlsx'>('docx')
const creating = ref(false)
const previewId = ref<string | null>(null)
const renaming = ref<Template | null>(null)
const renameValue = ref('')
let toastTimer: ReturnType<typeof setTimeout> | undefined

onMounted(() => load())
onBeforeUnmount(() => {
  if (toastTimer) clearTimeout(toastTimer)
})

async function load() {
  loading.value = true
  error.value = ''
  try {
    templates.value = await listTemplates()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

async function handleCreate() {
  if (!formName.value.trim()) return
  creating.value = true
  error.value = ''
  try {
    await createTemplate({ name: formName.value.trim(), ext: formExt.value })
    formName.value = ''
    await load()
    showToast('Plantilla creada correctamente')
  } catch (e) {
    error.value = 'Error al crear: ' + String(e)
  } finally {
    creating.value = false
  }
}

function editTemplate(t: Template) {
  router.push(`/templates/editor/${t.id}`)
}

function previewTemplate(t: Template) {
  previewId.value = t.id
}

function startRename(t: Template) {
  renaming.value = t
  renameValue.value = t.name
}

async function handleRename() {
  if (!renaming.value || !renameValue.value.trim()) return
  try {
    await renameTemplate(renaming.value.id, renameValue.value.trim())
    renaming.value = null
    await load()
    showToast('Plantilla renombrada')
  } catch (e) {
    error.value = 'Error al renombrar: ' + String(e)
  }
}

async function handleDelete(t: Template) {
  if (!confirm(`Eliminar la plantilla "${t.name}"?`)) return
  try {
    await deleteTemplate(t.id)
    await load()
    showToast('Plantilla eliminada')
  } catch (e) {
    error.value = 'Error al eliminar: ' + String(e)
  }
}

function showToast(message: string) {
  toast.value = message
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => {
    toast.value = ''
  }, 4000)
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<template>
  <div class="templates">
    <section class="create-form">
      <h2>Nueva plantilla</h2>
      <form @submit.prevent="handleCreate">
        <div class="form-row">
          <div class="field">
            <label>Nombre</label>
            <input v-model="formName" placeholder="Nombre de la plantilla" required />
          </div>
          <div class="field">
            <label>Tipo</label>
            <select v-model="formExt">
              <option value="docx">Word (.docx)</option>
              <option value="xlsx">Excel (.xlsx)</option>
            </select>
          </div>
          <div class="field action">
            <button type="submit" :disabled="creating || !formName.trim()">
              {{ creating ? 'Creando...' : 'Crear plantilla' }}
            </button>
          </div>
        </div>
      </form>
      <p class="hint">
        Las plantillas son globales y pueden contener etiquetas. Al crear un documento desde una
        plantilla, se copia su contenido para seguir editando.
      </p>
    </section>

    <div v-if="error" class="error-banner">{{ error }}</div>
    <div v-if="toast" class="success-toast" role="status">{{ toast }}</div>

    <section class="template-list">
      <h2>Plantillas</h2>
      <div v-if="loading" class="state-msg">Cargando...</div>
      <div v-else-if="templates.length === 0" class="state-msg">No hay plantillas todavía. Cree una con el formulario.</div>
      <table v-else>
        <thead>
          <tr>
            <th>Nombre</th>
            <th>Tipo</th>
            <th>Propietario</th>
            <th>Tamaño</th>
            <th>Actualizado</th>
            <th>Acciones</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="t in templates" :key="t.id">
            <td class="tpl-name">{{ t.name }}.{{ t.ext }}</td>
            <td>
              <span class="badge" :class="t.ext">{{ t.ext === 'docx' ? 'WORD' : 'EXCEL' }}</span>
            </td>
            <td class="owner-cell">{{ t.owner_name }}</td>
            <td>{{ formatSize(t.size) }}</td>
            <td class="date">{{ new Date(t.updated_at).toLocaleString('es-BO') }}</td>
            <td class="actions">
              <button class="btn btn-edit" @click="editTemplate(t)">Editar</button>
              <button class="btn btn-preview" @click="previewTemplate(t)">Previsualizar</button>
              <button class="btn btn-rename" @click="startRename(t)">Renombrar</button>
              <button class="btn btn-delete" @click="handleDelete(t)">Eliminar</button>
            </td>
          </tr>
        </tbody>
      </table>
    </section>

    <TemplatePreviewModal
      :template-id="previewId"
      :template-name="templates.find((t) => t.id === previewId)?.name"
      @close="previewId = null"
    />

    <Teleport to="body">
      <div v-if="renaming" class="rename-overlay" @click.self="renaming = null">
        <div class="rename-modal">
          <h3>Renombrar plantilla</h3>
          <input v-model="renameValue" class="rename-input" @keyup.enter="handleRename" />
          <div class="rename-actions">
            <button class="btn btn-cancel" @click="renaming = null">Cancelar</button>
            <button class="btn btn-save" @click="handleRename">Guardar</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.templates {
  max-width: 1100px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.create-form,
.template-list {
  background: white;
  border-radius: 10px;
  padding: 1.5rem;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.06);
}

h2 {
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

.hint {
  margin-top: 0.75rem;
  font-size: 0.8rem;
  color: #64748b;
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

.state-msg {
  padding: 1rem 0.25rem;
  color: #64748b;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th,
td {
  text-align: left;
  padding: 0.85rem 0.6rem;
  border-bottom: 1px solid #edf2f7;
  vertical-align: top;
}

th {
  font-size: 0.8rem;
  color: #475569;
  background: #f8fafc;
}

.tpl-name {
  font-weight: 600;
}

.badge {
  display: inline-flex;
  align-items: center;
  padding: 0.24rem 0.55rem;
  border-radius: 999px;
  font-size: 0.72rem;
  font-weight: 700;
}

.badge.docx {
  background: #eff6ff;
  color: #1d4ed8;
}

.badge.xlsx {
  background: #ecfdf5;
  color: #047857;
}

.owner-cell,
.date {
  color: #334155;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}

.btn {
  padding: 0.38rem 0.72rem;
  border: 1px solid transparent;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.8rem;
}

.btn-preview {
  background: #475569;
  color: white;
}

.btn-edit {
  background: #dbeafe;
  color: #1d4ed8;
}

.btn-rename {
  background: #f1f5f9;
  color: #475569;
}

.btn-delete {
  background: #fee2e2;
  color: #b91c1c;
}

.rename-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.rename-modal {
  background: white;
  border-radius: 10px;
  padding: 1.25rem;
  width: min(380px, 90vw);
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.25);
}

.rename-modal h3 {
  font-size: 1rem;
}

.rename-input {
  padding: 0.55rem 0.75rem;
  border: 1px solid #d0d5dd;
  border-radius: 6px;
  font-size: 0.9rem;
  background: white;
}

.rename-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}

.btn-cancel {
  background: #f1f5f9;
  color: #475569;
}

.btn-save {
  background: #2563eb;
  color: white;
}

[data-theme='dark'] .create-form,
[data-theme='dark'] .template-list,
[data-theme='dark'] .rename-modal {
  background: var(--app-surface);
  border-color: var(--app-border);
}

[data-theme='dark'] .field input,
[data-theme='dark'] .field select,
[data-theme='dark'] .rename-input {
  background: var(--app-surface-muted);
  color: var(--app-text);
  border-color: var(--app-border);
}

[data-theme='dark'] th {
  background: var(--app-surface-muted);
  color: var(--app-text);
}

[data-theme='dark'] td {
  border-color: var(--app-border-soft);
}

[data-theme='dark'] .owner-cell,
[data-theme='dark'] .date,
[data-theme='dark'] .state-msg,
[data-theme='dark'] .hint {
  color: var(--app-text-muted);
}
</style>
