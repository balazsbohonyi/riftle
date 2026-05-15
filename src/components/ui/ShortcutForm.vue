<script setup lang="ts">
import Button from './Button.vue'
import type { DirectoryShortcut, FileShortcut } from './ShortcutList.vue'

type ShortcutMode = 'directory' | 'file'
type ShortcutEntry = DirectoryShortcut | FileShortcut

const props = defineProps<{
  modelValue: ShortcutEntry
  mode: ShortcutMode
  error?: string | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', v: ShortcutEntry): void
  (e: 'save'): void
  (e: 'cancel'): void
}>()

function updateEntry(patch: Partial<FileShortcut>) {
  emit('update:modelValue', { ...props.modelValue, ...patch } as ShortcutEntry)
}

async function browsePath() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const path = await open({
    directory: props.mode === 'directory',
    multiple: false,
  })
  if (!path || typeof path !== 'string') return
  updateEntry({ path })
}
</script>

<template>
  <div class="shortcut-form" @keydown.esc.stop.prevent="emit('cancel')">
    <label class="shortcut-field">
      <span class="shortcut-field-label">Path</span>
      <span class="path-field">
        <input
          class="shortcut-input shortcut-input--path"
          :value="modelValue.path"
          type="text"
          placeholder="Path"
          spellcheck="false"
          @input="updateEntry({ path: ($event.target as HTMLInputElement).value })"
        />
        <button class="browse-btn" type="button" @click="browsePath">Browse</button>
      </span>
    </label>

    <label v-if="mode === 'file'" class="shortcut-field">
      <span class="shortcut-field-label">Parameters</span>
      <input
        class="shortcut-input"
        :value="(modelValue as FileShortcut).parameters"
        type="text"
        placeholder="Parameters"
        spellcheck="false"
        @input="updateEntry({ parameters: ($event.target as HTMLInputElement).value })"
      />
    </label>

    <label class="shortcut-field">
      <span class="shortcut-field-label">Alias</span>
      <input
        class="shortcut-input"
        :value="modelValue.alias"
        type="text"
        placeholder="Alias"
        spellcheck="false"
        @input="updateEntry({ alias: ($event.target as HTMLInputElement).value })"
      />
    </label>

    <p v-if="error" class="shortcut-form-error">{{ error }}</p>

    <div class="shortcut-form-actions">
      <Button variant="default" @click="emit('cancel')">Cancel</Button>
      <Button variant="accent" @click="emit('save')">Save</Button>
    </div>
  </div>
</template>

<style scoped>
.shortcut-form {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  background: var(--color-bg-darker);
  border-radius: var(--radius-sm);
  padding: var(--spacing-sm);
  width: 100%;
  box-sizing: border-box;
}

.shortcut-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.shortcut-field-label {
  color: var(--color-text-muted);
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
}

.path-field {
  position: relative;
  display: block;
  min-width: 0;
}

.shortcut-input {
  box-sizing: border-box;
  width: 100%;
  min-width: 0;
  height: 30px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-bg);
  color: var(--color-text);
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  padding: 0 var(--spacing-sm);
  outline: none;
}

.shortcut-input--path {
  padding-right: 72px;
}

.shortcut-input:focus {
  border-color: var(--color-accent);
}

.browse-btn {
  position: absolute;
  top: 4px;
  right: 4px;
  height: 22px;
  border: 0;
  border-left: 1px solid var(--color-border);
  background: var(--color-bg);
  color: var(--color-text-muted);
  cursor: pointer;
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  padding: 0 var(--spacing-xs);
}

.browse-btn:hover {
  color: var(--color-text);
}

.shortcut-form-error {
  color: var(--color-danger);
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  margin: 0;
}

.shortcut-form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-xs);
  margin-top: var(--spacing-xs);
}
</style>
