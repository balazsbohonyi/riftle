<script setup lang="ts">
import { computed, nextTick, ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { emitTo } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import Section from './components/ui/Section.vue'
import Row from './components/ui/Row.vue'
import Toggle from './components/ui/Toggle.vue'
import KeyCapture from './components/ui/KeyCapture.vue'
import PathList from './components/ui/PathList.vue'
import ShortcutList, { type DirectoryShortcut, type FileShortcut } from './components/ui/ShortcutList.vue'
import Dropdown from './components/Dropdown.vue'
import Button from './components/ui/Button.vue'

interface SettingsData {
  hotkey: string
  theme: string
  play_sound: boolean

  show_path: boolean
  pin_shortcuts_to_top: boolean
  follow_cursor: boolean          // NEW
  autostart: boolean
  additional_paths: string[]
  excluded_paths: string[]
  reindex_interval: number
  system_tool_allowlist: string[]
  directory_shortcuts: DirectoryShortcut[]
  file_shortcuts: FileShortcut[]
}

interface SettingsResponse extends SettingsData {
  data_dir: string
  is_portable: boolean
  build_profile: 'debug' | 'release'
  can_autostart: boolean
}

const isTauriContext = ref(typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window)
const isPortable = ref(false)
const buildProfile = ref<'debug' | 'release'>('debug')
const canAutostart = ref(false)
const reindexButtonText = ref('Re-index')
const hotkeyError = ref<string | null>(null)
const shortcutsError = ref<string | null>(null)
const defaultHotkey = 'Ctrl+Space'
const lastRegisteredHotkey = ref(defaultHotkey)
const activeShortcutTab = ref<'directory' | 'file'>('directory')
const directoryShortcutList = ref<InstanceType<typeof ShortcutList> | null>(null)
const fileShortcutList = ref<InstanceType<typeof ShortcutList> | null>(null)
const settingsContentRef = ref<HTMLElement | null>(null)
const hotkeyRowRef = ref<HTMLElement | null>(null)
const isSettingsScrolling = ref(false)
const settingsScrollThumbTop = ref(0)
const settingsScrollThumbHeight = ref(0)
let settingsScrollTimer: number | undefined
let unlistenConflictRef: (() => void) | undefined  // unused, kept for cleanup safety

const settingsScrollThumbStyle = computed(() => ({
  height: `${settingsScrollThumbHeight.value}px`,
  transform: `translateY(${settingsScrollThumbTop.value}px)`,
}))

const autostartHint = computed(() => {
  if (isPortable.value) return 'Not available in portable mode'
  if (buildProfile.value === 'debug') return 'Not available in dev/debug builds'
  return undefined
})

const settings = ref<SettingsData>({
  hotkey: defaultHotkey,
  theme: 'system',

  show_path: false,
  pin_shortcuts_to_top: false,
  follow_cursor: false,             // NEW
  play_sound: true,
  autostart: false,
  additional_paths: [],
  excluded_paths: [],
  reindex_interval: 15,
  system_tool_allowlist: [],
  directory_shortcuts: [],
  file_shortcuts: [],
})

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown)
  if (!isTauriContext.value) return
  try {
    const response = await invoke<SettingsResponse>('get_settings_cmd')
    isPortable.value = response.is_portable
    buildProfile.value = response.build_profile
    canAutostart.value = response.can_autostart
    settings.value = {
      hotkey: response.hotkey,
      theme: response.theme,

      show_path: response.show_path,
      pin_shortcuts_to_top: response.pin_shortcuts_to_top,
      follow_cursor: response.follow_cursor ?? false,
      play_sound: response.play_sound ?? true,
      autostart: response.can_autostart ? response.autostart : false,
      additional_paths: response.additional_paths,
      excluded_paths: response.excluded_paths,
      reindex_interval: response.reindex_interval,
      system_tool_allowlist: response.system_tool_allowlist,
      directory_shortcuts: response.directory_shortcuts ?? [],
      file_shortcuts: response.file_shortcuts ?? [],
    }
    lastRegisteredHotkey.value = response.hotkey

    if (canAutostart.value) {
      const { isEnabled } = await import('@tauri-apps/plugin-autostart')
      settings.value.autostart = await isEnabled()
    }

    if (settings.value.theme !== 'system') {
      document.documentElement.setAttribute('data-theme', settings.value.theme)
    }
  } catch (e) {
    console.error('Failed to load settings:', e)
  }

  // Pull any startup hotkey conflict from Rust managed state.
  // Event-based approach is unreliable — the webview loads after the setup() callback fires,
  // so events emitted during setup are lost before the listener is registered.
  try {
    const conflictKey = await invoke<string | null>('get_startup_hotkey_conflict')
    if (conflictKey) {
      hotkeyError.value = `'${conflictKey}' is already in use by another app — please set a different hotkey`
      nextTick(() => {
        hotkeyRowRef.value?.scrollIntoView({ behavior: 'smooth', block: 'center' })
      })
    }
  } catch (e) {
    console.error('Failed to check startup hotkey conflict:', e)
  }
})

