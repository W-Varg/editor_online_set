export interface User {
  id: string
  username: string
  name: string
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
  created_at: string
  updated_at: string
}

export interface CreateDocumentPayload {
  name: string
  ext: string
  editor: string
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
