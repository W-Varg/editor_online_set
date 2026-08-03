<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { listUsers } from '@/services/api'
import type { UserSearchResult } from '@/services/types'

const users = ref<UserSearchResult[]>([])
const loading = ref(true)
const error = ref('')

onMounted(async () => {
  try {
    users.value = await listUsers()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="users">
    <section class="user-list">
      <h2>Usuarios</h2>
      <div v-if="loading" class="state-msg">Cargando...</div>
      <div v-else-if="error" class="error-banner">{{ error }}</div>
      <div v-else-if="users.length === 0" class="state-msg">No hay usuarios registrados.</div>
      <table v-else>
        <thead>
          <tr>
            <th>Nombre</th>
            <th>Usuario</th>
            <th>DNI</th>
            <th>Cargo</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in users" :key="user.id">
            <td class="user-name">{{ user.name }}</td>
            <td>{{ user.username }}</td>
            <td>{{ user.dni || '—' }}</td>
            <td>{{ user.cargo || '—' }}</td>
          </tr>
        </tbody>
      </table>
    </section>
  </div>
</template>

<style scoped>
.users {
  max-width: 1100px;
  margin: 0 auto;
}

.user-list {
  background: white;
  border-radius: 10px;
  padding: 1.5rem;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.06);
}

h2 {
  font-size: 1.1rem;
  margin-bottom: 1rem;
}

.state-msg {
  padding: 1rem 0.25rem;
  color: #64748b;
}

.error-banner {
  padding: 0.85rem 1rem;
  background: #fef2f2;
  border: 1px solid #fecaca;
  color: #b91c1c;
  border-radius: 8px;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th,
td {
  text-align: left;
  padding: 0.85rem 0.6rem;
  border-bottom: 1px solid #edf2f7;
}

th {
  font-size: 0.8rem;
  color: #475569;
  background: #f8fafc;
}

.user-name {
  font-weight: 600;
}

[data-theme='dark'] .user-list {
  background: var(--app-surface);
  border-color: var(--app-border);
}

[data-theme='dark'] th {
  background: var(--app-surface-muted);
  color: var(--app-text);
}

[data-theme='dark'] td {
  border-color: var(--app-border-soft);
}

[data-theme='dark'] .state-msg {
  color: var(--app-text-muted);
}
</style>
