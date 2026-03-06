# Phase 2: Data Layer - Research

**Researched:** 2026-03-06
**Domain:** Rust / rusqlite / tauri-plugin-store / Tauri v2 state management
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Error Recovery:**
- Corrupted SQLite file at startup -> silent reset: delete and recreate a fresh database. Launch history and frequency counts are lost, but the app starts normally.
- Unparseable or malformed settings.json -> silent reset to defaults: overwrite with the default Settings struct. User loses customizations but app starts normally. Consistent behavior between DB and settings recovery.
- Runtime DB errors (e.g., upsert_app fails mid-index) -> return `Result<T, rusqlite::Error>`, let callers decide. A failed upsert logs a warning but doesn't crash the app. db.rs functions never swallow errors internally.

**DB Init Timing:**
- `init_db()` runs synchronously in the Tauri `setup` callback (blocking). For an empty/small SQLite file, this is <5ms. Simpler — all later modules can assume the DB is ready immediately after setup.
- DB connection shared via Tauri managed state: `Arc<Mutex<Connection>>`. Stored in app state after `init_db()`, retrieved by indexer, search, and commands via `app.state::<DbState>()`. Standard Tauri v2 pattern.
- db.rs unit tests using `Connection::open_in_memory()` in Phase 2 — DATA-03 requires this. Tests cover init_db(), upsert_app(), get_all_apps(), increment_launch_count().

**Settings Persistence:**
- `set_settings()` accepts the **full Settings struct** every time — no partial JSON patch. Frontend (Phase 8 Settings Window) always sends the complete struct. No merge logic needed in store.rs.
- `get_settings()` and `set_settings()` are **internal Rust functions only** in Phase 2. Tauri IPC commands that expose settings to the frontend are deferred to Phase 8 (or Phase 5 as needed).
- Settings struct uses `#[serde(default)]` on every field for forward compatibility. Missing fields in settings.json auto-fill with defaults — old settings files survive schema additions without triggering a reset.

**Portable Path Detection:**
- Always use `std::env::current_exe()` to locate the `riftle-launcher.portable` marker file. Same code path in dev and release. In dev, the binary is in `target/debug/` — place `riftle-launcher.portable` there to test portable mode.
- Portable path detection and data_dir resolution live in a **separate `paths.rs` module**. Both db.rs and store.rs import `paths::data_dir()` — avoids duplication, makes detection testable independently.
- `paths::data_dir()` returns a `PathBuf` AND calls `std::fs::create_dir_all()` to ensure the directory exists. Callers always get a ready-to-use path without needing to mkdir themselves.

### Claude's Discretion
- Exact type aliases or newtype wrappers for AppId / app record structs
- Logging approach (eprintln vs tracing vs log crate) for error/warning output
- Whether paths.rs uses dirs-rs crate or manual env var lookup for %APPDATA%

### Deferred Ideas (OUT OF SCOPE)
- None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DATA-01 | SQLite database initialised at startup with portable-aware path detection | `paths::data_dir()` + rusqlite `Connection::open()` in setup callback |
| DATA-02 | Schema: apps table (id, name, path, icon_path, source, last_launched, launch_count) | `CREATE TABLE IF NOT EXISTS` via `conn.execute()` in `init_db()` |
| DATA-03 | db.rs exposes init_db(), upsert_app(), get_all_apps(), increment_launch_count() | rusqlite `execute()`, `query_map()`, `INSERT OR REPLACE` pattern |
| DATA-04 | Settings persisted via tauri-plugin-store to settings.json (portable-aware path) | `app.store(data_dir.join("settings.json"))` with `StoreExt` trait |
| DATA-05 | Default settings: hotkey Alt+Space, theme system, opacity 1.0, show_path false, autostart false, additional_paths [], excluded_paths [], reindex_interval 15 | Settings struct with `#[serde(default)]` and `Default` impl |
| DATA-06 | store.rs exposes get_settings() and set_settings(patch) with typed Settings struct | `store.get("settings")` / `store.set("settings", json!)` with serde_json |
| DATA-07 | Portable mode detection — riftle-launcher.portable file adjacent to exe triggers data path switch to ./data/ | `std::env::current_exe()?.parent()?.join("riftle-launcher.portable")` existence check |
</phase_requirements>

