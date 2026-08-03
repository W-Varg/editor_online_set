<script setup lang="ts">
import { ref, watch } from 'vue'
import { listTags } from '@/services/api'
import type { Tag } from '@/services/types'

const props = defineProps<{
  show: boolean
  title?: string
}>()

const emit = defineEmits<{ (e: 'close'): void; (e: 'select', key: string): void }>()

const tags = ref<Tag[]>([])
const loading = ref(false)
const error = ref('')
const copiedKey = ref('')
const tagSample = '{{key}}'
let copyTimer: ReturnType<typeof setTimeout> | undefined

function tagDisplay(tag: Tag): string {
  return `{{${tag.key}}}`
}

watch(
  () => props.show,
  async (visible) => {
    if (!visible) return
    loading.value = true
    error.value = ''
    copiedKey.value = ''
    try {
      tags.value = await listTags()
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      loading.value = false
    }
  },
)

function close() {
  if (copyTimer) clearTimeout(copyTimer)
  emit('close')
}

function insert(key: string) {
  emit('select', key)
  close()
}

async function copy(key: string) {
  try {
    await navigator.clipboard.writeText(`{{${key}}}`)
    copiedKey.value = key
    if (copyTimer) clearTimeout(copyTimer)
    copyTimer = setTimeout(() => {
      copiedKey.value = ''
    }, 2000)
  } catch {
    error.value = 'No se pudo copiar al portapapeles'
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="tags-overlay" @click.self="close">
      <div class="tags-modal">
        <div class="modal-header">
          <span class="modal-title">{{ title || 'Insertar etiqueta' }}</span>
          <button class="btn-close" type="button" @click="close">×</button>
        </div>
        <div class="modal-body">
          <p class="hint">
            Las etiquetas se insertan como <code>{{ tagSample }}</code> en el cursor y el servidor las
            reemplaza por los datos del usuario al previsualizar.
          </p>
          <div v-if="loading" class="modal-status">Cargando etiquetas...</div>
          <div v-else-if="error" class="modal-status error">{{ error }}</div>
          <ul v-else class="tags-list">
            <li v-for="tag in tags" :key="tag.key" class="tag-item">
              <div class="tag-info">
                <strong>{{ tag.label }}</strong>
                <code class="tag-key">{{ tagDisplay(tag) }}</code>
                <small>{{ tag.description }}</small>
              </div>
              <div class="tag-actions">
                <button class="btn-insert" type="button" @click="insert(tag.key)">
                  {{ copiedKey === tag.key ? 'Copiado' : 'Insertar' }}
                </button>
                <button class="btn-copy" type="button" @click="copy(tag.key)">
                  {{ copiedKey === tag.key ? '✓' : 'Copiar' }}
                </button>
              </div>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.tags-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  padding: 1.5rem;
}

.tags-modal {
  background: white;
  border-radius: 10px;
  width: min(520px, 100%);
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.3);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.7rem 1rem;
  border-bottom: 1px solid #e2e8f0;
  background: #f8fafc;
  flex-shrink: 0;
}

.modal-title {
  font-size: 0.95rem;
  font-weight: 600;
  color: #334155;
}

.btn-close {
  border: none;
  background: transparent;
  font-size: 1.4rem;
  line-height: 1;
  cursor: pointer;
  color: #475569;
}

.modal-body {
  padding: 1rem 1.25rem;
  overflow-y: auto;
}

.hint {
  font-size: 0.8rem;
  color: #64748b;
  margin: 0 0 0.85rem;
}

.hint code {
  background: #f1f5f9;
  padding: 0.05rem 0.3rem;
  border-radius: 4px;
  font-size: 0.75rem;
}

.modal-status {
  padding: 1.5rem 0;
  text-align: center;
  color: #64748b;
}

.modal-status.error {
  color: #b91c1c;
}

.tags-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.tag-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.7rem 0.85rem;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
}

.tag-info {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  min-width: 0;
}

.tag-info strong {
  font-size: 0.9rem;
  color: #0f172a;
}

.tag-key {
  font-size: 0.78rem;
  color: #7c3aed;
  background: #f5f3ff;
  padding: 0.05rem 0.35rem;
  border-radius: 4px;
  align-self: flex-start;
}

.tag-info small {
  color: #64748b;
  font-size: 0.75rem;
}

.tag-actions {
  display: flex;
  gap: 0.4rem;
  flex-shrink: 0;
}

.btn-insert {
  padding: 0.4rem 0.7rem;
  background: #2563eb;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 0.8rem;
  cursor: pointer;
}

.btn-copy {
  padding: 0.4rem 0.7rem;
  background: #f1f5f9;
  color: #475569;
  border: 1px solid #d0d5dd;
  border-radius: 6px;
  font-size: 0.8rem;
  cursor: pointer;
}

[data-theme='dark'] .tags-modal,
[data-theme='dark'] .modal-header {
  background: var(--app-surface);
  border-color: var(--app-border);
}

[data-theme='dark'] .modal-title {
  color: var(--app-text);
}

[data-theme='dark'] .btn-close {
  color: var(--app-text-muted);
}

[data-theme='dark'] .hint,
[data-theme='dark'] .modal-status,
[data-theme='dark'] .tag-info small {
  color: var(--app-text-muted);
}

[data-theme='dark'] .tag-item {
  border-color: var(--app-border);
}

[data-theme='dark'] .tag-info strong {
  color: var(--app-text);
}

[data-theme='dark'] .tag-key {
  background: var(--app-surface-muted);
}

[data-theme='dark'] .btn-copy {
  background: var(--app-surface-muted);
  border-color: var(--app-border);
  color: var(--app-text);
}
</style>
