<script setup lang="ts">
import { ref, watch } from 'vue'
const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{
  (e: 'update:modelValue', v: string): void
  (e: 'change', v: string): void
}>()
const capturing = ref(false)
const displayValue = ref(props.modelValue)

watch(() => props.modelValue, v => { if (!capturing.value) displayValue.value = v })

function startCapture() { capturing.value = true; displayValue.value = 'Press shortcut…' }

function onKeyDown(e: KeyboardEvent) {
  if (!capturing.value) return
  e.preventDefault()
  if (e.key === 'Escape') {
    capturing.value = false; displayValue.value = props.modelValue; return
  }
  const modKeys = ['Control', 'Alt', 'Shift', 'Meta']
  if (modKeys.includes(e.key)) return
  const mods: string[] = []
  if (e.ctrlKey) mods.push('Ctrl')
  if (e.altKey) mods.push('Alt')
  if (e.shiftKey) mods.push('Shift')
  if (e.metaKey) mods.push('Meta')
  const keyName = e.key === ' ' ? 'Space' : e.key
  const hotkey = [...mods, keyName].join('+')
  capturing.value = false
  displayValue.value = hotkey
  emit('update:modelValue', hotkey)
  emit('change', hotkey)
}

function onBlur() { if (capturing.value) { capturing.value = false; displayValue.value = props.modelValue } }
</script>
<template>
  <div class="key-capture" :class="{ capturing }" @click="startCapture"
    @keydown="onKeyDown" @blur="onBlur" tabindex="0">
    {{ displayValue }}
  </div>
</template>
<style scoped>
.key-capture { font-family: var(--font-mono); font-size: var(--font-size-sm);
  padding: var(--spacing-xs) var(--spacing-sm); border-radius: var(--radius-sm);
  border: 1px solid var(--color-border); background: var(--color-bg-darker);
  color: var(--color-text); cursor: pointer; min-width: 140px; text-align: center;
  transition: border-color var(--duration-fast); }
.key-capture.capturing { border-color: var(--color-accent); color: var(--color-text-muted); }
.key-capture:focus { outline: none; border-color: var(--color-accent); }
</style>
