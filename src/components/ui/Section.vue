<script setup lang="ts">
defineProps<{ title: string }>()
</script>
<template>
  <div class="section">
    <h2 class="section-title">{{ title }}</h2>
    <hr class="section-divider" />
    <div class="section-body"><slot /></div>
  </div>
</template>
<style scoped>
.section { margin-bottom: var(--spacing-lg); }
.section-title { font-family: var(--font-sans); font-size: var(--font-size-sm);
  font-weight: 600; color: var(--color-text-muted); text-transform: uppercase;
  letter-spacing: 0.06em; margin: 0 0 var(--spacing-sm); }
.section-divider { border: none; border-top: 1px solid var(--color-divider);
  margin: 0 0 var(--spacing-md); }
.section-body { display: flex; flex-direction: column; gap: var(--spacing-sm); }
</style>
