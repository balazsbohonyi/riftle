<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { LogicalSize } from '@tauri-apps/api/dpi'
import { RecycleScroller } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'
import magnifierIcon from './assets/magnifier.svg'

interface SearchResult {
  id: string
  name: string
  icon_path: string
  path: string
  kind: string
  requires_elevation: boolean
}

interface SettingsPayload {
  theme?: string
  show_path?: boolean
  reindex_interval?: number
}

interface BackendWarning {
  kind: string
  title: string
  message: string
  backup_path?: string | null
}

const GENERIC_ICON_FILENAME = 'generic.png'
const CONFIRM_REQUIRED = new Set(['system:shutdown', 'system:restart'])
const SHADOW_PAD = 32
const WINDOW_WIDTH = 564
const SEARCH_AREA_HEIGHT = 56
const RESULT_ROW_HEIGHT = 48
const RESULT_PANEL_VERTICAL_PAD = 16
const MAX_VISIBLE_RESULTS = 5

// ---- State ----
const query         = ref('')
const results       = ref<SearchResult[]>([])
const selectedIndex = ref(0)
const adminMode     = ref(false)
const showPath      = ref(false)
const inputRef      = ref<HTMLInputElement | null>(null)
const scrollerRef   = ref<any>(null)
const confirmBtnRef = ref<HTMLButtonElement | null>(null)
const cancelBtnRef  = ref<HTMLButtonElement | null>(null)
const warningListRef = ref<HTMLElement | null>(null)
const isTauriContext = ref(false)
const iconUrls      = ref<Record<string, string>>({})
const backendWarnings = ref<BackendWarning[]>([])
const resultPanelOpen = ref(false)
const resultPanelRendered = ref(false)
const renderedResults = ref<SearchResult[]>([])
const suppressResultPanelTransition = ref(false)

const iconRequests = new Map<string, Promise<string>>()
let resultPanelFrame: number | null = null

// ---- Context menu state ----
const menuVisible  = ref(false)
const menuX        = ref(0)
const menuY        = ref(0)
const MENU_HEIGHT  = 80

// ---- Confirmation overlay state ----
const confirmPending  = ref(false)
const pendingCommand  = ref<SearchResult | null>(null)

let unlistenFocus: (() => void) | null = null
let unlistenShow: (() => void) | null = null
let unlistenSettings: (() => void) | null = null
let unlistenBackendWarnings: (() => void) | null = null
let launchInProgress = false
let searchRequestId = 0
let restoredQuerySelected = false
let clearResultsForNextQueryChange = false

// ---- Computed ----
const listHeight = computed(() =>
  Math.min(renderedResults.value.length, MAX_VISIBLE_RESULTS) * RESULT_ROW_HEIGHT + RESULT_PANEL_VERTICAL_PAD
)

const shouldShowResults = computed(() => results.value.length > 0 && !confirmPending.value)
const targetListHeight = computed(() =>
  shouldShowResults.value ? Math.min(results.value.length, MAX_VISIBLE_RESULTS) * RESULT_ROW_HEIGHT + RESULT_PANEL_VERTICAL_PAD : 0
)
const resultPanelHeight = computed(() =>
  resultPanelOpen.value && resultPanelRendered.value ? 1 + listHeight.value : 0
)

function cancelResultPanelFrame() {
  if (resultPanelFrame === null) return
  cancelAnimationFrame(resultPanelFrame)
  resultPanelFrame = null
}

function openResultPanelOnNextFrame() {
  cancelResultPanelFrame()
  resultPanelFrame = requestAnimationFrame(() => {
    resultPanelFrame = requestAnimationFrame(() => {
      resultPanelOpen.value = true
      resultPanelFrame = null
    })
  })
}

function onResultPanelTransitionEnd(event: TransitionEvent) {
  if (event.propertyName !== 'height' || resultPanelOpen.value) return
  resultPanelRendered.value = false
  renderedResults.value = []
}

