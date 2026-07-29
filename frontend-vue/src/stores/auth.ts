import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { login as apiLogin } from '@/services/api'
import type { Document } from '@/services/types'

export interface AuthUser {
  id: string
  username: string
  name: string
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref(localStorage.getItem('token') || '')
  const user = ref<AuthUser | null>(
    JSON.parse(localStorage.getItem('user') || 'null')
  )

  const isAuthenticated = computed(() => !!token.value)
  const userName = computed(() => user.value?.name || '')
  const userId = computed(() => user.value?.id || '')

  async function login(username: string, password: string) {
    const res = await apiLogin(username, password)
    token.value = res.token
    user.value = res.user
    localStorage.setItem('token', res.token)
    localStorage.setItem('user', JSON.stringify(res.user))
  }

  function logout() {
    token.value = ''
    user.value = null
    localStorage.removeItem('token')
    localStorage.removeItem('user')
    // Clear editor stores
    const keys = Object.keys(localStorage)
    for (const key of keys) {
      if (key.startsWith('pinia-')) localStorage.removeItem(key)
    }
  }

  return { token, user, isAuthenticated, userName, userId, login, logout }
})
