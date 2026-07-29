<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useRoute } from 'vue-router'
import { getOnlyOfficeConfig } from '@/services/api'

const route = useRoute()
const appHost = window.location.hostname || 'localhost'
const appProtocol = window.location.protocol || 'http:'
const loading = ref(true)
const error = ref('')
const containerId = 'editor-container'

onMounted(async () => {
  const id = route.params.id as string
  try {
    const config = await getOnlyOfficeConfig(id)

    // Check if DocsAPI is available, load if not
    if (!(window as any).DocsAPI) {
      await loadScript(`${appProtocol}//${appHost}:8092/web-apps/apps/api/documents/api.js`)
    }

    // Wait a tick for the DOM to render the container
    setTimeout(() => {
      const el = document.getElementById(containerId)
      if (!el) {
        error.value = 'Contenedor del editor no encontrado'
        return
      }
      try {
        new (window as any).DocsAPI.DocEditor(containerId, config)
        loading.value = false
      } catch (e) {
        error.value = 'Error al inicializar el editor: ' + String(e)
        loading.value = false
      }
    }, 100)
  } catch (e) {
    error.value = 'Error al cargar configuracion: ' + String(e)
    loading.value = false
  }
})

function loadScript(src: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const script = document.createElement('script')
    script.src = src
    script.onload = () => resolve()
    script.onerror = () => reject(new Error('Failed to load ' + src))
    document.head.appendChild(script)
  })
}
</script>

<template>
  <div class="editor-onlyoffice">
    <div v-if="loading" class="status">Cargando editor ONLYOFFICE...</div>
    <div v-if="error" class="status error">{{ error }}</div>
    <div :id="containerId" class="editor-frame" />
  </div>
</template>

<style scoped>
.editor-onlyoffice {
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
}
</style>