---

## Summary

Phase 2 is a pure Rust backend phase with no frontend work. Three modules are created: `paths.rs` (new, must be added to lib.rs mod declarations), `db.rs` (fill stub), and `store.rs` (fill stub). All three coordinate around a single `data_dir()` PathBuf that is determined at startup by checking for a `riftle-launcher.portable` marker file.

The rusqlite 0.31 API is well-understood and stable. `Connection::open()` for the real DB, `Connection::open_in_memory()` for tests. The `execute()` / `prepare()` + `query_map()` pattern is idiomatic for all four db.rs functions. `INSERT OR REPLACE` handles upsert. `Arc<Mutex<Connection>>` is stored as Tauri managed state and retrieved in later phases via `app.state::<DbState>()`.

The tauri-plugin-store 2.4.2 `StoreExt` trait provides `app.store(path)` which accepts `impl AsRef<Path>`. Critically, passing an **absolute** `PathBuf` to `app.store()` works correctly: Tauri's internal path resolution calls `PathBuf::join()` under the hood, and Rust's `PathBuf::join()` replaces the base entirely when the argument is absolute — so `data_dir.join("settings.json")` passed directly to `app.store()` correctly bypasses the default `%APPDATA%` base regardless of mode. This is the correct mechanism for portable-mode settings storage.

**Primary recommendation:** Implement `paths.rs` first (it has no dependencies), then `db.rs` using the resolved data_dir, then `store.rs`. Wire both into `lib.rs` setup callback last. Add `paths.rs` to the `mod` declarations in `lib.rs`.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rusqlite | ^0.31 (bundled SQLite) | SQLite database access | Already in Cargo.toml; bundles SQLite so no system dep required |
| tauri-plugin-store | 2.4.2 (exact pin) | Settings persistence via JSON store | Already registered in lib.rs; official Tauri plugin |
| serde + serde_json | ^1 | Struct serialization for Settings | Already in Cargo.toml |
| std::env | stdlib | Portable detection via current_exe() | No additional dep needed |
| std::fs | stdlib | create_dir_all() for data_dir | No additional dep needed |

### Supporting (Claude's Discretion)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| dirs | ^5 | Cross-platform %APPDATA% lookup | Alternative to `std::env::var("APPDATA")` — adds one dep but more robust on edge cases |
| tauri Manager trait | (tauri built-in) | app.path().app_data_dir() in setup | Alternative to dirs crate — uses Tauri's own resolver which respects bundle identifier |

