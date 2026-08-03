export interface User {
  id: string
  username: string
  name: string
  dni?: string
  cargo?: string
}

export interface AuthResponse {
  token: string
  user: User
}

export interface Document {
  id: string
  name: string
  ext: string
  mime: string
  editor: string
  size: number
  status: string
  owner_id: string
  owner_name: string
  created_at: string
  updated_at: string
  shared?: boolean
  shared_by?: string
  shared_by_name?: string
}

export interface CreateDocumentPayload {
  name: string
  ext: string
  editor: string
  template_id?: string
}

export interface Template {
  id: string
  name: string
  ext: string
  mime: string
  editor: string
  size: number
  owner_id: string
  owner_name: string
  created_at: string
  updated_at: string
}

export interface CreateTemplatePayload {
  name: string
  ext: string
  source_document_id?: string
}

export interface CollaboraSession {
  iframe_url: string
  access_token: string
}

export interface OnlyOfficeConfig {
  document: {
    fileType: string
    key: string
    title: string
    url: string
  }
  documentType: string
  editorConfig: {
    callbackUrl: string
    lang: string
    mode: string
    customization: {
      autosave: boolean
      forcesave: boolean
    }
    user: {
      id: string
      name: string
    }
  }
  token: string
}

export interface ShareInfo {
  id: string
  document_id: string
  user_id: string
  user_name: string
  shared_by: string
  shared_by_name: string
  permission: string
  created_at: string
}

export interface UserSearchResult {
  id: string
  username: string
  name: string
  dni?: string
  cargo?: string
}
