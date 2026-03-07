<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
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

let unlistenFocus: (() => void) | null = null
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
  // 56px input + rows
  const h = Math.max(56 + listHeight.value, 56)
  console.log('[App] updateWindowHeight:', { listHeight: listHeight.value, totalHeight: h })
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
  setTimeout(() => { launchInProgress = false }, 500)
  await hideWindow()
}

// ---- Keyboard ----
function onKeyDown(e: KeyboardEvent) {
  adminMode.value = e.ctrlKey && e.shiftKey

  if (e.key === 'Escape') {
    e.preventDefault()
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

// ---- Window show/hide ----
async function showWindow() {
  if (!isTauriContext.value) return // Skip in browser dev mode
  try {
    console.log('[App] showWindow called')
    const win = getCurrentWindow()
    console.log('[App] got window reference:', win)
    await win.show()
    console.log('[App] window shown successfully')
  } catch (e) {
    console.error('[App] showWindow failed:', e)
  }
}

async function hideWindow() {
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

  // Focus the input
  await nextTick()
  inputRef.value?.focus()

  // Delay before making CSS visible (allows window to settle)
  await nextTick()
  isVisible.value = true

  // Show the Tauri window only for the launcher (Phase 9 will wire hotkey to toggle this)
  // Settings window has its own show logic; we must not show it here
  if (isTauriContext.value) {
    const win = getCurrentWindow()
    if (win.label === 'launcher') {
      await showWindow()
    }
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
})

onUnmounted(() => {
  unlistenFocus?.()
})
</script>

<template>
  <div class="launcher-app" :class="[`anim-${animMode}`, { visible: isVisible }]">

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
      v-slot="{ item, index }"
    >
      <div
        class="result-row"
        :class="{ selected: index === selectedIndex }"
        @mousedown.prevent="launchItem(item)"
        @mousemove="selectedIndex = index"
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
  width: 100%;
  height: 100%;
  background: linear-gradient(180deg, #242427 0%, #1c1c1e 40%, #181818 100%);
  /* overflow: hidden; */

  border-radius: 9px;
  border: 1px solid rgba(255,255,255,0.15);
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
.anim-fade   { transition: opacity 120ms ease; }
.anim-fade.visible { opacity: 1; }

.anim-slide  { transition: opacity 180ms ease, transform 180ms ease; }
.anim-slide.visible { opacity: 1; transform: translateY(0); }

.anim-instant { transition: none; }
.anim-instant.visible { opacity: 1; transform: translateY(0); }

/* ---- Search area ---- */
.search-area {
  display: flex;
  align-items: center;
  height: 56px;
  padding: 0 16px;
  position: relative;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: #f0f0f0;
  font-family: 'Inter', sans-serif;
  font-size: 18px;
  font-weight: 400;
  caret-color: #0A84FF;
  padding: 0;
  padding-right: 28px; /* room for magnifier icon */
}

.search-input::placeholder {
  color: #555558;
  font-weight: 400;
}

.magnifier-icon {
  position: absolute;
  right: 16px;
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
  background: #ffffff18;
  margin: 0;
}

/* ---- Result list ---- */
.result-list {
  overflow-y: auto;
  overflow-x: hidden;
  scrollbar-width: none; /* Firefox */
}
.result-list::-webkit-scrollbar { display: none; }

/* ---- Result row ---- */
.result-row {
  display: flex;
  align-items: center;
  height: 48px;
  padding: 0 12px;
  cursor: pointer;
  position: relative;
  gap: 10px;
}

.result-row.selected {
  background: rgba(10, 132, 255, 0.18);
}

/* ---- App icon ---- */
.app-icon {
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  object-fit: contain;
  border-radius: 4px;
}

/* ---- Text block ---- */
.result-text {
  display: flex;
  flex-direction: column;
  min-width: 0; /* enables text-overflow: ellipsis in children */
  flex: 1;
}

.app-name {
  font-family: 'Inter', sans-serif;
  font-size: 14px;
  font-weight: 500;
  color: #f0f0f0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-row.selected .app-name {
  color: #ffffff;
}

.path-line {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 400;
  color: #888;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 1px;
}

/* ---- Admin badge ---- */
.admin-badge {
  font-family: 'Inter', sans-serif;
  font-size: 11px;
  font-weight: 500;
  color: #0A84FF;
  flex-shrink: 0;
  margin-left: auto;
  padding-left: 8px;
}
</style>
