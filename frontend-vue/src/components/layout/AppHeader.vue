<script setup lang="ts">
import { RouterLink, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const router = useRouter()

function handleLogout() {
  auth.logout()
  router.push('/login')
}
</script>

<template>
  <header v-if="auth.isAuthenticated" class="topbar">
    <div class="nav-left">
      <RouterLink to="/" class="brand">Editor Online</RouterLink>
    </div>
    <div class="nav-right">
      <span class="user-name">{{ auth.userName }}</span>
      <button class="btn-logout" @click="handleLogout">Salir</button>
    </div>
  </header>
</template>

<style scoped>
.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.7rem 1.5rem;
  background: white;
  border-bottom: 1px solid #e2e8f0;
  position: sticky;
  top: 0;
  z-index: 10;
}

.brand {
  font-weight: 700;
  font-size: 1.1rem;
  color: #2563eb;
}

.nav-right {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.user-name {
  font-size: 0.85rem;
  color: #475569;
  font-weight: 500;
}

.btn-logout {
  padding: 0.35rem 0.75rem;
  background: #f1f5f9;
  color: #475569;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  font-size: 0.8rem;
  cursor: pointer;
  transition: background 0.15s;
}

.btn-logout:hover {
  background: #e2e8f0;
}
</style>