function resetResultPanel() {
  cancelResultPanelFrame()
  resultPanelOpen.value = false
  resultPanelRendered.value = false
  renderedResults.value = []
}

function restoreResultPanelInstantly() {
  cancelResultPanelFrame()
  if (!results.value.length || confirmPending.value) {
    resetResultPanel()
    return
  }
  suppressResultPanelTransition.value = true
  renderedResults.value = results.value
  resultPanelRendered.value = true
  resultPanelOpen.value = true
}

function inputHasFullQuerySelection(): boolean {
  const input = inputRef.value
  return !!input && query.value.length > 0 && input.selectionStart === 0 && input.selectionEnd === query.value.length
}

function warningKey(warning: BackendWarning): string {
  return [
    warning.kind,
    warning.title,
    warning.message,
    warning.backup_path ?? '',
  ].join('::')
}

function appendBackendWarnings(warnings: BackendWarning[]) {
  if (!warnings.length) return

  const seen = new Set(backendWarnings.value.map(warningKey))
  const next = [...backendWarnings.value]

  for (const warning of warnings) {
    const key = warningKey(warning)
    if (seen.has(key)) continue
    seen.add(key)
    next.push(warning)
  }

  backendWarnings.value = next
}

function dismissBackendWarning(key: string) {
  backendWarnings.value = backendWarnings.value.filter((warning) => warningKey(warning) !== key)
}

function setIconUrl(iconPath: string, url: string) {
  if (!url || iconUrls.value[iconPath] === url) return
  iconUrls.value = {
    ...iconUrls.value,
    [iconPath]: url,
  }
}

function createIconUrl(bytes: number[]): string {
  const payload = Uint8Array.from(bytes)
  return URL.createObjectURL(new Blob([payload], { type: 'image/png' }))
}

async function loadIconUrl(iconPath: string): Promise<string> {
  if (!iconPath) return ''

  const cached = iconUrls.value[iconPath]
  if (cached) return cached

  const pending = iconRequests.get(iconPath)
  if (pending) return pending

  const request = (async () => {
    try {
      const bytes = await invoke<number[]>('get_icon_bytes', { iconPath })
      const url = createIconUrl(bytes)
      setIconUrl(iconPath, url)
      return url
    } catch (error) {
      if (iconPath !== GENERIC_ICON_FILENAME) {
        return loadIconUrl(GENERIC_ICON_FILENAME)
      }
      console.warn('[launcher] get_icon_bytes failed for generic icon:', error)
      return ''
    }
  })()

  iconRequests.set(iconPath, request)
  try {
    return await request
  } finally {
    iconRequests.delete(iconPath)
  }
}

function primeIconUrl(iconPath: string) {
  if (!isTauriContext.value || !iconPath) return
  if (iconUrls.value[iconPath] || iconRequests.has(iconPath)) return

  void loadIconUrl(iconPath)
}

// ---- Watchers ----
watch(query, async (q) => {
  const requestId = ++searchRequestId
  const trimmed = q.trim()
  const shouldClearRestoredResults = clearResultsForNextQueryChange
  clearResultsForNextQueryChange = false
  restoredQuerySelected = false
  selectedIndex.value = 0

  if (!trimmed) {
    results.value = []
    resetResultPanel()
    await updateWindowHeight()
    return
  }

  if (shouldClearRestoredResults) {
    results.value = []
    resetResultPanel()
    await updateWindowHeight()
  }

  const nextResults = await invoke<SearchResult[]>('search', { query: q }).catch(() => [])
  if (requestId !== searchRequestId) return
  results.value = nextResults
  console.log('[App] search results:', results.value.length, 'items')
  await updateWindowHeight()
  console.log('[App] window height updated')
})

