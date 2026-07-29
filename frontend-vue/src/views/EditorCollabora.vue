<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  getCollaboraSession,
  searchUsers,
  shareDocument,
  listShares,
  removeShare,
} from '@/services/api'
import type { ShareInfo, UserSearchResult } from '@/services/types'
import { useTheme } from '@/composables/useTheme'

const route = useRoute()
const router = useRouter()
const loading = ref(true)
const error = ref('')

const showShareModal = ref(false)
const shareSearchQuery = ref('')
const shareResults = ref<UserSearchResult[]>([])
const shares = ref<ShareInfo[]>([])
const selectedUserId = ref<string | null>(null)
const shareError = ref('')
const { isDark } = useTheme()
const rawIframeUrl = ref('')

const iframeUrl = computed(() => {
  if (!rawIframeUrl.value) return ''
  const url = new URL(rawIframeUrl.value, window.location.origin)
  url.searchParams.set('ui_theme', isDark.value ? 'dark' : 'light')
  return url.toString()
})

const docId = route.params.id as string

onMounted(async () => {
  try {
    const session = await getCollaboraSession(docId)
    rawIframeUrl.value = session.iframe_url
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

async function openShareModal() {
  showShareModal.value = true
  shareError.value = ''
  await loadShares()
}

function closeShareModal() {
  showShareModal.value = false
  shareSearchQuery.value = ''
  shareResults.value = []
  selectedUserId.value = null
}

async function loadShares() {
  try {
    shares.value = await listShares(docId)
  } catch (e) {
    shareError.value = 'Error al cargar compartidos'
  }
}

async function onSearchInput() {
  selectedUserId.value = null
  if (shareSearchQuery.value.length < 2) {
    shareResults.value = []
    return
  }
  try {
    shareResults.value = await searchUsers(shareSearchQuery.value)
  } catch (e) {
    shareResults.value = []
  }
}

async function handleShare() {
  if (!selectedUserId.value) return
  shareError.value = ''
  try {
    await shareDocument(docId, selectedUserId.value)
    shareSearchQuery.value = ''
    shareResults.value = []
    selectedUserId.value = null
    await loadShares()
  } catch (e) {
    shareError.value = String(e)
  }
}

async function handleRemoveShare(userId: string) {
  shareError.value = ''
  try {
    await removeShare(docId, userId)
    await loadShares()
  } catch (e) {
    shareError.value = 'Error al quitar acceso'
  }
}
</script>

<template>
  <div class="editor-collabora">
    <div class="toolbar">
      <button class="btn-back" @click="router.push('/')">← Volver</button>
      <button class="btn-share" @click="openShareModal">Compartir</button>
    </div>
    <div v-if="loading" class="status">Iniciando sesión de Collabora...</div>
    <div v-if="error" class="status error">{{ error }}</div>
    <iframe
      v-if="iframeUrl"
      :src="iframeUrl"
      class="editor-frame"
      allow="clipboard-read; clipboard-write"
      sandbox="allow-scripts allow-forms allow-same-origin allow-popups allow-popups-to-escape-sandbox allow-downloads allow-modals allow-top-navigation-by-user-activation"
    />

    <!-- Share Modal -->
    <Teleport to="body">
      <div v-if="showShareModal" class="modal-overlay" @click.self="closeShareModal">
        <div class="modal">
          <div class="modal-header">
            <h3>Compartir documento</h3>
            <button class="modal-close" @click="closeShareModal">✕</button>
          </div>

          <div class="modal-body">
            <div class="share-section">
              <input
                v-model="shareSearchQuery"
                placeholder="Buscar por nombre, DNI o usuario..."
                class="share-input"
                @input="onSearchInput"
              />
              <div v-if="shareResults.length > 0" class="search-results">
                <div
                  v-for="u in shareResults"
                  :key="u.id"
                  :class="['user-item', { selected: selectedUserId === u.id }]"
                  @click="selectedUserId = u.id"
                >
                  <strong>{{ u.name }}</strong>
                  <small>{{ u.dni || '' }} {{ u.cargo ? '| ' + u.cargo : '' }}</small>
                </div>
              </div>
              <button class="btn-share-action" :disabled="!selectedUserId" @click="handleShare">
                Compartir
              </button>
            </div>

            <div class="share-divider"></div>

            <div class="share-section">
              <h4>Usuarios con acceso ({{ shares.length }})</h4>
              <div v-if="shares.length === 0" class="no-shares">Sin compartir</div>
              <div v-for="s in shares" :key="s.id" class="share-item">
                <div class="share-info">
                  <strong>{{ s.user_name }}</strong>
                  <small>Compartido por {{ s.shared_by_name }}</small>
                </div>
                <button class="btn-remove" @click="handleRemoveShare(s.user_id)">Quitar</button>
              </div>
            </div>

            <div v-if="shareError" class="share-error">{{ shareError }}</div>
          </div>
        </div>
      </div>
    </Teleport>
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
  gap: 0.5rem;
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
.btn-back:hover {
  background: #f1f5f9;
}
.btn-share {
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
.btn-share:hover {
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
</style>

<style>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}
.modal {
  background: white;
  border-radius: 12px;
  width: 420px;
  max-width: 90vw;
  max-height: 85vh;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.15);
}
.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid #e2e8f0;
}
.modal-header h3 {
  font-size: 1rem;
  margin: 0;
}
.modal-close {
  background: none;
  border: none;
  font-size: 1.2rem;
  cursor: pointer;
  color: #888;
  padding: 0.25rem;
}
.modal-close:hover {
  color: #333;
}
.modal-body {
  padding: 1.25rem;
  overflow-y: auto;
  max-height: calc(85vh - 60px);
}
.share-section {
  margin-bottom: 1rem;
}
.share-section h4 {
  font-size: 0.85rem;
  color: #555;
  margin-bottom: 0.5rem;
}
.share-input {
  width: 100%;
  padding: 0.6rem 0.75rem;
  border: 1px solid #d0d5dd;
  border-radius: 6px;
  font-size: 0.9rem;
  box-sizing: border-box;
}
.search-results {
  margin-top: 0.5rem;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  max-height: 200px;
  overflow-y: auto;
}
.user-item {
  padding: 0.6rem 0.75rem;
  cursor: pointer;
  border-bottom: 1px solid #f1f5f9;
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
}
.user-item:last-child {
  border-bottom: none;
}
.user-item:hover {
  background: #f8fafc;
}
.user-item.selected {
  background: #dbeafe;
}
.user-item small {
  color: #888;
  font-size: 0.8rem;
}
.btn-share-action {
  width: 100%;
  padding: 0.6rem;
  margin-top: 0.5rem;
  background: #2563eb;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
}
.btn-share-action:disabled {
  background: #93c5fd;
  cursor: not-allowed;
}
.share-divider {
  height: 1px;
  background: #e2e8f0;
  margin: 1rem 0;
}
.no-shares {
  color: #888;
  font-size: 0.85rem;
  padding: 0.5rem 0;
}
.share-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0;
  border-bottom: 1px solid #f1f5f9;
}
.share-item:last-child {
  border-bottom: none;
}
.share-info {
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
}
.share-info small {
  color: #888;
  font-size: 0.8rem;
}
.btn-remove {
  background: #fee2e2;
  border: 1px solid #fca5a5;
  color: #dc2626;
  padding: 0.3rem 0.6rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8rem;
}
.btn-remove:hover {
  background: #fecaca;
}
.share-error {
  margin-top: 0.75rem;
  padding: 0.5rem;
  background: #fef2f2;
  color: #dc2626;
  border-radius: 4px;
  font-size: 0.85rem;
}
</style>
