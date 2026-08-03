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

export async function listDocuments(tab: string = 'mine') {
  const res = await fetch(`${API_BASE}/api/documents?tab=${tab}`, { headers: headers() })
  if (!res.ok) throw new Error('Failed to fetch documents')
  return res.json()
}

export async function getDocument(id: string) {
  const res = await fetch(`${API_BASE}/api/documents/${id}`)
  if (!res.ok) throw new Error('Document not found')
  return res.json()
}

export async function createDocument(data: {
  name: string
  ext: string
  editor: string
  template_id?: string
}) {
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
  if (!res.ok) {
    const message = await res.text()
    throw new Error(message || 'Failed to convert document')
  }
  return res.json()
}

export async function previewDocument(id: string): Promise<Blob> {
  const res = await fetch(`${API_BASE}/api/documents/${id}/preview`, {
    headers: headers(),
  })
  if (!res.ok) {
    const message = await res.text()
    throw new Error(message || 'No se pudo generar la previsualización')
  }
  return res.blob()
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

export async function listTemplates() {
  const res = await fetch(`${API_BASE}/api/templates`, { headers: headers() })
  if (!res.ok) throw new Error('Failed to fetch templates')
  return res.json()
}

export async function createTemplate(data: { name: string; ext: string; source_document_id?: string }) {
  const res = await fetch(`${API_BASE}/api/templates`, {
    method: 'POST',
    headers: headers(),
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const message = await res.text()
    throw new Error(message || 'Failed to create template')
  }
  return res.json()
}

export async function renameTemplate(id: string, name: string) {
  const res = await fetch(`${API_BASE}/api/templates/${id}`, {
    method: 'PUT',
    headers: headers(),
    body: JSON.stringify({ name }),
  })
  if (!res.ok) throw new Error('Failed to rename template')
  return res.json()
}

export async function deleteTemplate(id: string) {
  const res = await fetch(`${API_BASE}/api/templates/${id}`, {
    method: 'DELETE',
    headers: headers(),
  })
  if (!res.ok) throw new Error('Failed to delete template')
  return res.json()
}

export async function previewTemplate(id: string): Promise<Blob> {
  const res = await fetch(`${API_BASE}/api/templates/${id}/preview`, {
    headers: headers(),
  })
  if (!res.ok) {
    const message = await res.text()
    throw new Error(message || 'No se pudo generar la previsualización')
  }
  return res.blob()
}

export async function getTemplateOnlyOfficeConfig(id: string) {
  const res = await fetch(`${API_BASE}/api/onlyoffice/config/template/${id}`, {
    headers: headers(),
  })
  if (!res.ok) throw new Error('Failed to get template OnlyOffice config')
  return res.json()
}

export async function listUsers() {
  const res = await fetch(`${API_BASE}/api/users`, { headers: headers() })
  if (!res.ok) throw new Error('Failed to fetch users')
  return res.json()
}

export async function searchUsers(query: string) {
  const res = await fetch(`${API_BASE}/api/users/search?q=${encodeURIComponent(query)}`, {
    headers: headers(),
  })
  if (!res.ok) throw new Error('Failed to search users')
  return res.json()
}

export async function shareDocument(docId: string, userId: string) {
  const res = await fetch(`${API_BASE}/api/documents/${docId}/shares`, {
    method: 'POST',
    headers: headers(),
    body: JSON.stringify({ user_id: userId }),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || 'Failed to share document')
  }
  return res.json()
}

export async function listShares(docId: string) {
  const res = await fetch(`${API_BASE}/api/documents/${docId}/shares`, { headers: headers() })
  if (!res.ok) throw new Error('Failed to list shares')
  return res.json()
}

export async function removeShare(docId: string, userId: string) {
  const res = await fetch(`${API_BASE}/api/documents/${docId}/shares/${userId}`, {
    method: 'DELETE',
    headers: headers(),
  })
  if (!res.ok) throw new Error('Failed to remove share')
  return res.json()
}