async function saveSettings() {
  if (!isTauriContext.value) return
  const shortcutError = validateShortcutSettings()
  shortcutsError.value = shortcutError
  if (shortcutError) return
  await invoke('set_settings_cmd', { settings: { ...settings.value } }).catch(console.error)
}

async function setHotkeyCaptureActive(active: boolean) {
  if (!isTauriContext.value) return
  await invoke('set_hotkey_capture_active', { active }).catch(console.error)
}

// General
async function onAutostartChange(v: boolean) {
  settings.value.autostart = v
  if (!canAutostart.value) {
    settings.value.autostart = false
    return
  }
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
  const oldHotkey = lastRegisteredHotkey.value   // always the last SUCCESSFULLY registered key
  hotkeyError.value = null
  try {
    await invoke('update_hotkey', { hotkey })
    lastRegisteredHotkey.value = hotkey           // update only on success
    settings.value.hotkey = hotkey
    await saveSettings()
  } catch (e: any) {
    console.warn('Hotkey update failed:', e)
    const msg = typeof e === 'string' ? e : (e?.message ?? 'Could not register hotkey')
    hotkeyError.value = `${msg} — still using ${oldHotkey}`
    settings.value.hotkey = oldHotkey            // triggers KeyCapture prop watcher → reverts display
    // Do NOT call saveSettings() — backend hotkey is unchanged
  }
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

function pathName(path: string, stripExtension: boolean): string {
  const trimmed = path.trim().replace(/[\\/]+$/, '')
  const last = trimmed.split(/[\\/]/).filter(Boolean).pop() ?? ''
  if (!stripExtension) return last
  const dot = last.lastIndexOf('.')
  return dot > 0 ? last.slice(0, dot) : last
}

function shortcutName(path: string, alias: string, stripExtension: boolean): string {
  return (alias.trim() || pathName(path, stripExtension)).trim()
}

function isParameterizedExecutableTarget(path: string): boolean {
  const extension = path.trim().split(/[\\/]/).pop()?.split('.').pop()?.toLowerCase()
  return extension !== undefined && ['exe', 'com', 'bat', 'cmd'].includes(extension)
}

function validateShortcutSettings(): string | null {
  const names = new Set<string>()

  for (const shortcut of settings.value.directory_shortcuts) {
    if (!shortcut.path.trim()) return 'Directory shortcut path is required.'
    const name = shortcutName(shortcut.path, shortcut.alias, false).toLowerCase()
    if (!name || names.has(name)) return `Duplicate shortcut name: ${shortcutName(shortcut.path, shortcut.alias, false)}`
    names.add(name)
  }

  for (const shortcut of settings.value.file_shortcuts) {
    if (!shortcut.path.trim()) return 'File shortcut path is required.'
    const hasParameters = shortcut.parameters.trim().length > 0
    const canUseParameters = isParameterizedExecutableTarget(shortcut.path)
    if (hasParameters && !canUseParameters) {
      return 'Parameters are only supported for .exe, .com, .bat, and .cmd files.'
    }
    if (hasParameters && !shortcut.alias.trim()) {
      return 'An alias is required when file parameters are set.'
    }
    const name = shortcutName(shortcut.path, shortcut.alias, true).toLowerCase()
    if (!name || names.has(name)) return `Duplicate shortcut name: ${shortcutName(shortcut.path, shortcut.alias, true)}`
    names.add(name)
  }

  return null
}

async function onShortcutsChange(
  field: 'directory_shortcuts' | 'file_shortcuts',
  shortcuts: DirectoryShortcut[] | FileShortcut[],
) {
  if (field === 'directory_shortcuts') {
    settings.value.directory_shortcuts = shortcuts as DirectoryShortcut[]
  } else {
    settings.value.file_shortcuts = shortcuts as FileShortcut[]
  }
  await saveSettings()
}

async function addShortcutFromTab(mode: 'directory' | 'file') {
  activeShortcutTab.value = mode
  await nextTick()
  if (mode === 'directory') {
    await directoryShortcutList.value?.addShortcut()
  } else {
    await fileShortcutList.value?.addShortcut()
  }
}

function onSettingsScroll() {
  updateSettingsScrollThumb()
  isSettingsScrolling.value = true
  window.clearTimeout(settingsScrollTimer)
  settingsScrollTimer = window.setTimeout(() => {
    isSettingsScrolling.value = false
  }, 700)
}

function updateSettingsScrollThumb() {
  const el = settingsContentRef.value
  if (!el || el.scrollHeight <= el.clientHeight) {
    settingsScrollThumbHeight.value = 0
    settingsScrollThumbTop.value = 0
    return
  }

  const ratio = el.clientHeight / el.scrollHeight
  const thumbHeight = Math.max(28, el.clientHeight * ratio)
  const maxTop = el.clientHeight - thumbHeight
  const scrollRatio = el.scrollTop / (el.scrollHeight - el.clientHeight)
  settingsScrollThumbHeight.value = thumbHeight
  settingsScrollThumbTop.value = maxTop * scrollRatio
}

// Appearance
async function onThemeChange(v: string) {
  settings.value.theme = v
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

async function onPinShortcutsToTopChange(v: boolean) {
  settings.value.pin_shortcuts_to_top = v
  await saveSettings()
}

async function onFollowCursorChange(v: boolean) {
  settings.value.follow_cursor = v
  await saveSettings()
  await emitTo('launcher', 'settings-changed', { follow_cursor: v }).catch(console.error)
}

async function onPlaySoundChange(v: boolean) {
  settings.value.play_sound = v
  await saveSettings()
  await emitTo('launcher', 'settings-changed', { play_sound: v }).catch(console.error)
}

async function closeWindow() {
  let shouldRestoreLauncher = true
  if (isTauriContext.value) {
    shouldRestoreLauncher = await invoke<boolean>('consume_restore_launcher_on_settings_close')
      .catch(() => true)
  }

  if (shouldRestoreLauncher) {
    await emitTo('launcher', 'launcher-show', { source: 'settings' }).catch(console.error)
  }
  await getCurrentWindow().hide()
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') closeWindow()
}

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
  window.clearTimeout(settingsScrollTimer)
  void setHotkeyCaptureActive(false)
  unlistenConflictRef?.()
})
</script>

