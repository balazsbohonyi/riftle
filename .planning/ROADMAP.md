# Roadmap: Launcher — Minimal Windows App Launcher

**Milestone:** v1 — Full Working Launcher
**Granularity:** Standard (5-8 phases target → expanded to 10 given clear domain decomposition)
**Created:** 2026-03-05

---

## Phase 1: Project Scaffold & Configuration

**Goal:** Configure the Tauri v2 project skeleton so both windows (launcher and settings) are declared with correct flags, all required Rust crates are present, and `pnpm tauri dev` starts cleanly.

**Requirements**: SCAF-02, SCAF-03, SCAF-04

**Plans:** 2/2 plans complete

Plans:
- [ ] 01-01-PLAN.md — Expand Cargo.toml with all domain/plugin crates; replace lib.rs with plugin registration scaffold; create seven Rust module stubs
- [ ] 01-02-PLAN.md — Rewrite tauri.conf.json (two windows, correct flags); update capabilities; replace App.vue with transparent shell; install JS plugin packages; smoke-test `pnpm tauri dev`

**Success Criteria:**
1. `tauri.conf.json` declares two windows: launcher (frameless, skip_taskbar, always_on_top) and settings (normal, hidden by default)
2. `Cargo.toml` includes rusqlite (bundled), windows-sys with all required features, and all plugin crates
3. `pnpm tauri dev` compiles and opens the launcher window without errors
4. No build warnings from missing features or version conflicts

---

## Phase 2: Data Layer

**Goal:** Implement SQLite schema, settings persistence via tauri-plugin-store, and portable-mode path detection — the foundation every other module depends on.

**Requirements**: DATA-01, DATA-02, DATA-03, DATA-04, DATA-05, DATA-06, DATA-07

**Plans:** 3/3 plans complete

Plans:
- [ ] 02-01-PLAN.md — Create paths.rs (portable-aware data_dir); add mod paths to lib.rs; wire data_dir call in setup callback
- [ ] 02-02-PLAN.md — Implement db.rs (AppRecord, DbState, init_db, upsert_app, get_all_apps, increment_launch_count + unit tests); register DbState in lib.rs setup
- [ ] 02-03-PLAN.md — Implement store.rs (Settings struct, get_settings, set_settings + unit tests); wire get_settings in lib.rs setup for first-run defaults

**Success Criteria:**
1. On launch, SQLite database is created at the correct path (portable: ./data/, installed: %APPDATA%\launcher\)
2. apps table schema matches spec (id, name, path, icon_path, source, last_launched, launch_count)
3. db.rs functions (init_db, upsert_app, get_all_apps, increment_launch_count) compile and pass basic unit tests
4. settings.json is created with all default values on first run
5. get_settings() returns typed Settings struct; set_settings(patch) merges and persists correctly
6. Placing launcher.portable next to the exe switches data directory to ./data/ without code changes

---

## Phase 3: Indexer

**Goal:** Build the Windows application indexer: crawl all configured paths, resolve .lnk shortcuts, extract icons asynchronously, persist to SQLite, and keep the index fresh via background timer and filesystem watcher.

**Requirements**: INDX-01, INDX-02, INDX-03, INDX-04, INDX-05, INDX-06, INDX-07, INDX-08

**Plans:** 5/5 plans complete

Plans:
- [ ] 03-01-PLAN.md — Wave 0 scaffold: add lnk/notify-debouncer-mini/image crates, create generic.png, write all indexer.rs function stubs + 12 failing test stubs
- [ ] 03-02-PLAN.md — Implement path discovery, crawl_dir, resolve_lnk, make_app_record, icon_filename, prune_stale (INDX-01, INDX-02, INDX-03)
- [ ] 03-03-PLAN.md — Implement ensure_generic_icon and extract_icon_png GDI pipeline (INDX-04, INDX-05)
- [ ] 03-04-PLAN.md — Implement run_full_index assembling all primitives (INDX-01, INDX-03, INDX-04, INDX-05)
- [ ] 03-05-PLAN.md — Implement try_start_index, start_background_tasks, reindex command; wire into lib.rs (INDX-06, INDX-07, INDX-08)

**Success Criteria:**
1. Full index on startup populates the apps table with apps from Start Menu, Desktop, PATH, and user-defined paths
2. .lnk shortcuts resolve to their target executable paths
3. Stale entries (apps removed from disk) are purged from SQLite on each full index
4. Icons extracted to {data_dir}/icons/ as .png files; placeholder shown while extraction pending
5. Background re-index fires at configured interval (default 15 min)
6. Changes to Start Menu directories trigger incremental re-index within ~500ms

