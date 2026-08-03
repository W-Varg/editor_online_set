<script setup lang="ts">
import type { HeaderFooterMode } from '@/services/types'

defineProps<{
  title?: string
}>()

const emit = defineEmits<{
  (e: 'cancel'): void
  (e: 'confirm', mode: HeaderFooterMode): void
}>()
</script>

<template>
  <Teleport to="body">
    <div class="hf-overlay" @click.self="emit('cancel')">
      <div class="hf-modal">
        <div class="modal-header">
          <span class="modal-title">
            {{ title ? `Encabezado y pie · ${title}` : 'Encabezado y pie de página' }}
          </span>
          <button class="btn-close" type="button" @click="emit('cancel')">×</button>
        </div>
        <p class="modal-desc">Elige cómo se mostrará el encabezado y el pie de página en este PDF:</p>
        <div class="options">
          <button class="option preserve" type="button" @click="emit('confirm', 'preserve')">
            <span class="option-title">Preservar</span>
            <span class="option-desc">No inyecta nada: respeta el encabezado y pie que ya tenga el archivo.</span>
          </button>
          <button class="option replace" type="button" @click="emit('confirm', 'replace')">
            <span class="option-title">Reemplazar</span>
            <span class="option-desc">Inyecta el encabezado y pie editables del sistema (título, QR, link, página y fecha).</span>
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.hf-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 120;
  padding: 1.5rem;
}

.hf-modal {
  background: white;
  border-radius: 10px;
  width: min(520px, 100%);
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.3);
  padding: 1.25rem 1.5rem 1.5rem;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid #e2e8f0;
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

.modal-desc {
  margin: 0.9rem 0;
  font-size: 0.85rem;
  color: #64748b;
}

.options {
  display: flex;
  flex-direction: column;
  gap: 0.7rem;
}

.option {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.25rem;
  padding: 0.8rem 1rem;
  border: 1px solid #d0d5dd;
  border-radius: 8px;
  background: #f8fafc;
  cursor: pointer;
  text-align: left;
}

.option:hover {
  border-color: #2563eb;
  background: #eff6ff;
}

.option-title {
  font-size: 0.9rem;
  font-weight: 700;
  color: #1e293b;
}

.option-desc {
  font-size: 0.78rem;
  color: #64748b;
}

[data-theme='dark'] .hf-modal,
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

[data-theme='dark'] .modal-desc,
[data-theme='dark'] .option-desc {
  color: var(--app-text-muted);
}

[data-theme='dark'] .option {
  background: var(--app-surface-muted, #1e293b);
  border-color: var(--app-border);
}

[data-theme='dark'] .option-title {
  color: var(--app-text);
}
</style>
