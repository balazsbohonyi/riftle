<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Button from './Button.vue'
import ShortcutForm from './ShortcutForm.vue'
import ShortcutReadOnlyEntry from './ShortcutReadOnlyEntry.vue'

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

const props = withDefaults(defineProps<{
  modelValue: ShortcutEntry[]
  siblingShortcuts?: ShortcutEntry[]
  mode: ShortcutMode
  label: string
}>(), {
  siblingShortcuts: () => [],
})

const emit = defineEmits<{
  (e: 'update:modelValue', v: ShortcutEntry[]): void
  (e: 'change', v: ShortcutEntry[]): void
}>()

const isTauriContext = ref(typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window)
const newDraft = ref<ShortcutEntry | null>(null)
const editIndex = ref<number | null>(null)
const editDraft = ref<ShortcutEntry | null>(null)
const formError = ref<string | null>(null)

function blankEntry(path = ''): ShortcutEntry {
  if (props.mode === 'file') {
    return { path, parameters: '', alias: '' }
  }
  return { path, alias: '' }
}

function cloneEntry(entry: ShortcutEntry): ShortcutEntry {
  if (props.mode === 'file') {
    const file = entry as FileShortcut
    return { path: file.path, parameters: file.parameters, alias: file.alias }
  }
  return { path: entry.path, alias: entry.alias }
}

function updateEntries(updated: ShortcutEntry[]) {
  emit('update:modelValue', updated)
  emit('change', updated)
}

function pathName(path: string, stripExtension: boolean): string {
  const trimmed = path.trim().replace(/[\\/]+$/, '')
  const last = trimmed.split(/[\\/]/).filter(Boolean).pop() ?? ''
  if (!stripExtension) return last
  const dot = last.lastIndexOf('.')
  return dot > 0 ? last.slice(0, dot) : last
}

function shortcutName(entry: ShortcutEntry, mode: ShortcutMode): string {
  return (entry.alias.trim() || pathName(entry.path, mode === 'file')).trim()
}

function isParameterizedExecutableTarget(path: string): boolean {
  const extension = path.trim().split(/[\\/]/).pop()?.split('.').pop()?.toLowerCase()
  return extension !== undefined && ['exe', 'com', 'bat', 'cmd'].includes(extension)
}

async function validateDraft(draft: ShortcutEntry, originalIndex: number | null): Promise<string | null> {
  if (!draft.path.trim()) {
    return props.mode === 'directory' ? 'Directory shortcut path is required.' : 'File shortcut path is required.'
  }

  if (isTauriContext.value) {
    const exists = await invoke<boolean>('shortcut_target_exists', {
      path: draft.path,
      directory: props.mode === 'directory',
    }).catch(() => false)
    if (!exists) {
      return props.mode === 'directory' ? 'Directory shortcut path does not exist.' : 'File shortcut path does not exist.'
    }
  }

  const pathKey = draft.path.trim().toLowerCase()
  const duplicatePath = props.modelValue.some((entry, index) => (
    index !== originalIndex && entry.path.trim().toLowerCase() === pathKey
  ))
  if (duplicatePath) return 'Shortcut path already exists.'

  if (props.mode === 'file') {
    const file = draft as FileShortcut
    const hasParameters = file.parameters.trim().length > 0
    if (hasParameters && !isParameterizedExecutableTarget(file.path)) {
      return 'Parameters are only supported for .exe, .com, .bat, and .cmd files.'
    }
    if (hasParameters && !file.alias.trim()) {
      return 'An alias is required when file parameters are set.'
    }
  }

  const name = shortcutName(draft, props.mode).toLowerCase()
  if (!name) return 'Shortcut name is required.'

  const currentNames = props.modelValue
    .filter((_, index) => index !== originalIndex)
    .map((entry) => shortcutName(entry, props.mode).toLowerCase())
  const siblingNames = props.siblingShortcuts.map((entry) => (
    shortcutName(entry, props.mode === 'file' ? 'directory' : 'file').toLowerCase()
  ))
  if ([...currentNames, ...siblingNames].includes(name)) {
    return `Duplicate shortcut name: ${shortcutName(draft, props.mode)}`
  }

  return null
}

function clearError() {
  formError.value = null
}

async function addShortcut() {
  if (!isTauriContext.value) return
  const { open } = await import('@tauri-apps/plugin-dialog')
  const path = await open({
    directory: props.mode === 'directory',
    multiple: false,
  })
  if (!path || typeof path !== 'string') return

  editIndex.value = null
  editDraft.value = null
  newDraft.value = blankEntry(path)
  formError.value = null
}

function startEdit(index: number) {
  newDraft.value = null
  editIndex.value = index
  editDraft.value = cloneEntry(props.modelValue[index])
  formError.value = null
}

function cancelForm() {
  newDraft.value = null
  editIndex.value = null
  editDraft.value = null
  formError.value = null
}

async function saveNew() {
  if (!newDraft.value) return
  const error = await validateDraft(newDraft.value, null)
  if (error) {
    formError.value = error
    return
  }

  updateEntries([cloneEntry(newDraft.value), ...props.modelValue])
  cancelForm()
}

async function saveEdit() {
  if (editIndex.value === null || !editDraft.value) return
  const error = await validateDraft(editDraft.value, editIndex.value)
  if (error) {
    formError.value = error
    return
  }

  const updated = props.modelValue.map((entry, index) => (
    index === editIndex.value ? cloneEntry(editDraft.value as ShortcutEntry) : entry
  ))
  updateEntries(updated)
  cancelForm()
}

function removeEntry(index: number) {
  updateEntries(props.modelValue.filter((_, i) => i !== index))
  if (editIndex.value === index) cancelForm()
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
    <p v-if="mode === 'file'" class="shortcut-note">
      If a file does not open reliably on its own, point the shortcut at the app associated with it, and put the file path in Parameters.
    </p>

    <ShortcutForm
      v-if="newDraft"
      v-model="newDraft"
      :mode="mode"
      :error="formError"
      @update:modelValue="clearError"
      @save="saveNew"
      @cancel="cancelForm"
    />

    <template v-for="(entry, i) in modelValue" :key="`${entry.path}-${i}`">
      <ShortcutForm
        v-if="editIndex === i && editDraft"
        v-model="editDraft"
        :mode="mode"
        :error="formError"
        @update:modelValue="clearError"
        @save="saveEdit"
        @cancel="cancelForm"
      />
      <ShortcutReadOnlyEntry
        v-else
        :entry="entry"
        :mode="mode"
        @edit="startEdit(i)"
        @remove="removeEntry(i)"
      />
    </template>
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

.shortcut-note {
  margin: calc(-1 * var(--spacing-xs)) 0 var(--spacing-xs);
  color: var(--color-text-muted);
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  line-height: 1.4;
}
</style>