---

## Phase 4: Search Engine

**Goal:** Implement the search Tauri command using nucleo with MRU-weighted ranking and the > prefix for system commands.

**Requirements**: SRCH-01, SRCH-02, SRCH-03, SRCH-04, SRCH-05

**Plans:** 3/3 plans complete

Plans:
- [ ] 04-01-PLAN.md — Wave 0 scaffold: add nucleo-matcher dep, create system_command.png asset, write 13 RED test stubs in search.rs
- [ ] 04-02-PLAN.md — TDD: implement score_and_rank, match_tier, is_acronym_match, search_system_commands, ensure_system_command_icon; all 13 tests GREEN
- [ ] 04-03-PLAN.md — Wire search() Tauri command, init_search_index(), rebuild_index() into lib.rs and indexer.rs

**Success Criteria:**
1. search("") returns empty (or top MRU apps); search("ch") returns Chrome above other matches if launched most often
2. Exact prefix matches rank above fuzzy substring matches
3. Results capped at 50 items
4. "> sh" returns shutdown and sleep; "> lo" returns lock; no app results shown
5. System command results carry kind: "system" field and distinct fixed icon path

---

## Phase 5: Launcher Window UI

**Goal:** Build the complete Vue 3 launcher window: frameless layout, search input, virtualised result list, full keyboard navigation, conditional path display, admin badge, and auto-hide on focus loss.

**Requirements**: LWND-01, LWND-02, LWND-03, LWND-04, LWND-05, LWND-06, LWND-07, LWND-08, LWND-09, LWND-10, LWND-11, LWND-12

**Plans:** 4/5 plans executed

Plans:
- [ ] 05-01-PLAN.md — Rust prep: add animation field to Settings struct, add get_settings_cmd Tauri command, enable asset protocol in tauri.conf.json
- [ ] 05-02-PLAN.md — Frontend scaffold: install vue-virtual-scroller + @fontsource packages, create magnifier SVG, build complete App.vue with layout, search input, RecycleScroller result list, and window resize
- [ ] 05-03-PLAN.md — Smoke test checkpoint: center window config + human verify all 12 LWND requirements
- [ ] 05-04-PLAN.md — Gap closure: fix admin badge — add requires_elevation to SearchResult struct + TypeScript interface, decouple badge v-if from keyboard state
- [ ] 05-05-PLAN.md — Gap closure: fix window resize animation (CSS height transition + deferred setSize) and virtual scroll selection highlight (active slot prop guard)

**Success Criteria:**
1. Window is 640px wide, frameless, always-on-top, not in taskbar; height grows up to 8 result rows
2. Search input is autofocused and cleared when window appears
3. ↑/↓ navigate the list; Enter launches selected; Escape hides window; list wraps at boundaries
4. Ctrl+Shift+Enter triggers elevated launch (visual [Admin] badge appears while Ctrl+Shift held)
5. Path line for selected row appears only when show_path setting is true
6. Typing > in the input shows system command results with gear icon; no path line shown
7. Window hides automatically when it loses focus
8. Result list renders without lag for 50 items (virtualised)

---

## Phase 6: Launch Actions

**Goal:** Implement all Tauri launch commands in Rust — normal, elevated, and system commands — using windows-sys APIs.

**Requirements**: LAUN-01, LAUN-02, LAUN-03, LAUN-04

**Plans:** 1/1 plans complete

Plans:
- [ ] 06-01-PLAN.md — Implement launch, launch_elevated, run_system_command commands + invoke_handler wiring + smoke test checkpoint

**Success Criteria:**
1. launch(id) opens the target application via ShellExecuteW
2. launch_elevated(id) opens with runas verb; if user cancels UAC prompt, launcher remains open with no error displayed
3. run_system_command("lock") locks the workstation; shutdown/restart/sleep trigger correct system calls
4. All launch actions hide the launcher window immediately after execution

---

## Phase 7: Context Menu

**Goal:** Add a right-click context menu as a custom Vue HTML overlay with Settings and Quit actions.

**Requirements**: MENU-01, MENU-02, MENU-03

**Plans:** 1/2 plans executed

Plans:
- [ ] 07-01-PLAN.md — Add quit_app Rust command (commands.rs + lib.rs) and implement full context menu overlay in App.vue (state, template, CSS, keyboard/dismissal wiring)
- [ ] 07-02-PLAN.md — Human verify: right-click behavior, menu actions, Escape dismissal, click-outside, quit, menu state reset

