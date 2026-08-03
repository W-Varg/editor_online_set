<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import { previewTemplate } from '@/services/api'

const props = defineProps<{
  templateId: string | null
  templateName?: string
}>()

const emit = defineEmits<{ (e: 'close'): void }>()

const loading = ref(false)
const error = ref('')
const pdfUrl = ref('')

watch(
  () => props.templateId,
  async (id) => {
    if (!id) return
    loading.value = true
    error.value = ''
    if (pdfUrl.value) URL.revokeObjectURL(pdfUrl.value)
    pdfUrl.value = ''
    try {
      const blob = await previewTemplate(id)
      pdfUrl.value = URL.createObjectURL(blob)
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      loading.value = false
    }
  },
  { immediate: true },
)

function close() {
  if (pdfUrl.value) URL.revokeObjectURL(pdfUrl.value)
  pdfUrl.value = ''
  emit('close')
}

onBeforeUnmount(() => {
  if (pdfUrl.value) URL.revokeObjectURL(pdfUrl.value)
})
</script>

<template>
  <Teleport to="body">
    <div v-if="templateId" class="preview-overlay" @click.self="close">
      <div class="preview-modal">
        <div class="modal-header">
          <span class="modal-title">
            {{ loading ? 'Generando previsualización...' : `Previsualización: ${templateName || ''}` }}
          </span>
          <button class="btn-close" type="button" @click="close">×</button>
        </div>
        <div v-if="loading" class="modal-status">Generando PDF de la plantilla...</div>
        <div v-else-if="error" class="modal-status error">{{ error }}</div>
        <iframe v-else :src="pdfUrl" class="modal-frame" title="Previsualización de plantilla" />
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.preview-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  padding: 1.5rem;
}

.preview-modal {
  background: white;
  border-radius: 10px;
  width: min(900px, 100%);
  height: min(85vh, 100%);
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

.btn-close:hover {
  color: #0f172a;
}

.modal-status {
  padding: 2rem;
  text-align: center;
  color: #64748b;
}

.modal-status.error {
  color: #b91c1c;
}

.modal-frame {
  width: 100%;
  flex: 1;
  border: 0;
}

[data-theme='dark'] .preview-modal,
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
</style>
