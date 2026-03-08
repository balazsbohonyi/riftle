<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { emitTo } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import Section from './components/ui/Section.vue'
import Row from './components/ui/Row.vue'
import Toggle from './components/ui/Toggle.vue'
import KeyCapture from './components/ui/KeyCapture.vue'
import PathList from './components/ui/PathList.vue'
import Dropdown from './components/Dropdown.vue'

interface SettingsData {
  hotkey: string
  theme: string

  show_path: boolean
  autostart: boolean
  additional_paths: string[]
  excluded_paths: string[]
  reindex_interval: number
  animation: string
  system_tool_allowlist: string[]
}

interface SettingsResponse extends SettingsData {
  data_dir: string
  is_portable: boolean
}

const isTauriContext = ref(typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window)
const isPortable = ref(false)
const reindexButtonText = ref('Re-index')

const settings = ref<SettingsData>({
  hotkey: 'Alt+Space',
  theme: 'system',

  show_path: false,
  autostart: false,
  additional_paths: [],
  excluded_paths: [],
  reindex_interval: 15,
  animation: 'slide',
  system_tool_allowlist: [],
})

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown)
  if (!isTauriContext.value) return
  try {
    const response = await invoke<SettingsResponse>('get_settings_cmd')
    isPortable.value = response.is_portable
    settings.value = {
      hotkey: response.hotkey,
      theme: response.theme,

      show_path: response.show_path,
      autostart: response.autostart,
      additional_paths: response.additional_paths,
      excluded_paths: response.excluded_paths,
      reindex_interval: response.reindex_interval,
      animation: response.animation,
      system_tool_allowlist: response.system_tool_allowlist,
    }

    if (!isPortable.value) {
      const { isEnabled } = await import('@tauri-apps/plugin-autostart')
      settings.value.autostart = await isEnabled()
    }

    if (settings.value.theme !== 'system') {
      document.documentElement.setAttribute('data-theme', settings.value.theme)
    }
  } catch (e) {
    console.error('Failed to load settings:', e)
  }
})

async function saveSettings() {
  if (!isTauriContext.value) return
  await invoke('set_settings_cmd', { settings: { ...settings.value } }).catch(console.error)
}

// General
async function onAutostartChange(v: boolean) {
  if (isPortable.value) return
  try {
    const { enable, disable } = await import('@tauri-apps/plugin-autostart')
    if (v) {
      await enable()
    } else {
      await disable()
    }
  } catch (e) {
    console.error('Autostart change failed:', e)
  }
  await saveSettings()
}

// Hotkey
async function onHotkeyChange(hotkey: string) {
  try {
    await invoke('update_hotkey', { hotkey })
    settings.value.hotkey = hotkey
  } catch (e: any) {
    // Rust fell back to a different hotkey (e.g. OS-reserved key)
    // Extract the fallback from the error message and reload from backend
    console.warn('Hotkey update error:', e)
    const fresh = await invoke<SettingsResponse>('get_settings_cmd').catch(() => null)
    if (fresh) settings.value.hotkey = fresh.hotkey
  }
  await saveSettings()
}

// Search
async function onPathsChange(field: 'additional_paths' | 'excluded_paths', paths: string[]) {
  settings.value[field] = paths
  await saveSettings()
  await invoke('reindex').catch(console.error)
}

async function onIntervalChange(val: number) {
  settings.value.reindex_interval = val
  await saveSettings()
  await emitTo('launcher', 'settings-changed', { reindex_interval: val }).catch(console.error)
}

async function onReindexNow() {
  reindexButtonText.value = 'Indexing\u2026'
  await invoke('reindex').catch(console.error)
  setTimeout(() => { reindexButtonText.value = 'Re-index' }, 1000)
}

// Appearance
async function onThemeChange() {
  const theme = settings.value.theme
  if (theme === 'system') {
    document.documentElement.removeAttribute('data-theme')
  } else {
    document.documentElement.setAttribute('data-theme', theme)
  }
  await saveSettings()
  await emitTo('launcher', 'settings-changed', { theme }).catch(console.error)
}


async function onShowPathChange(v: boolean) {
  settings.value.show_path = v
  await saveSettings()
  await emitTo('launcher', 'settings-changed', { show_path: v }).catch(console.error)
}

async function closeWindow() {
  await emitTo('launcher', 'launcher-show').catch(console.error)
  await getCurrentWindow().hide()
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') closeWindow()
}

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
})
</script>

