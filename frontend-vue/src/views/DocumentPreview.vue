<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { previewDocument } from '@/services/api'
import type { HeaderFooterMode } from '@/services/types'

const route = useRoute()
const router = useRouter()
const pdfUrl = ref('')
const loading = ref(true)
const error = ref('')

onMounted(async () => {
  const mode = (route.query.header_footer as HeaderFooterMode | undefined) ?? 'preserve'
  try {
    const blob = await previewDocument(route.params.id as string, mode)
    pdfUrl.value = URL.createObjectURL(blob)
  } catch (reason) {
    error.value = reason instanceof Error ? reason.message : String(reason)
  } finally {
    loading.value = false
  }
})

onBeforeUnmount(() => {
  if (pdfUrl.value) URL.revokeObjectURL(pdfUrl.value)
})
</script>

<template>
  <div class="preview-page">
    <div class="toolbar">
      <button class="btn-back" @click="router.push('/')">Volver a documentos</button>
      <span>Previsualización PDF</span>
    </div>
    <div v-if="loading" class="status">Generando previsualización...</div>
    <div v-else-if="error" class="status error">{{ error }}</div>
    <iframe v-else :src="pdfUrl" class="pdf-frame" title="Previsualización del documento" />
  </div>
</template>

<style scoped>
.preview-page {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #e5e7eb;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.65rem 1rem;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
  color: #334155;
  font-size: 0.9rem;
}

.btn-back {
  padding: 0.4rem 0.85rem;
  background: white;
  border: 1px solid #d0d5dd;
  border-radius: 6px;
  cursor: pointer;
  color: #1a1a2e;
}

.status {
  padding: 2rem;
  text-align: center;
  color: #64748b;
}

.status.error {
  color: #b91c1c;
}

.pdf-frame {
  width: 100%;
  flex: 1;
  border: 0;
}
</style>