**Success Criteria:**
1. Right-clicking anywhere on the launcher window shows the custom overlay positioned at cursor coordinates
2. Clicking Settings opens (or focuses) the settings window
3. Clicking Quit Launcher exits the process cleanly
4. Clicking outside the menu or pressing Escape dismisses it without side effects

---

## Phase 8: Settings Window

**Goal:** Build the full Settings window as a separate single-instance Tauri window with all four sections — General, Hotkey, Search, Appearance — with reactive updates to the open launcher.

**Requirements**: SETT-01, SETT-02, SETT-03, SETT-04, SETT-05, SETT-06, SETT-07

**Plans:** 4/4 plans complete

Plans:
- [ ] 08-01-PLAN.md — CSS tokens + Vite multi-page build + Rust backend (set_settings_cmd, open_settings_window, extended get_settings_cmd, tauri-plugin-dialog)
- [ ] 08-02-PLAN.md — App.vue CSS refactor to tokens + five UI primitives (Section, Row, Toggle, KeyCapture, PathList)
- [ ] 08-03-PLAN.md — Settings.vue with all four sections (General, Hotkey, Search, Appearance) + settings-main.ts entry point
- [ ] 08-04-PLAN.md — App.vue settings-changed reactive listener + Ctrl+, shortcut + human verification

**Success Criteria:**
1. Settings window opens from context menu → Settings and from Ctrl+, in the launcher
2. If already open, open_settings_window() brings it to focus rather than opening a second instance
3. Autostart toggle works on installed mode; toggle is visible but disabled in portable mode
4. Changing the hotkey in Settings takes effect immediately (old deregistered, new registered)
5. Adding/removing index paths and clicking Re-index now triggers a fresh full index
6. Changing theme, opacity, or show_path in Appearance updates the open launcher reactively without restart

---

## Phase 9: Global Hotkey

**Goal:** Register the configurable global hotkey and implement toggle show/hide behaviour with input clear-and-focus on show.

**Requirements**: HKEY-01, HKEY-02, HKEY-03

**Plans:** 1 plan

Plans:
- [ ] 09-01-PLAN.md — Implement hotkey.rs (register + update_hotkey command), wire into lib.rs, update App.vue to listen for launcher-show event and remove auto-show on mount

**Success Criteria:**
1. Alt+Space (default) toggles launcher visibility from any foreground window
2. When hotkey shows the window: it appears on the primary monitor, input is cleared, input is focused
3. When hotkey hides the window: window is hidden, previous foreground window regains focus
4. Changing hotkey in Settings instantly registers the new shortcut without app restart

---

### Phase 09.1: System Tray Icon with context menu

**Goal:** Add a system tray icon with a context menu that allows opening the settings window and quitting the app.
**Requirements**: TBD
**Depends on:** Phase 9
**Plans:** 1/1 plans complete

Plans:
- [x] TBD (run /gsd:plan-phase 09.1 to break down) (completed 2026-03-10)

## Phase 10: Packaging & Distribution

**Goal:** Configure tauri build to produce NSIS and MSI installers. Document and verify the portable build. Confirm installers work on a clean Windows machine.

**Requirements**: PACK-01, PACK-02, PACK-03, PACK-04, PACK-05

**Success Criteria:**
1. `pnpm tauri build` produces both .exe (NSIS) and .msi artifacts without errors
2. NSIS installer installs without admin rights; WebView2 bootstrapper downloads if not present
3. Installed build stores data in %APPDATA%\launcher\; autostart toggle functions correctly
4. Portable zip (exe + launcher.portable + README_portable.txt) runs from any folder
5. Portable build stores all data in ./data/ adjacent to the exe; no registry writes for autostart

---

## Summary

| # | Phase | Requirements | Status |
|---|-------|--------------|--------|
| 1 | Project Scaffold & Configuration | SCAF-02–04 | Complete |
| 2 | Data Layer | DATA-01–07 | Complete |
| 3 | Indexer | INDX-01–08 | Complete |
| 4 | Search Engine | SRCH-01–05 | Complete |
| 5 | Launcher Window UI | LWND-01–12 | Complete |
| 6 | Launch Actions | LAUN-01–04 | Complete |
| 7 | Context Menu | MENU-01–03 | Complete |
| 8 | Settings Window | SETT-01–07 | Complete |
| 9 | Global Hotkey | HKEY-01–03 | Complete |
| 10 | Packaging & Distribution | PACK-01–05 | Pending |

**10 phases** | **51 requirements** | All v1 requirements covered ✓
