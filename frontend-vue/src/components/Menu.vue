<!-- eslint-disable vue/multi-word-component-names -->
<script setup lang="ts">
import { RouterLink } from 'vue-router'

const items = [
  { label: 'Documentos', to: '/' },
  { label: 'Usuarios', to: '/users' },
  { label: 'Plantillas', to: '/templates' },
]
</script>

<template>
  <nav class="menu">
    <RouterLink
      v-for="item in items"
      :key="item.to"
      :to="item.to"
      class="menu-item"
      active-class="active"
      exact-active-class="active"
    >
      {{ item.label }}
    </RouterLink>
  </nav>
</template>

<style scoped>
.menu {
  display: flex;
  align-items: center;
  gap: 0.35rem;
}

.menu-item {
  padding: 0.45rem 0.9rem;
  border-radius: 8px;
  font-size: 0.88rem;
  font-weight: 500;
  color: #475569;
  transition: background 0.15s, color 0.15s;
}

.menu-item:hover {
  background: #f1f5f9;
  color: #1d4ed8;
}

.menu-item.active {
  background: #2563eb;
  color: white;
}

[data-theme='dark'] .menu-item {
  color: var(--app-text-muted);
}

[data-theme='dark'] .menu-item:hover {
  background: var(--app-surface-muted);
  color: var(--app-text);
}

[data-theme='dark'] .menu-item.active {
  background: #2563eb;
  color: white;
}
</style>