**Recommendation for %APPDATA% lookup (Claude's Discretion):** Use `std::env::var("APPDATA")` + hard-coded subfolder `\riftle-launcher\` rather than `app.path().app_data_dir()`. This returns `%APPDATA%\riftle-launcher\` (hardcoded for discoverability), avoids relying on the bundle identifier in the path, and is already available in the setup callback where `paths::data_dir()` is called. However, `paths::data_dir()` needs an `&AppHandle` parameter to use this approach.

**Alternative:** Use `app.path().app_data_dir()`. This uses Tauri's bundle identifier for the path but is less predictable since it depends on the bundle identifier matching the expected folder name.

**No new crates needed** — the `dirs` crate is an unnecessary addition given Tauri's built-in path resolution.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `INSERT OR REPLACE` | `INSERT ... ON CONFLICT DO UPDATE SET` | Both work; `INSERT OR REPLACE` is simpler; `ON CONFLICT DO UPDATE` is more granular but unnecessary here |
| `Arc<Mutex<Connection>>` | `Mutex<Connection>` directly | Tauri's `State<T>` already wraps in Arc internally; either works; `Arc<Mutex<>>` is the community-standard pattern for thread-spawning scenarios (Phase 3 indexer) |
| `app.store(absolute_path)` | Separate serde_json file read/write | Plugin handles file I/O, watch for changes, and JS interop; don't hand-roll |

**Installation:** No new crates to install — all deps already in Cargo.toml.

---

## Architecture Patterns

### Recommended Module Structure
```
src-tauri/src/
├── paths.rs        # NEW: data_dir() -> PathBuf; portable detection
├── db.rs           # FILL: SQLite init + CRUD; imports paths::data_dir()
├── store.rs        # FILL: get_settings() + set_settings(); imports paths::data_dir()
├── lib.rs          # UPDATE: add mod paths; wire init_db + store init in setup
└── (other stubs unchanged)
```

### Pattern 1: paths.rs — Portable-Aware Data Directory
**What:** Single function `data_dir(app: &AppHandle) -> PathBuf` that checks for marker file and returns correct data directory, creating it if needed.
**When to use:** Called once in setup, result passed around; or called on-demand (both work since it's idempotent).

```rust
// Source: std::env docs + tauri::Manager docs (verified)
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn data_dir(app: &AppHandle) -> PathBuf {
    let exe_dir = std::env::current_exe()
        .expect("cannot resolve current exe")
        .parent()
        .expect("exe has no parent directory")
        .to_path_buf();

    let portable_marker = exe_dir.join("riftle-launcher.portable");
    let dir = if portable_marker.exists() {
        exe_dir.join("data")
    } else {
        // Hardcoded via APPDATA env var — returns %APPDATA%\riftle-launcher\
        PathBuf::from(std::env::var("APPDATA").expect("APPDATA not set")).join("riftle-launcher")
    };

    std::fs::create_dir_all(&dir)
        .expect("cannot create data directory");
    dir
}
```

### Pattern 2: db.rs — init_db with CREATE TABLE IF NOT EXISTS
**What:** Opens (or creates) the SQLite file at the data_dir path, runs schema DDL, returns the Connection wrapped in Arc<Mutex<>>.
**When to use:** Called once in setup, result stored as managed state.

```rust
// Source: rusqlite docs (verified at docs.rs/rusqlite/latest)
use rusqlite::{Connection, Result};
use std::path::Path;

pub fn init_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS apps (
            id           TEXT PRIMARY KEY,
            name         TEXT NOT NULL,
            path         TEXT NOT NULL,
            icon_path    TEXT,
            source       TEXT NOT NULL,
            last_launched INTEGER,
            launch_count  INTEGER NOT NULL DEFAULT 0
        );
    ")?;
    Ok(conn)
}
```

### Pattern 3: db.rs — upsert_app with INSERT OR REPLACE
**What:** Inserts or updates an app record by primary key (id).
**When to use:** Called by Phase 3 indexer for every discovered app.

```rust
// Source: rusqlite docs + SQLite UPSERT docs (verified)
pub fn upsert_app(conn: &Connection, app: &AppRecord) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO apps
         (id, name, path, icon_path, source, last_launched, launch_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            app.id, app.name, app.path, app.icon_path,
            app.source, app.last_launched, app.launch_count
        ],
    )?;
    Ok(())
}
```

**Note on INSERT OR REPLACE:** This deletes and re-inserts the row, which resets `launch_count` to 0 if not explicitly included. The upsert MUST carry the existing `launch_count` value — the indexer should read current count first, or the planner should use `ON CONFLICT DO UPDATE SET` for name/path/icon_path only while leaving launch_count untouched. See Pitfall 2 below.

### Pattern 4: db.rs — get_all_apps with query_map
**What:** Returns all rows as a Vec<AppRecord>.
**When to use:** Called by Phase 4 search engine to build the search index.

```rust
// Source: rusqlite docs query_map example (verified at docs.rs/rusqlite/latest)
pub fn get_all_apps(conn: &Connection) -> Result<Vec<AppRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, icon_path, source, last_launched, launch_count
         FROM apps"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(AppRecord {
            id:           row.get(0)?,
            name:         row.get(1)?,
            path:         row.get(2)?,
            icon_path:    row.get(3)?,
            source:       row.get(4)?,
            last_launched: row.get(5)?,
            launch_count: row.get(6)?,
        })
    })?;
    rows.collect()
}
```

### Pattern 5: store.rs — get_settings and set_settings
**What:** get_settings() loads from store or returns defaults; set_settings() writes the full struct.
**When to use:** Called in setup for initial load; called by future Phase 8 commands on save.

```rust
// Source: tauri-plugin-store 2.x docs (verified at v2.tauri.app/plugin/store/)
use tauri_plugin_store::StoreExt;
use serde_json::json;

pub fn get_settings(app: &AppHandle, data_dir: &Path) -> Settings {
    let store_path = data_dir.join("settings.json");
    // app.store() accepts impl AsRef<Path>; absolute path bypasses AppData base
    let store = app.store(store_path).expect("cannot open settings store");

    match store.get("settings") {
        Some(val) => serde_json::from_value(val).unwrap_or_default(),
        None => Settings::default(),
    }
}

pub fn set_settings(app: &AppHandle, data_dir: &Path, settings: &Settings) {
    let store_path = data_dir.join("settings.json");
    let store = app.store(store_path).expect("cannot open settings store");
    store.set("settings", json!(settings));
    store.save().expect("cannot persist settings");
}
```

### Pattern 6: Tauri Managed State for DbState
**What:** Wrap the Connection in Arc<Mutex<>> and store it as Tauri managed state.
**When to use:** In lib.rs setup callback, after init_db() succeeds.

```rust
// Source: v2.tauri.app/develop/state-management/ (verified)
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

pub struct DbState(pub Arc<Mutex<Connection>>);

// In setup callback:
let db_path = data_dir.join("launcher.db");
let conn = db::init_db(&db_path).expect("failed to init database");
app.manage(DbState(Arc::new(Mutex::new(conn))));

// In later commands/modules:
let db = app.state::<DbState>();
let conn = db.0.lock().unwrap();
```

### Pattern 7: Unit Tests with open_in_memory
**What:** Each db.rs function tested in a `#[cfg(test)]` module using an in-memory connection.
**When to use:** All four db.rs functions need tests per DATA-03.

```rust
// Source: rusqlite docs (verified at docs.rs/rusqlite/latest)
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db_connection(&conn).unwrap(); // schema DDL extracted to take &Connection
        conn
    }

    #[test]
    fn test_upsert_and_get_all() {
        let conn = setup_test_db();
        let app = AppRecord { id: "test".into(), name: "Test App".into(), .. };
        upsert_app(&conn, &app).unwrap();
        let apps = get_all_apps(&conn).unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "Test App");
    }
}
```

**Note:** To make functions testable with `&Connection` (not `Arc<Mutex<Connection>>`), split `init_db()` into two layers:
- `init_db_connection(conn: &Connection) -> Result<()>` — executes DDL, used in tests
- `init_db(path: &Path) -> Result<Connection>` — opens file + calls `init_db_connection`

This avoids needing to lock a Mutex in unit tests.

### Anti-Patterns to Avoid
- **Swallowing errors in db.rs functions:** The CONTEXT.md mandates `Result<T, rusqlite::Error>` returned to callers. Never use `.unwrap()` or `let _ = ...` in public db.rs functions.
- **Calling `create_dir_all` in db.rs or store.rs:** `paths::data_dir()` already does this. Don't duplicate.
- **Using `with_store()` (v1 API):** This is the old tauri-plugin-store v1 pattern. Use `app.store()` via `StoreExt` instead.
- **Storing `DbState` as `Mutex<Connection>` directly:** Prefer `Arc<Mutex<Connection>>` because Phase 3 (indexer) may spawn threads that need a clone of the Arc.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Settings file I/O | Manual `File::open` + `serde_json::from_reader` | `tauri-plugin-store` via `app.store()` | Plugin handles atomic writes, file locking, JS interop, and change events |
| SQL parameter escaping | String concatenation in SQL | `rusqlite::params![]` macro | SQL injection prevention; rusqlite handles type binding correctly |
| Directory path computation | Custom platform detection | `app.path().app_data_dir()` for installed, `current_exe().parent()` for portable | Tauri path resolver handles Windows UNC paths, symlinks, and bundle ID injection |
| DB schema migration | Manual ALTER TABLE logic | `CREATE TABLE IF NOT EXISTS` + schema versioning (if needed in future phases) | Phase 2 schema is v1 with no migrations needed; keep it simple |

**Key insight:** Both SQLite file management and settings persistence have significant edge cases (file locking, atomic writes, path normalization on Windows). Using the established libraries avoids all of them.

---

## Common Pitfalls

### Pitfall 1: store plugin resolves paths relative to AppData by default
**What goes wrong:** `app.store("settings.json")` stores the file at `%APPDATA%\riftle-launcher\settings.json` regardless of portable mode. In portable mode, you want `./data/settings.json`.
**Why it happens:** `resolve_store_path()` in the plugin calls `app.path().resolve(path, BaseDirectory::AppData)`. Relative paths get the AppData prefix.
**How to avoid:** Pass the full absolute `PathBuf` to `app.store()`. Rust's `PathBuf::join()` semantics mean that when the argument is an absolute path, it replaces the base — so `data_dir.join("settings.json")` (where `data_dir` is an absolute path from `paths::data_dir()`) correctly bypasses the AppData default in Tauri's resolver.
**Warning signs:** settings.json appears in `%APPDATA%\riftle-launcher\` even when `riftle-launcher.portable` exists; portable build data is not co-located with the exe.

### Pitfall 2: INSERT OR REPLACE resets launch_count to 0
**What goes wrong:** Indexer re-runs and calls `upsert_app()` for all found apps. `INSERT OR REPLACE` deletes and re-inserts the row, resetting `launch_count` to 0 (the column default).
**Why it happens:** SQLite's `INSERT OR REPLACE` is equivalent to `DELETE` + `INSERT`, not a partial update. If `launch_count` is not explicitly included in the INSERT, it defaults to 0.
**How to avoid:** Include `launch_count` in the upsert call. The indexer should either carry the existing count (read first) or use `INSERT ... ON CONFLICT(id) DO UPDATE SET name=excluded.name, path=excluded.path, icon_path=excluded.icon_path, source=excluded.source` to update only non-launch fields. The planner should decide which approach; the safer one for Phase 3 compatibility is the `ON CONFLICT DO UPDATE` variant.
**Warning signs:** Launch frequency counts disappear after any re-index cycle.

### Pitfall 3: rusqlite::Connection is not Sync
**What goes wrong:** Attempting to share a raw `Connection` across threads (e.g., passing to a spawned thread) causes a compile error because `Connection: !Sync`.
**Why it happens:** rusqlite enforces thread safety at compile time. `Connection` is `Send` but not `Sync`.
**How to avoid:** Always wrap in `Mutex<Connection>`. With `Arc<Mutex<Connection>>`, the Arc can be cloned and sent to multiple threads; each thread locks before use.
**Warning signs:** Compiler error "... cannot be sent between threads safely" when Phase 3 indexer tries to use the DbState across async/threaded boundaries.

### Pitfall 4: init_db_connection signature vs unit testability
**What goes wrong:** If `init_db()` opens the file and runs DDL in a single function taking a `&Path`, tests must either mock the filesystem or create temp files — both are awkward.
**Why it happens:** Tests want to use `Connection::open_in_memory()` directly without a file path.
**How to avoid:** Split into two layers: `init_db_connection(&Connection) -> Result<()>` for DDL (testable), and `init_db(&Path) -> Result<Connection>` for the full flow. Tests call `init_db_connection` directly on an in-memory connection.
**Warning signs:** Test module cannot call db functions without creating a temp file path.

### Pitfall 5: Missing `mod paths;` declaration in lib.rs
**What goes wrong:** Creating `paths.rs` without adding `mod paths;` to `lib.rs` causes "file not found" compile errors in db.rs and store.rs when they `use crate::paths::data_dir`.
**Why it happens:** Rust requires explicit module declarations; creating the file is not enough.
**How to avoid:** Add `mod paths;` to `lib.rs` alongside the existing module stubs as the very first task.
**Warning signs:** `error[E0432]: unresolved import 'crate::paths'` on first compile.

### Pitfall 6: serde_json::Value vs typed Settings struct round-trip
**What goes wrong:** `store.get("settings")` returns `Option<serde_json::Value>`. Calling `.unwrap()` and assuming the shape matches panics if the stored value was malformed.
**Why it happens:** The store persists raw JSON; if the schema changed or the file was manually edited, deserialization may fail.
**How to avoid:** Use `serde_json::from_value(val).unwrap_or_default()` in `get_settings()`. The `#[serde(default)]` on every Settings field means partial JSON gracefully fills missing fields with defaults — this is the forward-compatibility mechanism locked in CONTEXT.md.
**Warning signs:** App crashes on startup after a settings schema addition if `unwrap()` is used instead of `unwrap_or_default()`.

---

## Code Examples

Verified patterns from official sources:

### Settings Struct with serde defaults
```rust
// Source: serde documentation (https://serde.rs/field-attrs.html)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_opacity")]
    pub opacity: f64,

    #[serde(default)]
    pub show_path: bool,        // bool default is false

    #[serde(default)]
    pub autostart: bool,

    #[serde(default)]
    pub additional_paths: Vec<String>,

    #[serde(default)]
    pub excluded_paths: Vec<String>,

    #[serde(default = "default_reindex_interval")]
    pub reindex_interval: u32,
}

fn default_hotkey() -> String { "Alt+Space".to_string() }
fn default_theme() -> String { "system".to_string() }
fn default_opacity() -> f64 { 1.0 }
fn default_reindex_interval() -> u32 { 15 }

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: default_hotkey(),
            theme: default_theme(),
            opacity: default_opacity(),
            show_path: false,
            autostart: false,
            additional_paths: vec![],
            excluded_paths: vec![],
            reindex_interval: default_reindex_interval(),
        }
    }
}
```

### Portable detection in paths.rs
```rust
// Source: std::env docs (https://doc.rust-lang.org/std/env/fn.current_exe.html)
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn data_dir(app: &AppHandle) -> PathBuf {
    let exe_path = std::env::current_exe()
        .expect("cannot resolve current exe path");
    let exe_dir = exe_path.parent()
        .expect("exe has no parent directory")
        .to_path_buf();

    let dir = if exe_dir.join("riftle-launcher.portable").exists() {
        exe_dir.join("data")
    } else {
        PathBuf::from(std::env::var("APPDATA").expect("APPDATA not set")).join("riftle-launcher")
    };

    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|e| panic!("cannot create data dir {:?}: {}", dir, e));
    dir
}
```

### lib.rs setup wiring
```rust
// Source: v2.tauri.app/develop/state-management/ (verified)
use std::sync::{Arc, Mutex};

.setup(|app| {
    #[cfg(desktop)]
    {
        // ... existing plugin registration ...
    }

    // Phase 2: Data layer initialization
    let data_dir = crate::paths::data_dir(app.handle());

    // Init SQLite
    let db_path = data_dir.join("launcher.db");
    let conn = crate::db::init_db(&db_path)
        .expect("failed to initialize database");
    app.manage(crate::db::DbState(Arc::new(Mutex::new(conn))));

    // Init settings store (load to verify defaults are written on first run)
    let _settings = crate::store::get_settings(app.handle(), &data_dir);

    Ok(())
})
```

### increment_launch_count
```rust
// Source: rusqlite execute() docs (verified at docs.rs/rusqlite/latest)
pub fn increment_launch_count(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "UPDATE apps SET
             launch_count = launch_count + 1,
             last_launched = ?1
         WHERE id = ?2",
        rusqlite::params![
            chrono::Utc::now().timestamp(),  // or SystemTime — see note below
            id
        ],
    )?;
    Ok(())
}
```

**Note:** `last_launched` is stored as `INTEGER` (Unix timestamp). Prefer `std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64` to avoid adding the `chrono` crate (not in Cargo.toml). The column is `INTEGER` in the schema.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `with_store()` helper | `StoreExt::store()` on AppHandle | tauri-plugin-store v2 | Simpler API; v1 `with_store` still compiles but is discouraged |
| `app.path_resolver()` (v1) | `app.path()` (v2) | Tauri 2.0 stable | `path_resolver()` is removed in v2; must use `app.path()` |
| Manual `INSERT` + `UPDATE` upsert | `INSERT OR REPLACE` or `ON CONFLICT DO UPDATE` | SQLite 3.24+ (2018) | Bundled SQLite in rusqlite is current; both syntax forms available |
| `Mutex<Connection>` as managed state | `Arc<Mutex<Connection>>` | Always | `Arc<>` needed when Connection must be cloned across thread boundaries |

**Deprecated/outdated:**
- `app.path_resolver()`: Removed in Tauri v2. Use `app.path()` instead.
- `StoreCollection` + `with_store()`: v1 pattern. Use `StoreExt` in v2.
- `tauri_plugin_store::Builder::default()`: Still works but `Builder::new()` is the current form.

---

## Open Questions

1. **Does passing an absolute PathBuf to `app.store()` truly bypass BaseDirectory::AppData?**
   - What we know: `resolve_store_path()` calls `app.path().resolve(path, BaseDirectory::AppData)`. Tauri's `resolve()` is not a plain `PathBuf::join()` call. The implementation was confirmed to use `dunce::simplified()` and delegate to an internal function.
   - What's unclear: Whether Tauri's `path().resolve()` respects absolute paths the way `PathBuf::join()` does (i.e., replaces base when arg is absolute).
   - Recommendation: **Test this in the first implementation task.** Create a `data_dir` at `C:\temp\riftle_test\`, call `app.store(data_dir.join("settings.json"))`, and verify `settings.json` appears at `C:\temp\riftle_test\` (not in `%APPDATA%`). If absolute paths are NOT respected, the fallback is to use `std::fs` for direct JSON read/write in `store.rs` instead of the plugin — this is the backup plan.

2. **AppRecord struct: use String id or hash?**
   - What we know: `id` is `TEXT PRIMARY KEY` in the schema. Source is not specified.
   - What's unclear: Whether the planner should define `id` as the full exe path (natural key) or a hash thereof.
   - Recommendation: Use the normalized exe path as the id string — it is naturally unique, human-readable in the DB, and doesn't require a hash crate. The planner should decide.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | None needed — `#[cfg(test)]` modules in source files |
| Quick run command | `cargo test -p riftle --lib db` |
| Full suite command | `cargo test -p riftle` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DATA-01 | DB created at correct path | Integration (manual smoke) | `cargo test -p riftle` + verify file on disk | N/A — smoke |
| DATA-02 | apps table has correct schema | Unit | `cargo test -p riftle --lib db::tests::test_schema` | Wave 0 |
| DATA-03 | init_db, upsert_app, get_all_apps, increment_launch_count compile and pass | Unit | `cargo test -p riftle --lib db::tests` | Wave 0 |
| DATA-04 | settings.json created at correct path | Integration (manual smoke) | `cargo test -p riftle` + verify file | N/A — smoke |
| DATA-05 | get_settings returns all default values on first run | Unit | `cargo test -p riftle --lib store::tests::test_defaults` | Wave 0 |
| DATA-06 | set_settings persists full struct; get_settings reads it back | Unit | `cargo test -p riftle --lib store::tests::test_round_trip` | Wave 0 |
| DATA-07 | portable marker causes data_dir to return exe_dir/data | Unit | `cargo test -p riftle --lib paths::tests::test_portable_detection` | Wave 0 |

**Note on DATA-04/DATA-06 store tests:** `tauri-plugin-store`'s `app.store()` requires a live `AppHandle`. Unit tests cannot easily construct one. Options:
1. Extract Settings serialization/deserialization logic into pure functions testable without AppHandle, and test those in isolation.
2. Accept that store.rs integration tests are manual/smoke-only in Phase 2, with unit tests for the Settings struct shape only.

**Recommendation:** Test the Settings struct `Default` impl and serde round-trip as pure unit tests. Store integration (actual file write/read) is verified by the smoke test when `pnpm tauri dev` launches successfully and settings.json appears on disk.

### Sampling Rate
- **Per task commit:** `cargo test -p riftle --lib` (all lib unit tests, ~1s)
- **Per wave merge:** `cargo test -p riftle` (full suite)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/db.rs` — `#[cfg(test)] mod tests` block covering DATA-02, DATA-03
- [ ] `src-tauri/src/store.rs` — `#[cfg(test)] mod tests` block covering DATA-05 (Settings Default), DATA-06 (serde round-trip)
- [ ] `src-tauri/src/paths.rs` — `#[cfg(test)] mod tests` block covering DATA-07 (portable path logic, using a tempdir)

---

## Sources

### Primary (HIGH confidence)
- [docs.rs/rusqlite/latest](https://docs.rs/rusqlite/latest/rusqlite/) — Connection::open_in_memory, execute, prepare, query_map, INSERT OR REPLACE
- [v2.tauri.app/plugin/store/](https://v2.tauri.app/plugin/store/) — StoreExt trait, app.store(), set/get/save API
- [v2.tauri.app/develop/state-management/](https://v2.tauri.app/develop/state-management/) — app.manage(), State<T>, Mutex<T> pattern
- [docs.rs/tauri-plugin-store/2.4.2](https://docs.rs/crate/tauri-plugin-store/latest) — Store struct methods, StoreExt, StoreBuilder
- [github.com/tauri-apps/plugins-workspace store/src/lib.rs](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/store/src/lib.rs) — StoreExt path type: `impl AsRef<Path>`
- [github.com/tauri-apps/plugins-workspace store/src/store.rs](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/store/src/store.rs) — resolve_store_path uses BaseDirectory::AppData

### Secondary (MEDIUM confidence)
- [std::env::current_exe docs](https://doc.rust-lang.org/std/env/fn.current_exe.html) — returns full path to binary, works in dev and release
- [Rust PathBuf::join behavior](https://doc.rust-lang.org/std/path/struct.PathBuf.html) — absolute path arg replaces base entirely
- [serde field attributes](https://serde.rs/field-attrs.html) — `#[serde(default)]` and `#[serde(default = "fn")]` behavior

### Tertiary (LOW confidence — needs validation)
- Whether Tauri's `path().resolve()` with an absolute path argument truly bypasses the `BaseDirectory::AppData` prefix — confirmed via source code inspection that `resolve_store_path` calls `app.path().resolve(path, BaseDirectory::AppData)` but the exact behavior of `resolve()` with an absolute path arg was not directly verified in compiled tests.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all crates already in Cargo.toml; APIs verified via official docs
- Architecture: HIGH — patterns follow verified Tauri v2 and rusqlite docs; locked decisions from CONTEXT.md reduce ambiguity
- Pitfalls: HIGH — rusqlite thread-safety and INSERT OR REPLACE reset are documented; store path resolution inspected at source level
- Open question on absolute path + store: LOW — source-inspected but not runtime-verified

**Research date:** 2026-03-06
**Valid until:** 2026-06-06 (stable crates; Tauri 2.x minor updates unlikely to break these APIs)
