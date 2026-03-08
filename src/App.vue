<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
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

// ---- State ----
const query         = ref('')
const results       = ref<SearchResult[]>([])
const selectedIndex = ref(0)
const adminMode     = ref(false)
const showPath      = ref(false)
const animMode      = ref<'instant' | 'fade' | 'slide'>('slide')
const dataDir       = ref('')
const isVisible     = ref(false)
const inputRef      = ref<HTMLInputElement | null>(null)
const scrollerRef   = ref<any>(null) // RecycleScroller reference
const isTauriContext = ref(false) // true when running inside Tauri app, false in browser dev

// ---- Context menu state ----
const menuVisible  = ref(false)
const menuX        = ref(0)
const menuY        = ref(0)
const MENU_HEIGHT  = 80 // px — approximate height of 2-item context menu

let unlistenFocus: (() => void) | null = null
let unlistenShow: (() => void) | null = null
let launchInProgress = false

// ---- Computed ----
const listHeight = computed(() =>
  Math.min(results.value.length, 5) * 48
)

// ---- Watchers ----
watch(query, async (q) => {
  results.value = q.trim()
    ? await invoke<SearchResult[]>('search', { query: q }).catch(() => [])
    : []
  console.log('[App] search results:', results.value.length, 'items')
  selectedIndex.value = 0
  await updateWindowHeight()
  console.log('[App] window height updated')
})

watch(results, () => {
  selectedIndex.value = 0
})

watch(menuVisible, async (visible) => {
  if (!visible && isTauriContext.value) {
    const h = Math.max(56 + listHeight.value, 56)
    await getCurrentWindow().setSize(new LogicalSize(500, h)).catch(console.error)
  }
})

watch(selectedIndex, () => {
  // Scroll only when selection is at edge of visible items
  if (scrollerRef.value && results.value.length > 5) {
    const visibleRows = 5
    const firstVisible = Math.floor((scrollerRef.value.$el?.scrollTop || 0) / 48)
    const lastVisible = firstVisible + visibleRows - 1

    // Scroll if selection is above first visible or below last visible
    if (selectedIndex.value < firstVisible || selectedIndex.value > lastVisible) {
      scrollerRef.value.scrollToItem(selectedIndex.value)
    }
  }
})

// ---- Window sizing ----
async function updateWindowHeight() {
  if (!isTauriContext.value) {
    console.log('[App] updateWindowHeight skipped: not in Tauri context')
    return
  }
  const h = Math.max(56 + listHeight.value, 56)
  console.log('[App] updateWindowHeight:', { listHeight: listHeight.value, totalHeight: h })
  // Delay OS window resize until after the CSS height transition completes
  const delay = animMode.value === 'slide' ? 180 : animMode.value === 'fade' ? 120 : 0
  if (delay > 0) {
    await new Promise(resolve => setTimeout(resolve, delay))
  }
  await getCurrentWindow().setSize(new LogicalSize(500, h)).catch(console.error)
}

// ---- Icon URL ----
function getIconUrl(iconPath: string): string {
  if (!iconPath) return ''
  // icon_path is a filename (e.g. "notepad.png"); construct absolute path
  const sep = dataDir.value.includes('\\') ? '\\' : '/'
  const fullPath = dataDir.value + sep + 'icons' + sep + iconPath
  const url = convertFileSrc(fullPath)
  console.log('[App] icon URL:', { iconPath, dataDir: dataDir.value, fullPath, url })
  return url
}

// ---- Launch stubs (Phase 6 implements commands) ----
async function launchItem(item: SearchResult) {
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
  // Do NOT call hideWindow() here — the Rust command owns the hide decision.
  // On success: Rust hides the window and the process launches elevated.
  // On UAC cancel: Rust returns Ok(()) without hiding, so the launcher stays open.
  setTimeout(() => { launchInProgress = false }, 500)
}