watch(results, (items) => {
  selectedIndex.value = 0
  if (shouldShowResults.value) {
    renderedResults.value = items
  }
  if (isTauriContext.value) {
    for (const item of items) {
      primeIconUrl(item.icon_path)
    }
  }
})

watch(shouldShowResults, async (show) => {
  cancelResultPanelFrame()
  if (show) {
    renderedResults.value = results.value
    resultPanelRendered.value = true
    if (suppressResultPanelTransition.value) {
      resultPanelOpen.value = true
      await updateWindowHeight()
      return
    }
    resultPanelOpen.value = false
    await nextTick()
    openResultPanelOnNextFrame()
    await updateWindowHeight()
    return
  }

  resultPanelOpen.value = false
  if (!resultPanelRendered.value) {
    renderedResults.value = []
  }
  await updateWindowHeight()
})

watch(backendWarnings, async () => {
  await nextTick()
  await updateWindowHeight()
}, { deep: true })

watch(menuVisible, async (visible) => {
  if (!visible && isTauriContext.value) {
    await nextTick()
    const warningHeight = warningListRef.value?.offsetHeight ?? 0
    await setLauncherWindowSize(warningHeight)
  }
})

watch(selectedIndex, () => {
  if (scrollerRef.value && results.value.length > MAX_VISIBLE_RESULTS) {
    const visibleRows = MAX_VISIBLE_RESULTS
    const firstVisible = Math.floor((scrollerRef.value.$el?.scrollTop || 0) / RESULT_ROW_HEIGHT)
    const lastVisible = firstVisible + visibleRows - 1

    if (selectedIndex.value < firstVisible || selectedIndex.value > lastVisible) {
      scrollerRef.value.scrollToItem(selectedIndex.value)
    }
  }
})

// ---- Window sizing ----
function launcherWindowHeight(warningHeight: number, listHeight: number) {
  return SHADOW_PAD + Math.max(SEARCH_AREA_HEIGHT + listHeight + warningHeight, SEARCH_AREA_HEIGHT) + SHADOW_PAD
}

function launcherCenterAnchorHeight(warningHeight: number) {
  const fiveResultListHeight = MAX_VISIBLE_RESULTS * RESULT_ROW_HEIGHT + RESULT_PANEL_VERTICAL_PAD
  return launcherWindowHeight(warningHeight, fiveResultListHeight)
}

async function setLauncherWindowSize(warningHeight: number) {
  const h = launcherWindowHeight(warningHeight, targetListHeight.value)
  console.log('[App] updateWindowHeight:', {
    listHeight: targetListHeight.value,
    totalHeight: h,
  })
  await getCurrentWindow().setSize(new LogicalSize(WINDOW_WIDTH, h)).catch(console.error)
}

async function updateWindowHeight() {
  if (!isTauriContext.value) {
    console.log('[App] updateWindowHeight skipped: not in Tauri context')
    return
  }
  await nextTick()
  const warningHeight = warningListRef.value?.offsetHeight ?? 0
  await setLauncherWindowSize(warningHeight)
}

async function showPositionedLauncher() {
  if (!isTauriContext.value) return
  const warningHeight = warningListRef.value?.offsetHeight ?? 0
  await invoke('show_positioned_launcher', {
    windowWidth: WINDOW_WIDTH,
    windowHeight: launcherWindowHeight(warningHeight, targetListHeight.value),
    anchorHeight: launcherCenterAnchorHeight(warningHeight),
  }).catch(console.error)
}

// ---- Icon URL ----
function getIconUrl(iconPath: string): string {
  if (!iconPath) return ''
  if (isTauriContext.value) {
    primeIconUrl(iconPath)
    return iconUrls.value[iconPath] ?? iconUrls.value[GENERIC_ICON_FILENAME] ?? ''
  }
  return ''
}

