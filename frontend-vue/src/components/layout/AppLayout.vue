<script setup lang="ts">
import { RouterView } from 'vue-router'
import AppHeader from './AppHeader.vue'
import AppContent from './AppContent.vue'
import AppFooter from './AppFooter.vue'
</script>

<template>
  <div class="app-shell">
    <AppHeader />
    <AppContent>
      <RouterView />
    </AppContent>
    <AppFooter />
  </div>
</template>

<style scoped>
.app-shell {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
</style>
