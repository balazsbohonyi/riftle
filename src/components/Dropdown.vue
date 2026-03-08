<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

interface Option {
  value: any
  label: string
}

const props = defineProps<{
  modelValue: any
  options: Option[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: any]
}>()

const isOpen = ref(false)
const highlightedIndex = ref(-1)
const rootEl = ref<HTMLElement | null>(null)

const currentLabel = computed(() => {
  const found = props.options.find(o => o.value === props.modelValue)
  return found ? found.label : ''
})

function toggleDropdown() {
  if (isOpen.value) {
    isOpen.value = false
    highlightedIndex.value = -1
  } else {
    const idx = props.options.findIndex(o => o.value === props.modelValue)
    highlightedIndex.value = idx >= 0 ? idx : 0
    isOpen.value = true
  }
}

function selectOption(value: any) {
  emit('update:modelValue', value)
  isOpen.value = false
  highlightedIndex.value = -1
}

function onRootKeydown(e: KeyboardEvent) {
  if (!isOpen.value) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      e.stopPropagation()
      const idx = props.options.findIndex(o => o.value === props.modelValue)
      highlightedIndex.value = idx >= 0 ? idx : 0
      isOpen.value = true
    }
    return
  }

  if (e.key === 'ArrowDown') {
    e.preventDefault()
    e.stopPropagation()
    highlightedIndex.value = Math.min(highlightedIndex.value + 1, props.options.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    e.stopPropagation()
    highlightedIndex.value = Math.max(highlightedIndex.value - 1, 0)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    e.stopPropagation()
    if (highlightedIndex.value >= 0) {
      selectOption(props.options[highlightedIndex.value].value)
    }
  } else if (e.key === 'Escape') {
    e.preventDefault()
    e.stopPropagation()
    isOpen.value = false
    highlightedIndex.value = -1
  }
}

function onDocumentClick(e: MouseEvent) {
  if (!rootEl.value?.contains(e.target as Node)) {
    isOpen.value = false
    highlightedIndex.value = -1
  }
}

onMounted(() => {
  document.addEventListener('click', onDocumentClick)
})

onUnmounted(() => {
  document.removeEventListener('click', onDocumentClick)
})
</script>

<template>
  <div
    ref="rootEl"
    class="custom-select"
    :class="{ open: isOpen }"
    @keydown="onRootKeydown"
  >
    <button
      type="button"
      class="custom-select-trigger"
      @click.stop="toggleDropdown"
    >
      <span>{{ currentLabel }}</span>
      <span class="custom-select-arrow">&#9660;</span>
    </button>
    <div class="custom-select-dropdown" v-if="isOpen" @keydown.stop>
      <div
        v-for="(opt, index) in options"
        :key="opt.value"
        class="custom-select-option"
        :class="{ highlighted: index === highlightedIndex }"
        @click.stop="selectOption(opt.value)"
        @mouseenter="highlightedIndex = index"
      >{{ opt.label }}</div>
    </div>
  </div>
</template>

<style scoped>
.custom-select {
  position: relative;
  display: inline-block;
  min-width: 120px;
}

.custom-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-sm);
  width: 100%;
  background: var(--color-bg-darker);
  border: 1px solid var(--color-border);
  color: var(--color-text);
  border-radius: var(--radius-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  cursor: pointer;
}

.custom-select-trigger:focus {
  outline: none;
  border-color: var(--color-accent);
}

.custom-select-arrow {
  font-size: 10px;
  opacity: 0.6;
  pointer-events: none;
  transition: transform var(--duration-fast);
}

.custom-select.open .custom-select-arrow {
  transform: rotate(180deg);
}

.custom-select-dropdown {
  position: absolute;
  top: calc(100% + 2px);
  left: 0;
  right: 0;
  background: var(--color-bg-darker);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  overflow: hidden;
  z-index: 100;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.custom-select-option {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: var(--font-size-sm);
  font-family: var(--font-sans);
  cursor: pointer;
  color: var(--color-text);
  transition: background var(--duration-fast);
}

.custom-select-option:hover {
  background: var(--color-accent);
  color: #ffffff;
}

.custom-select-option.highlighted {
  background: var(--color-accent);
  color: #ffffff;
}

</style>
