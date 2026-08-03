<!-- eslint-disable @typescript-eslint/no-explicit-any -->
<script setup lang="ts">
import { computed, ref, onBeforeUnmount, onMounted, nextTick, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getTemplate, getTemplateOnlyOfficeConfig, getTemplateCollaboraSession } from '@/services/api'
import { useTheme } from '@/composables/useTheme'
import { useCollaboraTags } from '@/composables/useCollaboraTags'
import TagsModal from '@/components/TagsModal.vue'
import TemplatePreviewModal from '@/components/TemplatePreviewModal.vue'

const route = useRoute()
const router = useRouter()
const appHost = window.location.hostname || 'localhost'
const appProtocol = window.location.protocol || 'http:'
const onlyOfficeBaseUrl = import.meta.env.VITE_ONLYOFFICE_URL || `${appProtocol}//${appHost}:8092`
const loading = ref(true)
const error = ref('')
const editorKind = ref<'onlyoffice' | 'collabora' | ''>('')
const containerId = 'template-editor-container'
const { isDark } = useTheme()
const editor = ref<any>(null)
let editorConfig: any = null
let editorTimer: ReturnType<typeof setTimeout> | undefined

const collaboraIframe = ref<HTMLIFrameElement | null>(null)
const rawIframeUrl = ref('')
const templateName = ref('')
const showTagsModal = ref(false)
const showPreviewModal = ref(false)
const toast = ref('')
let toastTimer: ReturnType<typeof setTimeout> | undefined

const { insertTag } = useCollaboraTags(collaboraIframe)

const iframeUrl = computed(() => {
  if (!rawIframeUrl.value) return ''
  const url = new URL(rawIframeUrl.value, window.location.origin)
  url.searchParams.set('ui_theme', isDark.value ? 'dark' : 'light')
  return url.toString()
})

onMounted(async () => {
  const id = route.params.id as string
  try {
    const template = await getTemplate(id)
    templateName.value = template.name

    if (template.editor === 'collabora') {
      editorKind.value = 'collabora'
      const session = await getTemplateCollaboraSession(id)
      rawIframeUrl.value = session.iframe_url
      loading.value = false
      return
    }

    editorKind.value = 'onlyoffice'
    const config = await getTemplateOnlyOfficeConfig(id)

    if (!(window as any).DocsAPI) {
      await loadScript(`${onlyOfficeBaseUrl.replace(/\/$/, '')}/web-apps/apps/api/documents/api.js`)
    }

    editorConfig = config
    await mountEditor()
  } catch (e) {
    error.value = 'Error al cargar la plantilla: ' + String(e)
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
  if (editorKind.value !== 'onlyoffice') return
  if (!editor.value || !editorConfig) return
  editor.value.destroyEditor?.()
  editor.value = null
  loading.value = true
  await mountEditor()
})

onBeforeUnmount(() => {
  if (editorTimer) clearTimeout(editorTimer)
  if (toastTimer) clearTimeout(toastTimer)
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

async function onSelectTag(key: string) {
  const ok = await insertTag(key)
  if (!ok) {
    try {
      await navigator.clipboard.writeText(`{{${key}}}`)
      showToast(`No se pudo insertar en el cursor. Etiqueta {{${key}}} copiada al portapapeles.`)
    } catch {
      showToast('No se pudo insertar ni copiar la etiqueta.')
    }
  }
}

function showToast(message: string) {
  toast.value = message
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => {
    toast.value = ''
  }, 4000)
}
</script>

<template>
  <div class="template-editor">
    <div class="toolbar">
      <button class="btn-back" @click="router.push('/templates')">← Volver a plantillas</button>
      <span class="toolbar-title">Editar plantilla{{ templateName ? `: ${templateName}` : '' }}</span>
      <div v-if="editorKind === 'collabora'" class="toolbar-actions">
        <button class="btn-tags" @click="showTagsModal = true">Etiquetas</button>
        <button class="btn-preview" @click="showPreviewModal = true">Previsualizar</button>
      </div>
    </div>
    <div v-if="loading" class="status">
      {{ editorKind === 'collabora' ? 'Iniciando sesión de Collabora...' : 'Cargando editor ONLYOFFICE...' }}
    </div>
    <div v-if="error" class="status error">{{ error }}</div>
    <iframe
      v-if="editorKind === 'collabora' && iframeUrl"
      ref="collaboraIframe"
      :src="iframeUrl"
      class="editor-frame"
      allow="clipboard-read; clipboard-write"
      sandbox="allow-scripts allow-forms allow-same-origin allow-popups allow-popups-to-escape-sandbox allow-downloads allow-modals allow-top-navigation-by-user-activation"
    />
    <div v-if="editorKind === 'onlyoffice'" :id="containerId" class="editor-frame" />

    <TagsModal
      :show="showTagsModal"
      title="Insertar etiqueta en la plantilla"
      @close="showTagsModal = false"
      @select="onSelectTag"
    />
    <TemplatePreviewModal
      :template-id="showPreviewModal ? (route.params.id as string) : null"
      :template-name="templateName"
      @close="showPreviewModal = false"
    />
    <div v-if="toast" class="success-toast" role="status">{{ toast }}</div>
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
.toolbar-actions {
  display: flex;
  gap: 0.5rem;
  margin-left: auto;
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
.btn-tags,
.btn-preview {
  padding: 0.4rem 0.85rem;
  background: #7c3aed;
  border: none;
  border-radius: 6px;
  font-size: 0.85rem;
  font-weight: 500;
  cursor: pointer;
  color: white;
  transition: opacity 0.15s;
}
.btn-preview {
  background: #475569;
}
.btn-tags:hover,
.btn-preview:hover {
  opacity: 0.85;
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
  border: none;
}
.success-toast {
  position: fixed;
  right: 1.25rem;
  bottom: 1.25rem;
  z-index: 20;
  padding: 0.85rem 1rem;
  border: 1px solid #86efac;
  border-radius: 8px;
  background: #f0fdf4;
  color: #166534;
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.14);
  font-size: 0.9rem;
}
</style>
