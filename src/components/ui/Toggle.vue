<script setup lang="ts">
const props = defineProps<{ modelValue: boolean; disabled?: boolean }>()
const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>()
function toggle() { if (!props.disabled) emit('update:modelValue', !props.modelValue) }
</script>
<template>
  <button role="switch" :aria-checked="modelValue" :aria-disabled="disabled"
    class="toggle" :class="{ on: modelValue, disabled }" @click="toggle"
    type="button">
    <span class="toggle-thumb" />
  </button>
</template>
<style scoped>
.toggle { width: 40px; height: 22px; border-radius: 11px; border: none; cursor: pointer;
  background: var(--color-text-dim); position: relative; transition: background var(--duration-fast);
  padding: 0; flex-shrink: 0; }
.toggle.on { background: var(--color-accent); }
.toggle.disabled { opacity: 0.4; cursor: not-allowed; }
.toggle-thumb { position: absolute; top: 3px; left: 3px; width: 16px; height: 16px;
  border-radius: 50%; background: var(--color-text); transition: left var(--duration-fast); }
.toggle.on .toggle-thumb { left: 21px; }
</style>
