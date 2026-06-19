<script setup lang="ts">
import type { DirectoryShortcut, FileShortcut } from './ShortcutList.vue'

type ShortcutMode = 'directory' | 'file'
type ShortcutEntry = DirectoryShortcut | FileShortcut

const props = defineProps<{
  entry: ShortcutEntry
  mode: ShortcutMode
}>()

const emit = defineEmits<{
  (e: 'edit'): void
  (e: 'remove'): void
}>()

function pathName(path: string, stripExtension: boolean): string {
  const trimmed = path.trim().replace(/[\\/]+$/, '')
  const last = trimmed.split(/[\\/]/).filter(Boolean).pop() ?? ''
  if (!stripExtension) return last
  const dot = last.lastIndexOf('.')
  return dot > 0 ? last.slice(0, dot) : last
}

function displayName(): string {
  return props.entry.alias.trim() || pathName(props.entry.path, props.mode === 'file') || props.entry.path
}

function compactMiddle(value: string, maxLength = 76): string {
  if (value.length <= maxLength) return value
  const keep = Math.floor((maxLength - 3) / 2)
  return `${value.slice(0, keep)}...${value.slice(value.length - keep)}`
}
</script>

<template>
  <div class="shortcut-entry">
    <div class="shortcut-entry-top">
      <span class="shortcut-entry-name" :title="displayName()">{{ displayName() }}</span>
      <div class="shortcut-entry-actions">
        <button class="icon-btn" type="button" title="Edit shortcut" @click="emit('edit')">&#9998;</button>
        <button class="icon-btn" type="button" title="Remove shortcut" @click="emit('remove')">&#8722;</button>
      </div>
    </div>
    <div class="shortcut-entry-path" :title="entry.path">{{ compactMiddle(entry.path) }}</div>
    <div
      v-if="mode === 'file' && (entry as FileShortcut).parameters.trim()"
      class="shortcut-entry-parameters"
      :title="(entry as FileShortcut).parameters"
    >
      {{ compactMiddle((entry as FileShortcut).parameters, 88) }}
    </div>
  </div>
</template>

<style scoped>
.shortcut-entry {
  display: flex;
  flex-direction: column;
  gap: 6px;
  border-bottom: 1px solid var(--color-divider);
  border-radius: var(--radius-sm);
  padding: var(--spacing-md) 0;
  width: 100%;
  box-sizing: border-box;
}

.shortcut-entry-top {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.shortcut-entry-name {
  min-width: 0;
  max-width: 100%;
  background: var(--color-bg-darker);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  color: var(--color-text);
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  font-weight: 500;
  line-height: 1.2;
  padding: 3px var(--spacing-xs);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.shortcut-entry-actions {
  display: flex;
  gap: 2px;
  margin-left: auto;
  flex-shrink: 0;
  opacity: 0;
  pointer-events: none;
  transition: opacity var(--duration-fast);
}

.shortcut-entry:hover .shortcut-entry-actions,
.shortcut-entry:focus-within .shortcut-entry-actions {
  opacity: 1;
  pointer-events: auto;
}

.icon-btn {
  flex-shrink: 0;
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: 15px;
  padding: 2px 4px;
  line-height: 1;
  transition: color var(--duration-fast);
}

.icon-btn:hover {
  color: var(--color-text);
}

.shortcut-entry-path,
.shortcut-entry-parameters {
  min-width: 0;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.shortcut-entry-parameters {
  font-family: var(--font-mono);
}
</style>
