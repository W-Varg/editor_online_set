<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getCollaboraSession } from '@/services/api'

const route = useRoute()
const router = useRouter()
const iframeUrl = ref('')
const loading = ref(true)
const error = ref('')

onMounted(async () => {
  const id = route.params.id as string
  try {
    const session = await getCollaboraSession(id)
    iframeUrl.value = session.iframe_url
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="editor-collabora">
    <div class="toolbar">
      <button class="btn-back" @click="router.push('/')">← Volver a documentos</button>
    </div>
    <div v-if="loading" class="status">Iniciando sesion de Collabora...</div>
    <div v-if="error" class="status error">{{ error }}</div>
    <iframe
      v-if="iframeUrl"
      :src="iframeUrl"
      class="editor-frame"
      allow="clipboard-read; clipboard-write"
      sandbox="allow-scripts allow-forms allow-same-origin allow-popups allow-popups-to-escape-sandbox allow-downloads allow-modals allow-top-navigation-by-user-activation"
    />
  </div>
</template>

<style scoped>
.editor-collabora {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
}
.toolbar {
  display: flex;
  align-items: center;
  padding: 0.5rem 1rem;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
  flex-shrink: 0;
}
.btn-back {
  padding: 0.4rem 0.85rem;
  background: white;
  border: 1px solid #d0d5dd;
  border-radius: 6px;
  font-size: 0.85rem;
  font-weight: 500;
  cursor: pointer;
  color: #1a1a2e;
  transition: background 0.15s;
}
.btn-back:hover { background: #f1f5f9; }
.status {
  padding: 2rem;
  text-align: center;
  font-family: system-ui, sans-serif;
  color: #666;
}
.status.error { color: #c0392b; }
.editor-frame {
  width: 100%;
  flex: 1;
  border: none;
}
</style>
