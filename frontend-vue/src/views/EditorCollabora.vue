<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { getCollaboraSession } from '@/services/api'

const route = useRoute()
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
.status {
  padding: 2rem;
  text-align: center;
  font-family: system-ui, sans-serif;
  color: #666;
}
.status.error { color: #c0392b; }
.editor-frame {
  width: 100%;
  height: 100%;
  border: none;
}
</style>