<template>
  <div class="settings-app" :data-theme="settings.theme === 'system' ? undefined : settings.theme">
    <div class="settings-header" data-tauri-drag-region>
      <span class="settings-title" data-tauri-drag-region>Riftle Settings</span>
      <button class="settings-close" type="button" @mousedown.stop @click="closeWindow">&times;</button>
    </div>
    <div class="settings-content">

      <Section title="General">
        <Row
          label="Launch at startup"
          :hint="isPortable ? 'Not available in portable mode' : undefined"
        >
          <Toggle
            v-model="settings.autostart"
            :disabled="isPortable"
            @update:modelValue="onAutostartChange"
          />
        </Row>
      </Section>

      <Section title="Hotkey">
        <Row label="Global shortcut">
          <div class="hotkey-row">
            <button
              v-if="settings.hotkey !== 'Alt+Space'"
              type="button"
              class="reset-link"
              @click="onHotkeyChange('Alt+Space')"
            >Reset</button>
            <KeyCapture v-model="settings.hotkey" @change="onHotkeyChange" />
          </div>
        </Row>
      </Section>

      <Section title="Search">
        <PathList
          label="Additional paths"
          v-model="settings.additional_paths"
          @change="onPathsChange('additional_paths', $event)"
        />
        <PathList
          label="Excluded paths"
          v-model="settings.excluded_paths"
          @change="onPathsChange('excluded_paths', $event)"
        />
        <Row label="Re-index interval">
          <Dropdown
            :options="[{ value: 5, label: '5 min' }, { value: 15, label: '15 min' }, { value: 30, label: '30 min' }, { value: 60, label: '60 min' }, { value: 0, label: 'Manual only' }]"
            v-model="settings.reindex_interval"
            @update:modelValue="onIntervalChange"
          />
        </Row>
        <Row label="Re-index now">
          <button type="button" @click="onReindexNow">{{ reindexButtonText }}</button>
        </Row>
      </Section>

      <Section title="Appearance">
        <Row label="Theme">
          <Dropdown
            :options="[{ value: 'system', label: 'System' }, { value: 'light', label: 'Light' }, { value: 'dark', label: 'Dark' }]"
            v-model="settings.theme"
            @update:modelValue="onThemeChange"
          />
        </Row>

        <Row label="Show path">
          <Toggle v-model="settings.show_path" @update:modelValue="onShowPathChange" />
        </Row>
      </Section>

    </div>
  </div>
</template>

<style scoped>
.settings-app {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-bg);
  color: var(--color-text);
  font-family: var(--font-sans);
  font-size: var(--font-size-base);
}

.settings-header {
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--spacing-md);
  border-bottom: 1px solid var(--color-divider);
  flex-shrink: 0;
  user-select: none;
}

.settings-title {
  font-size: var(--font-size-base);
  font-weight: 500;
  color: var(--color-text);
}

.settings-close {
  background: none;
  border: none;
  color: var(--color-text-muted);
  font-size: 18px;
  cursor: pointer;
  padding: var(--spacing-xs);
  border-radius: var(--radius-sm);
  line-height: 1;
  transition: color var(--duration-fast);
}

.settings-close:hover {
  color: var(--color-text);
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-lg);
}

select,
button {
  background: var(--color-bg-darker);
  border: 1px solid var(--color-border);
  color: var(--color-text);
  border-radius: var(--radius-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  cursor: pointer;
}

select:focus,
button:focus {
  outline: none;
  border-color: var(--color-accent);
}

input[type='range'] {
  accent-color: var(--color-accent);
}

/* Custom scrollbar for settings content */
.settings-content::-webkit-scrollbar {
  width: 6px;
}

.settings-content::-webkit-scrollbar-track {
  background: transparent;
}

.settings-content::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 3px;
}

.settings-content::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-muted);
}

.settings-content::-webkit-scrollbar-button {
  display: none;
}

.hotkey-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.reset-link {
  background: none;
  border: none;
  color: var(--color-accent);
  font-size: var(--font-size-sm);
  cursor: pointer;
  padding: 0;
  text-decoration: none;
}

.reset-link:focus {
  outline: none;
  opacity: 0.8;
}

.reset-link:hover {
  opacity: 0.7;
}

</style>

<style>
html,
body,
#app {
  height: 100%;
  margin: 0;
}
</style>
