<template>
  <AppLayout />
</template>

<script setup lang="ts">
import AppLayout from '@/components/layout/AppLayout.vue'
</script>

<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
html, body, #app { min-height: 100%; }
body {
  font-family: system-ui, -apple-system, sans-serif;
  background: var(--app-background, #f5f7fa);
  color: var(--app-text, #1a1a2e);
  transition: background-color 0.2s, color 0.2s;
}
a { text-decoration: none; }

:root {
  --app-background: #f5f7fa;
  --app-surface: #ffffff;
  --app-surface-muted: #f8fafc;
  --app-border: #e2e8f0;
  --app-border-soft: #edf2f7;
  --app-text: #1a1a2e;
  --app-text-muted: #64748b;
}

:root[data-theme='dark'] {
  --app-background: #0f172a;
  --app-surface: #182235;
  --app-surface-muted: #202d42;
  --app-border: #334155;
  --app-border-soft: #29384f;
  --app-text: #e5e7eb;
  --app-text-muted: #a8b3c4;
}

[data-theme='dark'] .topbar,
[data-theme='dark'] .footer,
[data-theme='dark'] .create-form,
[data-theme='dark'] .doc-list,
[data-theme='dark'] .modal,
[data-theme='dark'] .toolbar {
  background: var(--app-surface) !important;
  color: var(--app-text);
  border-color: var(--app-border) !important;
}

[data-theme='dark'] .field input,
[data-theme='dark'] .field select,
[data-theme='dark'] .tab,
[data-theme='dark'] .btn-back,
[data-theme='dark'] .btn-logout {
  background: var(--app-surface-muted) !important;
  color: var(--app-text) !important;
  border-color: var(--app-border) !important;
}

[data-theme='dark'] th { background: var(--app-surface-muted); color: var(--app-text); }
[data-theme='dark'] td { border-color: var(--app-border-soft); }
[data-theme='dark'] .user-name,
[data-theme='dark'] .owner-cell,
[data-theme='dark'] .date,
[data-theme='dark'] .state-msg { color: var(--app-text-muted); }
[data-theme='dark'] .btn-theme { color: var(--app-text); border-color: var(--app-border); }
[data-theme='dark'] .btn-theme:hover { background: var(--app-surface-muted); }
</style>
