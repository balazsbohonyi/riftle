<script setup lang="ts">
import { ref } from 'vue'
const props = defineProps<{ modelValue: string[] }>()
const emit = defineEmits<{
  (e: 'update:modelValue', v: string[]): void
  (e: 'change', v: string[]): void
}>()

const isTauriContext = ref(typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window)

async function addPath() {
  if (!isTauriContext.value) return
  const { open } = await import('@tauri-apps/plugin-dialog')
  const path = await open({ directory: true, multiple: false })
  if (!path || typeof path !== 'string') return
  if (props.modelValue.includes(path)) return
  const updated = [...props.modelValue, path]
  emit('update:modelValue', updated)
  emit('change', updated)
}

function removePath(index: number) {
  const updated = props.modelValue.filter((_, i) => i !== index)
  emit('update:modelValue', updated)
  emit('change', updated)
}
</script>
<template>
  <div class="path-list">
    <div v-for="(p, i) in modelValue" :key="p" class="path-row">
      <span class="path-text">{{ p }}</span>
      <button class="remove-btn" @click="removePath(i)" type="button">&#8722;</button>
    </div>
    <button class="add-btn" @click="addPath" type="button">+ Add folder</button>
  </div>
</template>
<style scoped>
.path-list { display: flex; flex-direction: column; gap: var(--spacing-xs); width: 100%; }
.path-row { display: flex; align-items: center; gap: var(--spacing-sm);
  background: var(--color-bg-darker); border-radius: var(--radius-sm);
  padding: var(--spacing-xs) var(--spacing-sm); }
.path-text { flex: 1; font-family: var(--font-mono); font-size: var(--font-size-xs);
  color: var(--color-text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.remove-btn { background: none; border: none; color: var(--color-text-muted);
  cursor: pointer; font-size: 16px; padding: 0 4px; line-height: 1;
  transition: color var(--duration-fast); }
.remove-btn:hover { color: var(--color-text); }
.add-btn { align-self: flex-start; background: none; border: 1px solid var(--color-border);
  color: var(--color-text-muted); font-size: var(--font-size-sm); cursor: pointer;
  padding: var(--spacing-xs) var(--spacing-sm); border-radius: var(--radius-sm);
  transition: border-color var(--duration-fast), color var(--duration-fast); }
.add-btn:hover { border-color: var(--color-accent); color: var(--color-text); }
</style>
