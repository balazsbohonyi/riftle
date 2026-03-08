<script setup lang="ts">
defineOptions({ inheritAttrs: false })

withDefaults(defineProps<{
  variant?: 'default' | 'accent'
  type?: 'button' | 'submit' | 'reset'
}>(), {
  variant: 'default',
  type: 'button',
})
</script>

<template>
  <button v-bind="$attrs" :type="type" :class="['btn', `btn--${variant}`]">
    <slot />
  </button>
</template>

<style scoped>
.btn {
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  border-radius: var(--radius-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  cursor: pointer;
  transition: opacity var(--duration-fast), border-color var(--duration-fast);
}

.btn:focus {
  outline: none;
}

.btn--default {
  background: var(--color-bg-darker);
  border: 1px solid var(--color-border);
  color: var(--color-text-muted);
}

.btn--default:hover {
  border-color: var(--color-accent);
  color: var(--color-text);
}

.btn--accent {
  background: var(--color-accent);
  border: none;
  color: #ffffff;
}

.btn--accent:hover {
  opacity: 0.88;
}

.btn--accent:focus {
  outline: none;
}
</style>