// ---- Launch stubs (Phase 6 implements commands) ----
async function launchItem(item: SearchResult) {
  if (item.kind === 'system' && CONFIRM_REQUIRED.has(item.id)) {
    showConfirm(item)
    return
  }
  launchInProgress = true
  if (item.kind === 'system') {
    await invoke('run_system_command', { cmd: item.id }).catch(console.error)
  } else {
    await invoke('launch', { id: item.id }).catch(console.error)
  }
  await hideWindow()
  launchInProgress = false
}

async function launchElevated(item: SearchResult) {
  launchInProgress = true
  await invoke('launch_elevated', { id: item.id }).catch(console.error)
  // Do NOT call hideWindow() here - the Rust command owns the hide decision.
  // On success: Rust hides the window and the process launches elevated.
  // On UAC cancel: Rust returns Ok(()) without hiding, so the launcher stays open.
  setTimeout(() => { launchInProgress = false }, 500)
}

// ---- Keyboard ----
function toggleConfirmFocus() {
  if (document.activeElement === cancelBtnRef.value) {
    confirmBtnRef.value?.focus()
    return
  }
  cancelBtnRef.value?.focus()
}

function onKeyDown(e: KeyboardEvent) {
  adminMode.value = e.ctrlKey && e.shiftKey
  const target = e.target as HTMLElement | null

  if (target === inputRef.value) {
    const willEditQuery =
      e.key.length === 1 ||
      e.key === 'Backspace' ||
      e.key === 'Delete' ||
      (e.key.toLowerCase() === 'v' && (e.ctrlKey || e.metaKey))
    clearResultsForNextQueryChange = restoredQuerySelected && inputHasFullQuerySelection() && willEditQuery
    if (!inputHasFullQuerySelection() && e.key !== 'Tab') {
      restoredQuerySelected = false
    }
  }

  if (e.key === ',' && e.ctrlKey) {
    e.preventDefault()
    openSettings()
    return
  }

  if (e.key === 'Escape') {
    e.preventDefault()
    if (menuVisible.value) {
      closeMenu()
      inputRef.value?.focus()
      return
    }
    if (confirmPending.value) {
      cancelConfirm()
      return
    }
    hideWindow()
    return
  }

  if (confirmPending.value && (e.key === 'ArrowLeft' || e.key === 'ArrowRight')) {
    e.preventDefault()
    toggleConfirmFocus()
    return
  }

  if (!results.value.length) return

  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value + 1) % results.value.length
      break
    case 'ArrowUp':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value - 1 + results.value.length) % results.value.length
      break
    case 'Enter': {
      if (confirmPending.value && target === cancelBtnRef.value) {
        return
      }
      e.preventDefault()
      if (confirmPending.value) {
        confirmAction()
        break
      }
      const item = results.value[selectedIndex.value]
      if (!item) break
      if (e.ctrlKey && e.shiftKey) {
        launchElevated(item)
      } else {
        launchItem(item)
      }
      break
    }
  }
}

function onKeyUp(e: KeyboardEvent) {
  adminMode.value = e.ctrlKey && e.shiftKey
}

// ---- Context menu ----
function closeMenu() {
  menuVisible.value = false
}

async function onContextMenu(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('.result-row')) return
  // Subtract SHADOW_PAD because transform on .launcher-app makes position:fixed
  // descendants use .launcher-app as their containing block, not the viewport.
  menuX.value = Math.min(e.clientX - SHADOW_PAD, 500 - 170)
  menuY.value = e.clientY - SHADOW_PAD
  menuVisible.value = true
  if (isTauriContext.value) {
    await nextTick()
    const warningHeight = warningListRef.value?.offsetHeight ?? 0
    const contentH = SHADOW_PAD + Math.max(56 + listHeight.value + warningHeight, 56) + SHADOW_PAD
    const neededH = menuY.value + MENU_HEIGHT + 2 * SHADOW_PAD
    if (neededH > contentH) {
      await getCurrentWindow().setSize(new LogicalSize(WINDOW_WIDTH, neededH)).catch(console.error)
    }
  }
}

