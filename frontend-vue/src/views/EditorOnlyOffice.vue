<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getOnlyOfficeConfig } from '@/services/api'

const route = useRoute()
const router = useRouter()
const appHost = window.location.hostname || 'localhost'
const appProtocol = window.location.protocol || 'http:'
const loading = ref(true)
const error = ref('')
const containerId = 'editor-container'

onMounted(async () => {
  const id = route.params.id as string
  try {
    const config = await getOnlyOfficeConfig(id)

    if (!(window as any).DocsAPI) {
      await loadScript(`${appProtocol}//${appHost}:8092/web-apps/apps/api/documents/api.js`)
    }

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
    <div class="toolbar">
      <button class="btn-back" @click="router.push('/')">← Volver a documentos</button>
    </div>
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
}
</style>
