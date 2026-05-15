<script setup lang="ts">
import { ref } from 'vue'
import Button from './Button.vue'

export interface DirectoryShortcut {
  path: string
  alias: string
}

export interface FileShortcut {
  path: string
  parameters: string
  alias: string
}

type ShortcutMode = 'directory' | 'file'
type ShortcutEntry = DirectoryShortcut | FileShortcut

const props = defineProps<{
  modelValue: ShortcutEntry[]
  mode: ShortcutMode
  label: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', v: ShortcutEntry[]): void
  (e: 'change', v: ShortcutEntry[]): void
}>()

const isTauriContext = ref(typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window)

function blankEntry(path = ''): ShortcutEntry {
  if (props.mode === 'file') {
    return { path, parameters: '', alias: '' }
  }
  return { path, alias: '' }
}

function updateEntries(updated: ShortcutEntry[]) {
  emit('update:modelValue', updated)
  emit('change', updated)
}

async function addShortcut() {
  if (!isTauriContext.value) return
  const { open } = await import('@tauri-apps/plugin-dialog')
  const path = await open({
    directory: props.mode === 'directory',
    multiple: false,
  })
  if (!path || typeof path !== 'string') return
  if (props.modelValue.some((entry) => entry.path === path)) return
  updateEntries([...props.modelValue, blankEntry(path)])
}

function updateEntry(index: number, patch: Partial<FileShortcut>) {
  const updated = props.modelValue.map((entry, i) => (
    i === index ? { ...entry, ...patch } : entry
  ))
  updateEntries(updated)
}

function removeEntry(index: number) {
  updateEntries(props.modelValue.filter((_, i) => i !== index))
}
</script>

<template>
  <div class="shortcut-list">
    <div class="shortcut-list-header">
      <span class="shortcut-list-label">{{ label }}</span>
      <Button variant="default" @click="addShortcut">
        {{ mode === 'directory' ? '+ Add folder' : '+ Add file' }}
      </Button>
    </div>

    <div v-for="(entry, i) in modelValue" :key="`${entry.path}-${i}`" class="shortcut-row">
      <div class="shortcut-fields">
        <input
          class="shortcut-input shortcut-input--path"
          :value="entry.path"
          type="text"
          placeholder="Path"
          spellcheck="false"
          @input="updateEntry(i, { path: ($event.target as HTMLInputElement).value })"
        />
        <input
          v-if="mode === 'file'"
          class="shortcut-input"
          :value="(entry as FileShortcut).parameters"
          type="text"
          placeholder="Parameters"
          spellcheck="false"
          @input="updateEntry(i, { parameters: ($event.target as HTMLInputElement).value })"
        />
        <input
          class="shortcut-input"
          :value="entry.alias"
          type="text"
          placeholder="Alias"
          spellcheck="false"
          @input="updateEntry(i, { alias: ($event.target as HTMLInputElement).value })"
        />
      </div>
      <button class="remove-btn" type="button" @click="removeEntry(i)">&#8722;</button>
    </div>
  </div>
</template>

<style scoped>
.shortcut-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  width: 100%;
  padding: var(--spacing-sm) 0;
  border-bottom: 1px solid var(--color-divider);
}

.shortcut-list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 40px;
}

.shortcut-list-label {
  font-family: var(--font-sans);
  font-size: var(--font-size-base);
  color: var(--color-text);
}

.shortcut-row {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-sm);
  background: var(--color-bg-darker);
  border-radius: var(--radius-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  width: 100%;
  box-sizing: border-box;
}

.shortcut-fields {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(0, 0.85fr) minmax(0, 0.65fr);
  gap: var(--spacing-xs);
  flex: 1;
  min-width: 0;
}

.shortcut-input {
  min-width: 0;
  height: 28px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-bg);
  color: var(--color-text);
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  padding: 0 var(--spacing-sm);
  outline: none;
}

.shortcut-input--path {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
}

.shortcut-input:focus {
  border-color: var(--color-accent);
}

.remove-btn {
  flex-shrink: 0;
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: 16px;
  padding: 6px 4px 0;
  line-height: 1;
  transition: color var(--duration-fast);
}

.remove-btn:hover {
  color: var(--color-text);
}

@media (max-width: 520px) {
  .shortcut-fields {
    grid-template-columns: 1fr;
  }
}
</style>