<template>
  <div class="settings-app" :data-theme="settings.theme === 'system' ? undefined : settings.theme">
    <div class="settings-header" data-tauri-drag-region>
      <span class="settings-title" data-tauri-drag-region>Riftle Settings</span>
      <button class="settings-close" type="button" @mousedown.stop @click="closeWindow">&times;</button>
    </div>
    <div
      ref="settingsContentRef"
      :class="['settings-content', { 'settings-content--scrolling': isSettingsScrolling }]"
      @scroll="onSettingsScroll"
    >

      <Section title="General">
        <Row
          label="Launch at startup"
          :hint="autostartHint"
        >
          <Toggle
            v-model="settings.autostart"
            :disabled="!canAutostart"
            @update:modelValue="onAutostartChange"
          />
        </Row>
        <Row label="Play sound on open">
          <Toggle v-model="settings.play_sound" @update:modelValue="onPlaySoundChange" />
        </Row>
      </Section>

      <Section title="Hotkey">
        <Row label="Global shortcut" ref="hotkeyRowRef">
          <div class="hotkey-row">
            <button
              v-if="settings.hotkey !== defaultHotkey"
              type="button"
              class="reset-link"
              @click="onHotkeyChange(defaultHotkey)"
            >Reset</button>
            <KeyCapture
              v-model="settings.hotkey"
              @change="onHotkeyChange"
              @capture-start="setHotkeyCaptureActive(true)"
              @capture-end="setHotkeyCaptureActive(false)"
            />
          </div>
        </Row>
        <p v-if="hotkeyError" class="hotkey-error">{{ hotkeyError }}</p>
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

        <Row label="Pin shortcuts to top">
          <Toggle
            v-model="settings.pin_shortcuts_to_top"
            @update:modelValue="onPinShortcutsToTopChange"
          />
        </Row>

        <Row label="Show where cursor is">
          <Toggle
            v-model="settings.follow_cursor"
            @update:modelValue="onFollowCursorChange"
          />
        </Row>
      </Section>

      <Section
        title="Search"
        description="Riftle already scans the standard Windows app locations by default. Add extra folders here when you want the searchable index to include apps or shortcuts stored somewhere else, and exclude paths that should never be scanned."
      >
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
          <Button variant="accent" @click="onReindexNow">{{ reindexButtonText }}</Button>
        </Row>
      </Section>

      <Section
        title="Shortcuts"
        description="Create named shortcuts that appear directly in search results. Use these for folders, files, or file launches that need a specific app or extra parameters."
      >
        <div class="shortcut-tabs">
          <button
            type="button"
            :class="['shortcut-tab', { 'shortcut-tab--active': activeShortcutTab === 'directory' }]"
            @click="activeShortcutTab = 'directory'"
          >
            <span>Directories</span>
            <span class="shortcut-tab-add" @click.stop="addShortcutFromTab('directory')">+ Add folder</span>
          </button>
          <button
            type="button"
            :class="['shortcut-tab', { 'shortcut-tab--active': activeShortcutTab === 'file' }]"
            @click="activeShortcutTab = 'file'"
          >
            <span>Files</span>
            <span class="shortcut-tab-add" @click.stop="addShortcutFromTab('file')">+ Add file</span>
          </button>
        </div>

        <div class="shortcut-tab-panel">
          <ShortcutList
            v-show="activeShortcutTab === 'directory'"
            ref="directoryShortcutList"
            label="Directories"
            mode="directory"
            v-model="settings.directory_shortcuts"
            :sibling-shortcuts="settings.file_shortcuts"
            :show-header="false"
            @change="onShortcutsChange('directory_shortcuts', $event as DirectoryShortcut[])"
          />
          <ShortcutList
            v-show="activeShortcutTab === 'file'"
            ref="fileShortcutList"
            label="Files"
            mode="file"
            v-model="settings.file_shortcuts"
            :sibling-shortcuts="settings.directory_shortcuts"
            :show-header="false"
            @change="onShortcutsChange('file_shortcuts', $event as FileShortcut[])"
          />
        </div>
        <p v-if="shortcutsError" class="shortcuts-error">{{ shortcutsError }}</p>
      </Section>

    </div>
    <div
      v-if="isSettingsScrolling && settingsScrollThumbHeight > 0"
      class="settings-scroll-thumb"
      :style="settingsScrollThumbStyle"
    />
  </div>
