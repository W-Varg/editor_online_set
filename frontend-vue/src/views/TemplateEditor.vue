<!-- eslint-disable @typescript-eslint/no-explicit-any -->
<script setup lang="ts">
import { ref, onBeforeUnmount, onMounted, nextTick, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getTemplateOnlyOfficeConfig } from '@/services/api'
import { useTheme } from '@/composables/useTheme'

const route = useRoute()
const router = useRouter()
const appHost = window.location.hostname || 'localhost'
const appProtocol = window.location.protocol || 'http:'
const onlyOfficeBaseUrl = import.meta.env.VITE_ONLYOFFICE_URL || `${appProtocol}//${appHost}:8092`
const loading = ref(true)
const error = ref('')
const containerId = 'template-editor-container'
const { isDark } = useTheme()
const editor = ref<any>(null)
let editorConfig: any = null
let editorTimer: ReturnType<typeof setTimeout> | undefined

onMounted(async () => {
  const id = route.params.id as string
  try {
    const config = await getTemplateOnlyOfficeConfig(id)

    if (!(window as any).DocsAPI) {
      await loadScript(`${onlyOfficeBaseUrl.replace(/\/$/, '')}/web-apps/apps/api/documents/api.js`)
    }

    editorConfig = config
    await mountEditor()
  } catch (e) {
    error.value = 'Error al cargar configuración: ' + String(e)
    loading.value = false
  }
})

async function mountEditor() {
  await nextTick()
  editorConfig.editorConfig.customization.uiTheme = isDark.value ? 'theme-dark' : 'theme-light'
  editorTimer = setTimeout(() => {
    const el = document.getElementById(containerId)
    if (!el) {
      error.value = 'Contenedor del editor no encontrado'
      return
    }
    try {
      editor.value = new (window as any).DocsAPI.DocEditor(containerId, editorConfig)
      loading.value = false
    } catch (e) {
      error.value = 'Error al inicializar el editor: ' + String(e)
      loading.value = false
    }
  }, 100)
}

watch(isDark, async () => {
  if (!editor.value || !editorConfig) return
  editor.value.destroyEditor?.()
  editor.value = null
  loading.value = true
  await mountEditor()
})

onBeforeUnmount(() => {
  if (editorTimer) clearTimeout(editorTimer)
  editor.value?.destroyEditor?.()
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
  <div class="template-editor">
    <div class="toolbar">
      <button class="btn-back" @click="router.push('/templates')">← Volver a plantillas</button>
      <span class="toolbar-title">Editar plantilla</span>
    </div>
    <div v-if="loading" class="status">Cargando editor ONLYOFFICE...</div>
    <div v-if="error" class="status error">{{ error }}</div>
    <div :id="containerId" class="editor-frame" />
  </div>
</template>

<style scoped>
.template-editor {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
}
.toolbar {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.5rem 1rem;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
  flex-shrink: 0;
}
.toolbar-title {
  font-size: 0.9rem;
  font-weight: 600;
  color: #334155;
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
.btn-back:hover {
  background: #f1f5f9;
}
.status {
  padding: 2rem;
  text-align: center;
  font-family: system-ui, sans-serif;
  color: #666;
}
.status.error {
  color: #c0392b;
}
.editor-frame {
  width: 100%;
  flex: 1;
}
</style>
