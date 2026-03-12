<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue'
const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{
  (e: 'update:modelValue', v: string): void
  (e: 'change', v: string): void
  (e: 'capture-start'): void
  (e: 'capture-end'): void
}>()
const capturing = ref(false)
const displayValue = ref(props.modelValue)
const rootRef = ref<HTMLElement | null>(null)

watch(() => props.modelValue, v => { if (!capturing.value) displayValue.value = v })

function startCapture() {
  if (capturing.value) return
  capturing.value = true
  displayValue.value = 'Press shortcut…'
  emit('capture-start')
}

function stopCapture(reset = true) {
  if (capturing.value) {
    emit('capture-end')
  }
  capturing.value = false
  if (reset) {
    displayValue.value = props.modelValue
  }
}

function finishCapture(hotkey: string) {
  if (capturing.value) {
    emit('capture-end')
  }
  capturing.value = false
  displayValue.value = hotkey
  emit('update:modelValue', hotkey)
  emit('change', hotkey)
}

function formatHotkey(e: KeyboardEvent): string | null {
  const modKeys = ['Control', 'Alt', 'Shift', 'Meta']
  if (modKeys.includes(e.key)) return null

  const mods: string[] = []
  if (e.ctrlKey) mods.push('Ctrl')
  if (e.altKey) mods.push('Alt')
  if (e.shiftKey) mods.push('Shift')
  if (e.metaKey) mods.push('Meta')

  const keyName = e.code === 'Space' || e.key === ' ' || e.key === 'Spacebar'
    ? 'Space'
    : e.key

  return [...mods, keyName].join('+')
}

function onKeyDown(e: KeyboardEvent) {
  if (!capturing.value) return
  e.preventDefault()
  e.stopPropagation()
  if (e.key === 'Escape') {
    stopCapture()
    return
  }

  const hotkey = formatHotkey(e)
  if (!hotkey) return
  finishCapture(hotkey)
}

function onKeyUp(e: KeyboardEvent) {
  if (!capturing.value) return
  if (e.code !== 'Space' && e.key !== ' ' && e.key !== 'Spacebar') return
  if (!e.altKey) return

  e.preventDefault()
  e.stopPropagation()
  finishCapture('Alt+Space')
}

function onWindowPointerDown(e: MouseEvent) {
  if (!capturing.value) return
  if (rootRef.value?.contains(e.target as Node)) return
  stopCapture()
}

function onWindowKeyDown(event: Event) {
  onKeyDown(event as KeyboardEvent)
}

function onWindowKeyUp(event: Event) {
  onKeyUp(event as KeyboardEvent)
}

function onWindowMouseDown(event: Event) {
  onWindowPointerDown(event as MouseEvent)
}

watch(capturing, (active) => {
  if (typeof window === 'undefined') return
  const method = active ? 'addEventListener' : 'removeEventListener'
  window[method]('keydown', onWindowKeyDown, true)
  window[method]('keyup', onWindowKeyUp, true)
  window[method]('mousedown', onWindowMouseDown, true)
})

onUnmounted(() => {
  if (typeof window === 'undefined') return
  window.removeEventListener('keydown', onWindowKeyDown, true)
  window.removeEventListener('keyup', onWindowKeyUp, true)
  window.removeEventListener('mousedown', onWindowMouseDown, true)
})
</script>
<template>
  <div ref="rootRef" class="key-capture" :class="{ capturing }" @click="startCapture"
    @keydown.prevent="onKeyDown" tabindex="0">
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
