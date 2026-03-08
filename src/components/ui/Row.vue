<script setup lang="ts">
defineProps<{ label: string; hint?: string }>()
</script>
<template>
  <div class="row">
    <div class="row-label-group">
      <span class="row-label">{{ label }}</span>
      <span v-if="hint" class="row-hint">{{ hint }}</span>
    </div>
    <div class="row-control"><slot /></div>
  </div>
</template>
<style scoped>
.row { display: flex; align-items: center; justify-content: space-between;
  padding: var(--spacing-sm) 0; min-height: 40px; }
.row-label-group { display: flex; flex-direction: column; gap: 2px; }
.row-label { font-family: var(--font-sans); font-size: var(--font-size-base);
  font-weight: 400; color: var(--color-text); }
.row-hint { font-size: var(--font-size-xs); color: var(--color-text-muted); }
.row-control { display: flex; align-items: center; gap: var(--spacing-sm); }
</style>