function applyTheme(theme: string) {
  const root = document.documentElement
  if (theme === 'system') {
    root.removeAttribute('data-theme')
  } else {
    root.setAttribute('data-theme', theme)
  }
}

async function openSettings() {
  closeMenu()
  await invoke('open_settings_window').catch(console.error)
}

async function quitApp() {
  closeMenu()
  await invoke('quit_app').catch(console.error)
}

// ---- Confirmation overlay ----
function showConfirm(item: SearchResult) {
  pendingCommand.value = item
  confirmPending.value = true
  nextTick(() => { confirmBtnRef.value?.focus() })
}

function cancelConfirm() {
  confirmPending.value = false
  pendingCommand.value = null
  nextTick(() => {
    inputRef.value?.focus()
  })
}

async function confirmAction() {
  const item = pendingCommand.value
  if (!item) return
  confirmPending.value = false
  pendingCommand.value = null
  await hideWindow()
  await invoke('run_system_command', { cmd: item.id }).catch(console.error)
}

// ---- Window show/hide ----
// showWindow is kept for Phase 8 Settings window - not called by launcher path (Phase 9 owns show via hotkey)
// @ts-ignore: reserved for Phase 8 Settings window show logic
async function showWindow() {
  if (!isTauriContext.value) return
  try {
    console.log('[App] showWindow called')
    const win = getCurrentWindow()
    console.log('[App] got window reference:', win)
    await win.show()
    await win.setFocus()
    console.log('[App] window shown and focused successfully')
  } catch (e) {
    console.error('[App] showWindow failed:', e)
  }
}

async function hideWindow() {
  menuVisible.value = false
  console.log('[App] hideWindow called, isTauriContext:', isTauriContext.value)
  if (isTauriContext.value) {
    console.log('[App] calling getCurrentWindow().hide()')
    await getCurrentWindow().hide().catch(e => {
      console.error('[App] hideWindow failed:', e)
    })
    await updateWindowHeight()
    console.log('[App] window hidden')
  } else {
    console.log('[App] hideWindow skipped: not in Tauri context')
  }
}

// ---- Lifecycle ----
onMounted(async () => {
  console.log('[App] onMounted called')

  isTauriContext.value = '__TAURI_INTERNALS__' in window
  console.log('[App] Tauri context available:', isTauriContext.value)

  if (isTauriContext.value) {
    try {
      const settings = await invoke<{
        show_path: boolean
        theme: string
      }>('get_settings_cmd')
      showPath.value = settings.show_path
      if (settings.theme) applyTheme(settings.theme)
      await loadIconUrl(GENERIC_ICON_FILENAME)

      console.log('[App] settings loaded:', { showPath: showPath.value })
    } catch (e) {
      console.warn('[launcher] get_settings_cmd failed, using defaults:', e)
    }
  }

  if (isTauriContext.value) {
    unlistenBackendWarnings = await listen<BackendWarning>('backend-warning', ({ payload }) => {
      appendBackendWarnings([payload])
    })
    const pendingWarnings = await invoke<BackendWarning[]>('take_backend_warnings').catch(() => [])
    appendBackendWarnings(pendingWarnings)
  }

  await updateWindowHeight()

  if (isTauriContext.value) {
    const win = getCurrentWindow()
    if (win.label === 'launcher') {
      // Hotkey (Alt+Space) owns show/hide - do NOT call showWindow() here.
    }
  } else {
    await nextTick()
    inputRef.value?.focus()
  }

  if (isTauriContext.value) {
    console.log('[App] setting up focus listener')
    unlistenFocus = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      console.log('[App] focus changed:', { focused, launchInProgress })
      if (!focused && !launchInProgress) {
        console.log('[App] auto-hiding window')
        hideWindow()
      }
    })
    console.log('[App] focus listener registered')
  }

  if (isTauriContext.value) {
    unlistenShow = await listen('launcher-show', async () => {
      menuVisible.value = false
      confirmPending.value = false
      pendingCommand.value = null
      restoreResultPanelInstantly()
      await nextTick()
      await updateWindowHeight()
      await showPositionedLauncher()
      await nextTick()
      inputRef.value?.focus()
      inputRef.value?.select()
      restoredQuerySelected = query.value.length > 0
      requestAnimationFrame(() => {
        suppressResultPanelTransition.value = false
      })
    })
  }

  if (isTauriContext.value) {
    unlistenSettings = await listen<SettingsPayload>('settings-changed', ({ payload }) => {
      if (payload.theme !== undefined) applyTheme(payload.theme)
      if (payload.show_path !== undefined) showPath.value = payload.show_path
    })
  }
})

