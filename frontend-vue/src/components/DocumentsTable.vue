<script setup lang="ts">
import type { Document } from '@/services/types'

const props = defineProps<{
  docs: Document[]
  loading: boolean
  activeTab: 'mine' | 'shared'
  converting: string | null
  previewing: string | null
}>()

const emit = defineEmits<{
  (e: 'switch-tab', tab: 'mine' | 'shared'): void
  (e: 'edit', doc: Document): void
  (e: 'convert', id: string): void
  (e: 'preview', id: string): void
  (e: 'delete', doc: Document): void
}>()

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function editorLabel(editor: string): string {
  if (editor === 'onlyoffice') return 'ONLYOFFICE'
  if (editor === 'collabora') return 'Collabora'
  return 'PDF'
}

function documentTypeLabel(ext: string): string {
  if (ext === 'pdf') return 'PDF'
  return ext === 'docx' || ext === 'doc' ? 'WORD' : 'EXCEL'
}

function statusLabel(status: string): string {
  return status === 'final' ? 'Convertido a PDF' : 'Borrador'
}

function tabMessage(): string {
  return props.activeTab === 'mine'
    ? 'No hay documentos. Cree uno usando el formulario de arriba.'
    : 'No hay documentos compartidos con usted.'
}
</script>

<template>
  <section class="doc-list">
    <div class="tabs">
      <button
        :class="['tab', { active: activeTab === 'mine' }]"
        @click="emit('switch-tab', 'mine')"
      >Mis documentos</button>
      <button
        :class="['tab', { active: activeTab === 'shared' }]"
        @click="emit('switch-tab', 'shared')"
      >Compartidos conmigo</button>
    </div>

    <div v-if="loading" class="state-msg">Cargando...</div>
    <div v-else-if="docs.length === 0" class="state-msg">{{ tabMessage() }}</div>

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
              {{ documentTypeLabel(doc.ext) }}
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
            <button
              v-if="doc.status !== 'final'"
              class="btn btn-edit"
              @click="emit('edit', doc)"
            >Editar</button>
            <button
              v-if="doc.editor === 'onlyoffice' || doc.editor === 'collabora' || doc.status === 'final'"
              class="btn btn-preview"
              :disabled="previewing === doc.id"
              @click="emit('preview', doc.id)"
            >
              {{ previewing === doc.id ? 'Generando...' : 'Previsualizar' }}
            </button>
            <button
              v-if="(doc.editor === 'onlyoffice' || doc.editor === 'collabora') && doc.status !== 'final'"
              class="btn btn-pdf"
              :disabled="converting === doc.id"
              @click="emit('convert', doc.id)"
            >
              {{ converting === doc.id ? 'Convirtiendo...' : 'PDF' }}
            </button>
            <button class="btn btn-delete" @click="emit('delete', doc)">
              {{ doc.shared ? 'Quitar acceso' : 'Eliminar' }}
            </button>
          </td>
        </tr>
      </tbody>
    </table>
  </section>
</template>

<style scoped>
.doc-list {
  background: white;
  border-radius: 10px;
  padding: 1.25rem;
  box-shadow: 0 1px 4px rgba(0,0,0,0.06);
}

.tabs {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.tab {
  padding: 0.55rem 0.9rem;
  border: 1px solid #d0d5dd;
  background: #f8fafc;
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.85rem;
  color: #334155;
}

.tab.active {
  background: #2563eb;
  border-color: #2563eb;
  color: white;
}

.state-msg {
  padding: 1rem 0.25rem;
  color: #64748b;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th, td {
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

.doc-name {
  font-weight: 600;
}

.badge {
  display: inline-flex;
  align-items: center;
  padding: 0.24rem 0.55rem;
  border-radius: 999px;
  font-size: 0.72rem;
  font-weight: 700;
  margin-top: 0.35rem;
}

.badge.docx,
.badge.onlyoffice {
  background: #eff6ff;
  color: #1d4ed8;
}

.badge.xlsx,
.badge.collabora {
  background: #ecfdf5;
  color: #047857;
}

.badge.pdf,
.badge.none {
  background: #fef2f2;
  color: #b91c1c;
}

.badge.final {
  background: #f0fdf4;
  color: #15803d;
}

.badge.shared-badge {
  margin-left: 0.5rem;
  background: #fef3c7;
  color: #92400e;
}

.owner-cell, .date {
  color: #334155;
}

.shared-by {
  color: #64748b;
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

.btn-pdf {
  background: #e0f2fe;
  color: #0369a1;
}

.btn-delete {
  background: #fee2e2;
  color: #b91c1c;
}

.btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

@media (max-width: 900px) {
  .doc-list {
    padding: 1rem;
    overflow-x: auto;
  }

  table {
    min-width: 860px;
  }
}
</style>
