const APP_HOST = window.location.hostname || 'localhost'
const APP_PROTOCOL = window.location.protocol || 'http:'
const API_BASE = `${APP_PROTOCOL}//${APP_HOST}:8091`

function headers() {
  const token = localStorage.getItem('token') || ''
  const h: Record<string, string> = { 'Content-Type': 'application/json' }
  if (token) h['Authorization'] = `Bearer ${token}`
  return h
}

export async function login(username: string, password: string) {
  const res = await fetch(`${API_BASE}/api/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password }),
  })
  if (!res.ok) throw new Error('Credenciales inválidas')
  return res.json()
}

export async function listDocuments() {
  const res = await fetch(`${API_BASE}/api/documents`, { headers: headers() })
  if (!res.ok) throw new Error('Failed to fetch documents')
  return res.json()
}

export async function getDocument(id: string) {
  const res = await fetch(`${API_BASE}/api/documents/${id}`)
  if (!res.ok) throw new Error('Document not found')
  return res.json()
}

export async function createDocument(data: { name: string; ext: string; editor: string }) {
  const res = await fetch(`${API_BASE}/api/documents`, {
    method: 'POST',
    headers: headers(),
    body: JSON.stringify(data),
  })
  if (!res.ok) throw new Error('Failed to create document')
  return res.json()
}

export async function deleteDocument(id: string) {
  const res = await fetch(`${API_BASE}/api/documents/${id}`, {
    method: 'DELETE',
    headers: headers(),
  })
  if (!res.ok) throw new Error('Failed to delete document')
  return res.json()
}

export async function convertToPdf(id: string) {
  const res = await fetch(`${API_BASE}/api/documents/${id}/convert`, {
    method: 'POST',
    headers: headers(),
  })
  if (!res.ok) throw new Error('Failed to convert document')
  return res.json()
}

export async function getCollaboraSession(id: string) {
  const res = await fetch(`${API_BASE}/api/collabora/session/${id}`, { headers: headers() })
  if (!res.ok) throw new Error('Failed to get Collabora session')
  return res.json()
}

export async function getOnlyOfficeConfig(id: string) {
  const res = await fetch(`${API_BASE}/api/onlyoffice/config/${id}`, { headers: headers() })
  if (!res.ok) throw new Error('Failed to get OnlyOffice config')
  return res.json()
}