onUnmounted(() => {
  cancelResultPanelFrame()
  unlistenFocus?.()
  unlistenShow?.()
  unlistenSettings?.()
  unlistenBackendWarnings?.()
  for (const url of Object.values(iconUrls.value)) {
    URL.revokeObjectURL(url)
  }
  iconRequests.clear()
})
</script>

<template>
  <div class="launcher-app" @contextmenu.prevent="onContextMenu" @keydown="onKeyDown">
    <div v-if="backendWarnings.length > 0" ref="warningListRef" class="warning-stack">
      <div
        v-for="warning in backendWarnings"
        :key="warningKey(warning)"
        class="warning-banner"
      >
        <div class="warning-copy">
          <strong class="warning-title">{{ warning.title }}</strong>
          <span class="warning-message">{{ warning.message }}</span>
          <div v-if="warning.backup_path" class="warning-path-row">
            <span class="warning-path-label">Backup:</span>
            <span class="warning-path">{{ warning.backup_path }}</span>
          </div>
        </div>
        <button
          class="warning-dismiss"
          type="button"
          @mousedown.prevent="dismissBackendWarning(warningKey(warning))"
        >
          Dismiss
        </button>
      </div>
    </div>

    <!-- Search input area -->
    <div class="search-area">
      <!-- Normal search input — hidden while confirming -->
      <template v-if="!confirmPending">
        <input
          ref="inputRef"
          v-model="query"
          class="search-input"
          type="text"
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
          placeholder="Search apps, or > for system commands..."
          @keydown.stop="onKeyDown"
          @keyup="onKeyUp"
        />
        <img :src="magnifierIcon" class="magnifier-icon" alt="" aria-hidden="true" />
      </template>

      <!-- Inline confirmation row — shown while confirming -->
      <div v-if="confirmPending" class="confirm-row">
        <span class="confirm-question">
          {{ pendingCommand?.id === 'system:shutdown' ? 'Shut down Windows?' : 'Restart Windows?' }}
        </span>
        <div class="confirm-actions">
          <button
            ref="confirmBtnRef"
            class="confirm-btn confirm-btn--danger"
            @mousedown.prevent="confirmAction"
            @click="confirmAction"
          >
            {{ pendingCommand?.id === 'system:shutdown' ? 'Shut Down' : 'Restart' }}
          </button>
          <button
            ref="cancelBtnRef"
            class="confirm-btn confirm-btn--cancel"
            @mousedown.prevent="cancelConfirm"
            @click="cancelConfirm"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>

    <div
      v-if="resultPanelRendered"
      class="result-panel"
      :class="{ 'result-panel--no-transition': suppressResultPanelTransition }"
      :style="{ height: resultPanelHeight + 'px' }"
      @transitionend="onResultPanelTransitionEnd"
    >
      <!-- Divider (revealed with results) -->
      <div class="divider"></div>

      <!-- Result list (virtualised) -->
      <RecycleScroller
        ref="scrollerRef"
        class="result-list"
        :items="renderedResults"
        :item-size="48"
        key-field="id"
        :style="{ height: listHeight + 'px' }"
        v-slot="{ item, index, active }"
      >
        <div
          class="result-row"
          :class="{ selected: active && index === selectedIndex }"
          @mousedown.left.prevent="selectedIndex = index; launchItem(item)"
          @mousemove="active && (selectedIndex = index)"
          @contextmenu.prevent
        >
          <!-- Icon -->
          <img
            class="app-icon"
            :src="getIconUrl(item.icon_path)"
            :alt="item.name"
            width="32"
            height="32"
            loading="eager"
          />

          <!-- Text -->
          <div class="result-text">
            <span class="app-name">{{ item.name }}</span>
            <span
              v-if="index === selectedIndex && showPath && item.kind !== 'system'"
              class="path-line"
            >{{ item.path }}</span>
          </div>

          <!-- Admin badge (right margin, no layout shift) -->
          <span
            v-if="item.requires_elevation"
            class="admin-badge"
            aria-label="Elevate with admin rights"
          >[Admin]</span>
        </div>
      </RecycleScroller>
    </div>

    <!-- Context menu backdrop: click-outside closes menu -->
    <div
      v-if="menuVisible"
      class="menu-backdrop"
      @mousedown.prevent="closeMenu"
    ></div>

    <!-- Context menu overlay: absolutely positioned at right-click coordinates -->
    <div
      v-if="menuVisible"
      class="context-menu"
      :style="{ left: menuX + 'px', top: menuY + 'px' }"
    >
      <div class="menu-item" @mousedown.prevent="openSettings">Settings</div>
      <div class="menu-item" @mousedown.prevent="quitApp">Quit Launcher</div>
    </div>


  </div>