</template>

<style scoped>
.settings-app {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--surface-settings-bg);
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
  scrollbar-width: none;
}

input[type='range'] {
  accent-color: var(--color-accent);
}

/* Custom scrollbar for settings content */
.settings-content::-webkit-scrollbar {
  width: 0;
  height: 0;
}

.settings-content::-webkit-scrollbar-track {
  background: transparent;
}

.settings-content::-webkit-scrollbar-thumb {
  background: transparent;
  border-radius: 3px;
}

.settings-content--scrolling::-webkit-scrollbar-thumb {
  background: transparent;
}

.settings-content--scrolling::-webkit-scrollbar-thumb:hover {
  background: transparent;
}

.settings-content::-webkit-scrollbar-button {
  display: none;
  width: 0;
  height: 0;
  -webkit-appearance: none;
  background: transparent;
}

.settings-scroll-thumb {
  position: absolute;
  top: 44px;
  right: 2px;
  width: 6px;
  border-radius: 3px;
  background: var(--color-border);
  pointer-events: none;
}

.settings-content::-webkit-scrollbar-button:single-button,
.settings-content::-webkit-scrollbar-button:vertical:start:decrement,
.settings-content::-webkit-scrollbar-button:vertical:start:increment,
.settings-content::-webkit-scrollbar-button:vertical:end:decrement,
.settings-content::-webkit-scrollbar-button:vertical:end:increment,
.settings-content::-webkit-scrollbar-button:horizontal:start:decrement,
.settings-content::-webkit-scrollbar-button:horizontal:start:increment,
.settings-content::-webkit-scrollbar-button:horizontal:end:decrement,
.settings-content::-webkit-scrollbar-button:horizontal:end:increment {
  display: none;
  width: 0;
  height: 0;
  -webkit-appearance: none;
  background: transparent;
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

.hotkey-error {
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  color: var(--color-danger);
  padding: 0 var(--spacing-lg);
  margin-top: calc(-1 * var(--spacing-sm));
  margin-bottom: var(--spacing-sm);
}

.shortcuts-error {
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  color: var(--color-danger);
  padding: 0;
  margin: var(--spacing-xs) 0 var(--spacing-sm);
}

.shortcut-tabs {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--spacing-sm);
  margin: var(--spacing-sm) 0 0;
}

.shortcut-tab {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-sm);
  min-width: 0;
  min-height: 38px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background: var(--color-bg-darker);
  color: var(--color-text-muted);
  cursor: pointer;
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  padding: 0 var(--spacing-sm);
}

.shortcut-tab--active {
  border-color: var(--color-accent);
  color: var(--color-text);
}

.shortcut-tab-add {
  color: var(--color-accent);
  font-size: var(--font-size-xs);
  white-space: nowrap;
}

.shortcut-tab-add:hover {
  opacity: 0.75;
}

.shortcut-tab-panel {
  height: 340px;
  overflow: hidden;
}

.shortcut-tab-panel :deep(.shortcut-list) {
  height: 340px;
  max-height: 340px;
  padding-bottom: 0;
  box-sizing: border-box;
}

.shortcut-tab-panel :deep(.shortcut-list-shell) {
  height: 340px;
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
