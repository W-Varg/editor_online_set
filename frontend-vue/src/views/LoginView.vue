<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

const username = ref('user1')
const password = ref('Admin123@')
const error = ref('')
const loading = ref(false)

async function handleLogin() {
  if (!username.value || !password.value) {
    error.value = 'Ingrese usuario y contraseña'
    return
  }
  loading.value = true
  error.value = ''
  try {
    await auth.login(username.value, password.value)
    router.push('/')
  } catch (e) {
    error.value = 'Usuario o contraseña incorrectos'
  } finally {
    loading.value = false
  }
}

const demoAccounts = ['user1', 'user2', 'user3', 'user4', 'user5']

function fillDemo(user: string) {
  username.value = user
  password.value = 'Admin123@'
}
</script>

<template>
  <div class="login-page">
    <div class="login-card">
      <div class="logo">
        <span class="logo-icon">📝</span>
      </div>
      <h1>Editor Online</h1>
      <p class="subtitle">Sistema de edición colaborativa de documentos</p>

      <form @submit.prevent="handleLogin">
        <div class="field">
          <label>Usuario</label>
          <input v-model="username" placeholder="Usuario" autocomplete="username" />
        </div>
        <div class="field">
          <label>Contraseña</label>
          <input
            v-model="password"
            type="password"
            placeholder="Contraseña"
            autocomplete="current-password"
          />
        </div>

        <div v-if="error" class="error">{{ error }}</div>

        <button type="submit" :disabled="loading" class="btn-login">
          {{ loading ? 'Ingresando...' : 'Iniciar sesión' }}
        </button>
      </form>

      <div class="demo-info">
        <p class="demo-label">Cuentas de prueba (demo):</p>
        <div class="demo-users">
          <span
            v-for="u in demoAccounts"
            :key="u"
            class="demo-user"
            @click="fillDemo(u)"
          >
            {{ u }}
          </span>
        </div>
        <p class="demo-hint">
          Click en el usuario → completa los campos con contraseña <code>Admin123@</code>
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.login-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #1e3a5f 0%, #2d5a87 100%);
  font-family: system-ui, sans-serif;
}
.login-card {
  background: white;
  border-radius: 16px;
  padding: 2.5rem 2rem;
  width: 100%;
  max-width: 400px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  text-align: center;
}
.logo {
  margin-bottom: 0.5rem;
}
.logo-icon {
  font-size: 3rem;
}
h1 {
  margin: 0;
  font-size: 1.5rem;
  color: #1a1a2e;
}
.subtitle {
  color: #666;
  font-size: 0.85rem;
  margin: 0.3rem 0 1.5rem;
}
form {
  text-align: left;
}
.field {
  margin-bottom: 1rem;
}
.field label {
  display: block;
  font-size: 0.8rem;
  font-weight: 600;
  color: #555;
  margin-bottom: 0.35rem;
}
.field input {
  width: 100%;
  padding: 0.65rem 0.75rem;
  border: 1px solid #d0d5dd;
  border-radius: 8px;
  font-size: 0.95rem;
  transition: border-color 0.15s;
}
.field input:focus {
  outline: none;
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
}
.error {
  color: #dc2626;
  font-size: 0.85rem;
  margin-bottom: 1rem;
}
.btn-login {
  width: 100%;
  padding: 0.7rem;
  background: #2563eb;
  color: white;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  font-size: 1rem;
  cursor: pointer;
  transition: background 0.15s;
}
.btn-login:hover {
  background: #1d4ed8;
}
.btn-login:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.demo-info {
  margin-top: 1.5rem;
  padding-top: 1.5rem;
  border-top: 1px solid #eee;
}
.demo-label {
  font-size: 0.8rem;
  color: #888;
  font-weight: 600;
  margin-bottom: 0.5rem;
}
.demo-users {
  display: flex;
  gap: 0.4rem;
  justify-content: center;
  flex-wrap: wrap;
}
.demo-user {
  padding: 0.3rem 0.6rem;
  background: #f0f4ff;
  color: #2563eb;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s;
}
.demo-user:hover {
  background: #dbeafe;
}
.demo-hint {
  font-size: 0.75rem;
  color: #999;
  margin-top: 0.5rem;
}
code {
  background: #f1f5f9;
  padding: 0.1rem 0.3rem;
  border-radius: 3px;
  font-size: 0.75rem;
}
</style>