</template>

<style>
/* ---- Reset (inherits from body reset in Phase 1) ---- */
* { margin: 0; padding: 0; box-sizing: border-box; }

/* ---- Root window ---- */
html, body {
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: transparent;
}

/* Vue root mount point */
#app {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 32px;
}

/* ---- Launcher container ---- */
.launcher-app {
  position: relative;
  width: 100%;
  height: auto;

  background: linear-gradient(180deg, var(--color-bg-lighter) 0%, var(--color-bg) 40%, var(--color-bg-darker) 100%);

  border-radius: var(--radius);
  border: 1px solid var(--color-border);
  box-shadow:
    0 2px 4px  hsl(var(--shadow-color) / 0.32),
    0 4px 10px hsl(var(--shadow-color) / 0.26),
    0 8px 24px hsl(var(--shadow-color) / 0.18),
    0 16px 40px hsl(var(--shadow-color) / 0.12);
}

/* ---- Search area ---- */
.warning-stack {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  padding: var(--spacing-md) var(--spacing-md) 0;
}

.warning-banner {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--spacing-md);
  padding: var(--spacing-sm) var(--spacing-md);
  border: 1px solid color-mix(in srgb, var(--color-accent) 40%, var(--color-border));
  background: color-mix(in srgb, var(--color-accent) 14%, var(--color-bg-lighter));
  border-radius: var(--radius-sm);
}

.warning-copy {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.warning-title {
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-text);
}

.warning-message {
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
  line-height: 1.35;
}

.warning-path-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  min-width: 0;
  align-items: baseline;
}

.warning-path-label {
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.warning-path {
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  color: var(--color-text-soft);
  line-height: 1.35;
  word-break: break-word;
}

.warning-dismiss {
  border: none;
  background: transparent;
  color: var(--color-accent);
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  font-weight: 600;
  cursor: pointer;
  flex-shrink: 0;
  padding: 2px 0;
}

.warning-dismiss:hover {
  color: var(--color-text);
}

.search-area {
  display: flex;
  align-items: center;
  height: 56px;
  padding: 0 var(--spacing-lg);
  position: relative;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--color-text);
  font-family: var(--font-sans);
  font-size: var(--font-size-xl);
  font-weight: 400;
  caret-color: var(--color-accent);
  padding: 0;
  padding-right: 28px;
}