// ---- Keyboard ----
function onKeyDown(e: KeyboardEvent) {
  adminMode.value = e.ctrlKey && e.shiftKey

  if (e.key === 'Escape') {
    e.preventDefault()
    if (menuVisible.value) {
      closeMenu()
      inputRef.value?.focus()
      return
    }
    hideWindow()
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
      e.preventDefault()
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
  // Right-click on result rows is reserved for future per-result menu — ignore
  if ((e.target as HTMLElement).closest('.result-row')) return
  // Clamp X so menu (min-width 160px) does not overflow the 500px window
  menuX.value = Math.min(e.clientX, 500 - 170)
  menuY.value = e.clientY
  menuVisible.value = true
  // Resize Tauri window if the menu would extend beyond the current window height
  if (isTauriContext.value) {
    const contentH = Math.max(56 + listHeight.value, 56)
    const neededH = menuY.value + MENU_HEIGHT + 8
    if (neededH > contentH) {
      await getCurrentWindow().setSize(new LogicalSize(500, neededH)).catch(console.error)
    }
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

// ---- Window show/hide ----
// showWindow is kept for Phase 8 Settings window — not called by launcher path (Phase 9 owns show via hotkey)
// @ts-ignore: reserved for Phase 8 Settings window show logic
async function showWindow() {
  if (!isTauriContext.value) return // Skip in browser dev mode
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
  isVisible.value = false
  const delay = animMode.value === 'slide' ? 180 : animMode.value === 'fade' ? 120 : 0
  await new Promise(resolve => setTimeout(resolve, delay))
  if (isTauriContext.value) {
    console.log('[App] calling getCurrentWindow().hide()')
    await getCurrentWindow().hide().catch(e => {
      console.error('[App] hideWindow failed:', e)
    })
    console.log('[App] window hidden')
  } else {
    console.log('[App] hideWindow skipped: not in Tauri context')
  }
}

// ---- Lifecycle ----
onMounted(async () => {
  console.log('[App] onMounted called')

  // Detect if we're in Tauri context (not in browser dev mode)
  // Tauri v2 exposes __TAURI_INTERNALS__ on the window object
  isTauriContext.value = '__TAURI_INTERNALS__' in window
  console.log('[App] Tauri context available:', isTauriContext.value)

  // Load settings from Rust (only works in Tauri context)
  if (isTauriContext.value) {
    try {
      const settings = await invoke<{
        show_path: boolean
        animation: string
        data_dir: string
      }>('get_settings_cmd')
      showPath.value = settings.show_path
      animMode.value = (settings.animation ?? 'slide') as typeof animMode.value
      dataDir.value  = settings.data_dir
      console.log('[App] settings loaded:', { dataDir: dataDir.value, showPath: showPath.value, animMode: animMode.value })
    } catch (e) {
      // Use defaults — dataDir stays empty, icons will not load but app still functions
      console.warn('[launcher] get_settings_cmd failed, using defaults:', e)
    }
  }

  // Set initial window height (no results yet)
  await updateWindowHeight()

  // In Tauri context: launcher stays hidden until first hotkey press (Phase 9 handles show).
  // In browser dev mode: show immediately so the dev workflow continues to work.
  if (isTauriContext.value) {
    const win = getCurrentWindow()
    if (win.label === 'launcher') {
      // Hotkey (Alt+Space) owns show/hide — do NOT call showWindow() here.
      // isVisible stays false until the 'launcher-show' event fires.
    }
  } else {
    // Browser dev mode: show immediately and focus the input
    isVisible.value = true
    await nextTick()
    inputRef.value?.focus()
  }

  // Auto-hide on focus loss (only in Tauri context)
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

  // Listen for 'launcher-show' event from hotkey.rs — replay animation, clear query, focus
  if (isTauriContext.value) {
    unlistenShow = await listen('launcher-show', async () => {
      menuVisible.value = false
      // Reset animation to hidden state, clear results and query
      isVisible.value = false
      results.value = []
      query.value = ''
      // Resize OS window to empty height immediately (window is hidden — no animation delay needed)
      await getCurrentWindow().setSize(new LogicalSize(500, 56)).catch(console.error)
      // Center after resize so the position is based on the correct (empty) height
      await getCurrentWindow().center().catch(console.error)
      // Wait for CSS to apply the hidden state
      await nextTick()
      // Trigger the appear animation
      isVisible.value = true
      await nextTick()
      inputRef.value?.focus()
    })
  }
})

onUnmounted(() => {
  unlistenFocus?.()
  unlistenShow?.()
})
</script>

<template>
  <div class="launcher-app" :class="[`anim-${animMode}`, { visible: isVisible }]" @contextmenu.prevent="onContextMenu">

    <!-- Search input area -->
    <div class="search-area">
      <input
        ref="inputRef"
        v-model="query"
        class="search-input"
        type="text"
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
        placeholder="Search apps, or > for system commands…"
        @keydown="onKeyDown"
        @keyup="onKeyUp"
      />
      <img :src="magnifierIcon" class="magnifier-icon" alt="" aria-hidden="true" />
    </div>

    <!-- Divider (only when results exist) -->
    <div v-if="results.length > 0" class="divider"></div>

    <!-- Result list (virtualised) -->
    <RecycleScroller
      ref="scrollerRef"
      v-if="results.length > 0"
      class="result-list"
      :items="results"
      :item-size="48"
      key-field="id"
      :style="{ height: listHeight + 'px' }"
      v-slot="{ item, index, active }"
    >
      <div
        class="result-row"
        :class="{ selected: active && index === selectedIndex }"
        @mousedown.left.prevent="launchItem(item)"
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
}

/* ---- Launcher container ---- */
.launcher-app {
  position: relative;
  width: 100%;
  height: auto;
  background: linear-gradient(180deg, var(--color-bg-lighter) 0%, var(--color-bg) 40%, var(--color-bg-darker) 100%);
  /* overflow: hidden; */

  border-radius: var(--radius);
  border: 1px solid var(--color-border);
  /* Josh Comeau;s beatiful shadows */
  /* box-shadow:
      1px 2px 2px hsl(220deg 60% 50% / 0.2),
      2px 4px 4px hsl(220deg 60% 50% / 0.2),
      4px 8px 8px hsl(220deg 60% 50% / 0.2),
      8px 16px 16px hsl(220deg 60% 50% / 0.2),
      16px 32px 32px hsl(220deg 60% 50% / 0.2); */

  /* Animation: hidden state */
  opacity: 0;
  transform: translateY(-6px);
}

/* Animation modes */
.anim-fade   { transition: opacity var(--duration-fast) ease; }
.anim-fade.visible { opacity: 1; }

.anim-slide  { transition: opacity var(--duration-normal) ease, transform var(--duration-normal) ease; }
.anim-slide.visible { opacity: 1; transform: translateY(0); }

.anim-instant { transition: none; }
.anim-instant.visible { opacity: 1; transform: translateY(0); }

/* ---- Search area ---- */
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
  padding-right: 28px; /* room for magnifier icon */
}

.search-input::placeholder {
  color: var(--color-text-dim);
  font-weight: 400;
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
.result-list {
  overflow-y: auto;
  overflow-x: hidden;
  scrollbar-width: none; /* Firefox */
  transition: height var(--duration-normal) ease;
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
  background: var(--color-selection-bg);
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
  min-width: 0; /* enables text-overflow: ellipsis in children */
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
  min-width: 160px;
  padding: 4px 0;
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
}

.menu-item:hover {
  background: var(--color-selection-bg);
  color: #ffffff;
}
</style>