.search-input::placeholder {
  color: var(--color-text-dim);
  font-weight: 400;
}

.search-input::selection {
  background: var(--color-accent);
  color: #ffffff;
}

.magnifier-icon {
  position: absolute;
  right: var(--spacing-lg);
  top: 50%;
  transform: translateY(-50%);
  width: 18px;
  height: 18px;
  opacity: 0.5;
  pointer-events: none;
  user-select: none;
}

/* ---- Divider ---- */
.divider {
  height: 1px;
  background: var(--color-divider);
  margin: 0;
}

/* ---- Result list ---- */
.result-panel {
  overflow: hidden;
  transition: height var(--duration-normal) cubic-bezier(0.22, 1, 0.36, 1);
}

.result-panel--no-transition {
  transition: none;
}

.result-list {
  overflow-y: auto;
  overflow-x: hidden;
  scrollbar-width: none;
  padding: 8px;
}
.result-list::-webkit-scrollbar { display: none; }

/* ---- Result row ---- */
.result-row {
  display: flex;
  align-items: center;
  height: 48px;
  padding: 0 var(--spacing-md);
  cursor: pointer;
  position: relative;
  gap: 10px;
}

.result-row.selected {
  background: var(--color-accent);
  border-radius: var(--radius-sm);
}

/* ---- App icon ---- */
.app-icon {
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  object-fit: contain;
  border-radius: var(--radius-sm);
}

/* ---- Text block ---- */
.result-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}

.app-name {
  font-family: var(--font-sans);
  font-size: var(--font-size-base);
  font-weight: 500;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-row.selected .app-name {
  color: #ffffff;
}

.result-row.selected .path-line {
  color: rgba(255, 255, 255, 0.75);
}

.path-line {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  font-weight: 400;
  color: var(--color-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 1px;
}

/* ---- Admin badge ---- */
.admin-badge {
  font-family: var(--font-sans);
  font-size: var(--font-size-xs);
  font-weight: 500;
  color: var(--color-accent);
  flex-shrink: 0;
  margin-left: auto;
  padding-left: var(--spacing-sm);
}

/* ---- Context menu ---- */
.menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 99;
}

.context-menu {
  position: fixed;
  background: linear-gradient(180deg, var(--color-bg-lighter) 0%, var(--color-bg) 40%, var(--color-bg-darker) 100%);
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  width: max-content;
  z-index: 100;
  overflow: hidden;
}

.menu-item {
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  font-weight: 400;
  color: var(--color-text);
  padding: var(--spacing-sm) 14px;
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
}

.menu-item:hover {
  background: var(--color-accent);
  color: #ffffff;
}

/* ---- Inline confirmation row ---- */
.confirm-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  gap: var(--spacing-sm);
  padding: 0 var(--spacing-sm);
  height: 40px;
}

.confirm-question {
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.confirm-actions {
  display: flex;
  flex-direction: row;
  gap: var(--spacing-sm);
  justify-content: center;
}

.confirm-btn {
  font-family: var(--font-sans);
  font-size: var(--font-size-sm);
  font-weight: 500;
  padding: var(--spacing-sm) var(--spacing-lg);
  border-radius: var(--radius-sm);
  border: none;
  cursor: pointer;
  outline: none;
  transition: opacity var(--duration-fast) ease;
}

.confirm-btn:focus-visible {
  outline: 2px solid var(--color-accent);
  outline-offset: 2px;
}

.confirm-btn--danger {
  background: var(--color-accent);
  color: #ffffff;
}

.confirm-btn--danger:hover {
  opacity: 0.85;
}

.confirm-btn--cancel {
  background: transparent;
  color: var(--color-text-muted);
  border: 1px solid var(--color-border);
}

.confirm-btn--cancel:hover {
  color: var(--color-text);
  border-color: rgba(255, 255, 255, 0.3);
}
</style>
