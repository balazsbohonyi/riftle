# Phase 4: Search Engine - Research

**Researched:** 2026-03-06
**Domain:** Rust nucleo fuzzy matching, Tauri managed state, MRU-weighted ranking
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- `search("")` returns an **empty list** — nothing shown until user types
- `search(">")` returns **all 4 system commands** immediately (discoverable)
- Any query starting with `>` triggers system command mode — `"> lo"`, `">lo"`, `"> shutdown"` all work (forgiving, no strict syntax)
- System commands **never appear in normal app results** — `>` prefix is the only way to reach them
- One **shared bundled PNG asset** for all 4 system commands (lock, shutdown, restart, sleep)
- Compiled into the binary with `include_bytes!` (same pattern as generic app icon in Phase 3)
- Copied to `{data_dir}/icons/system_command.png` at startup if missing
- `icon_path` in system command results is `"system_command.png"` (relative filename)
- Result fields: `{ id: String, name: String, icon_path: String, path: String, kind: String }`
- `path` is **always included** in every result — Phase 5 decides whether to display it based on `show_path`
- System command results have `path: ""` (empty string) and `kind: "system"`; app results have `kind: "app"`
- No `score` field — ranking is an internal detail
- Build nucleo index once at startup from DB, stored as **Tauri managed state** (`Arc<RwLock<SearchIndex>>`)
- Each `search()` call queries the cached index — no DB hit per keystroke
- After `reindex()` completes, nucleo index **rebuilds asynchronously** in the background
- Race condition handling: `Arc<RwLock<...>>` — writers swap in the new index atomically; readers always get a consistent snapshot; never returns empty during rebuild

### Claude's Discretion

- Exact nucleo API usage (matcher vs. Nucleo struct, threading model)
- MRU weighting formula — requirements say "secondary sort by launch_count" after fuzzy score tiers
- Acronym match detection logic
- Icon PNG design for system_command.png

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SRCH-01 | `search(query)` Tauri command returns ranked `Result[]` using nucleo fuzzy matching | nucleo-matcher Pattern + Matcher API; synchronous approach via pre-built in-memory index |
| SRCH-02 | Scoring order: exact prefix > acronym match > fuzzy substring; secondary sort by launch_count | Three-tier scoring: manual prefix/acronym detection + nucleo fuzzy score; composite sort key |
| SRCH-03 | Maximum 50 results returned | `.truncate(50)` after sort |
| SRCH-04 | Query starting with `>` returns only system command results (prefix-based matching) | Router branch at the top of search(); strip `> ` prefix then filter 4 static commands |
| SRCH-05 | Built-in system commands: lock, shutdown, restart, sleep — carry `kind: "system"` and fixed icon | Static `SearchResult` constants; `system_command.png` via `include_bytes!` |
</phase_requirements>

---

## Summary

Phase 4 implements the `search()` Tauri command: a pure Rust backend with no UI. The input is a query string; the output is a `Vec<SearchResult>` (capped at 50) returned as JSON to the frontend.

The architecture uses `nucleo-matcher` (the low-level crate, already pulled in as a transitive dependency of `nucleo = "^0.5"`) for synchronous per-call scoring. An in-memory index (`Vec<AppRecord>`) is built once at startup from `db::get_all_apps()` and stored as `Arc<RwLock<SearchIndex>>` managed state. Each `search()` call acquires a read lock, scores all entries against the query with a single `Matcher` instance, applies the three-tier sort, caps at 50, and returns immediately. This is the correct approach for a Tauri command handler: deterministic, fast, and testable without mocking threads.

The `>` prefix triggers a completely separate routing branch that scores against 4 static system command definitions using simple `.contains()` filtering (no fuzzy matching needed for 4 items).

**Primary recommendation:** Use `nucleo-matcher` Pattern API directly in a synchronous scoring loop over the cached `Vec<AppRecord>`. Do not use the high-level `Nucleo` struct (background threadpool, tick/snapshot model) for command-handler search — it adds threading complexity without benefit for a 300-3000 item dataset.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `nucleo-matcher` | 0.3.1 (transitive via `nucleo ^0.5`) | Fuzzy scoring engine | Already in Cargo.toml; same engine helix uses; no extra dep needed |
| `nucleo` | 0.5.0 | Pulls in `nucleo-matcher`; also available for background tick model if needed | Already declared |
| `serde` + `serde_json` | ^1 | SearchResult serialization to Tauri frontend | Already in Cargo.toml |
| `std::sync::RwLock` | stdlib | Reader/writer lock for SearchIndex managed state | No extra dep; allows concurrent reads |

### Not Needed (Do Not Add)

| What | Why |
|------|-----|
| `fuzzy-matcher` crate | Replaced by nucleo-matcher |
| `rayon` directly | nucleo already brings it; no parallel iter needed at this scale |
| `tokio` | Tauri commands are synchronous; async adds no value here |
| Additional string distance crates | nucleo-matcher covers exact prefix + fuzzy in one pass |

### Installation

nucleo is already in Cargo.toml:

```toml
nucleo = "^0.5"
```

`nucleo-matcher` is available as a direct import from the same workspace:

```toml
# nucleo-matcher is a transitive dep — import directly if needed separately
# nucleo = "^0.5" already exposes nucleo::pattern::Pattern, nucleo::Matcher etc.
```

All imports come from the `nucleo` crate re-exports or from `nucleo_matcher` crate directly — both resolve to the same binary code.

---

## Architecture Patterns

### Recommended Project Structure

```
src-tauri/src/
├── search.rs          # SearchIndex, SearchResult, search Tauri command,
│                      #   init_search_index, rebuild_index, system command defs
├── icons/
│   ├── generic.png    # Phase 3 — already exists
│   └── system_command.png  # Phase 4 — new asset, compiled with include_bytes!
└── lib.rs             # Wire: init_search_index() after run_full_index(),
                       #   register search command, call rebuild_index() after reindex()
```

### Pattern 1: SearchIndex Managed State

**What:** Wrap the scored app list in `Arc<RwLock<...>>` and store via `app.manage()`. The RwLock allows multiple concurrent readers (multiple `search()` calls) with exclusive writers only during index rebuild.

**When to use:** Any state shared between concurrent Tauri command calls.

```rust
// Source: CONTEXT.md locked decision + established Phase 3 pattern
use std::sync::{Arc, RwLock};

pub struct SearchIndex {
    pub apps: Vec<AppRecord>,  // snapshot from get_all_apps() at index build time
}

pub struct SearchIndexState(pub Arc<RwLock<SearchIndex>>);
```

Initialization in `lib.rs` `#[cfg(desktop)]` setup block:

```rust
// After run_full_index() completes in setup:
search::init_search_index(app.handle(), &app.state::<crate::db::DbState>().0);
```

### Pattern 2: SearchResult Struct (Phase 5 Contract)

**What:** The serialized return type of `search()`. Matches the field contract from CONTEXT.md exactly.

```rust
// Source: CONTEXT.md locked decision
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub icon_path: String,
    pub path: String,
    pub kind: String,   // "app" | "system"
}
```

### Pattern 3: nucleo-matcher Pattern API for Synchronous Scoring

**What:** Use `Pattern::parse()` + `Pattern::score()` to score each app in a synchronous loop. This is the correct API for a command handler — no background threads, no tick/snapshot lifecycle.

**When to use:** In-process, in-memory search over a bounded dataset (< 10,000 items).

```rust
// Source: docs.rs/nucleo-matcher/0.3.1/nucleo_matcher/pattern/struct.Pattern.html
use nucleo_matcher::{
    Matcher, Config,
    pattern::{Pattern, CaseMatching, Normalization},
    Utf32String,
};

fn score_apps(query: &str, apps: &[AppRecord]) -> Vec<(u32, &AppRecord)> {
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);

    let mut results: Vec<(u32, &AppRecord)> = apps.iter()
        .filter_map(|app| {
            let haystack = Utf32String::from(app.name.as_str());
            pattern.score(haystack.slice(..), &mut matcher)
                .map(|score| (score as u32, app))
        })
        .collect();
    results
}
```

**Key API note:** `Utf32String::from(&str)` is the standard constructor. `haystack.slice(..)` returns the `Utf32Str<'_>` that `Pattern::score()` expects.

### Pattern 4: Three-Tier Sort with MRU Secondary Key

**What:** After scoring, classify each result into a tier and sort by (tier DESC, score DESC, launch_count DESC). This implements SRCH-02 without hand-rolling a custom string distance algorithm.

```rust
// Source: CONTEXT.md requirements + SRCH-02
#[derive(Eq, PartialEq, Ord, PartialOrd)]
enum MatchTier {
    Fuzzy    = 0,  // lowest priority
    Acronym  = 1,
    Prefix   = 2,  // highest priority
}

fn match_tier(query: &str, name: &str) -> MatchTier {
    let q = query.to_lowercase();
    let n = name.to_lowercase();
    if n.starts_with(&q) {
        MatchTier::Prefix
    } else if is_acronym_match(&q, &n) {
        MatchTier::Acronym
    } else {
        MatchTier::Fuzzy
    }
}

fn is_acronym_match(query: &str, name: &str) -> bool {
    // Collect first char of each word in name; check if query matches as prefix
    let initials: String = name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect();
    initials.starts_with(query)
}

// Sort key: (tier DESC, nucleo_score DESC, launch_count DESC)
results.sort_unstable_by(|a, b| {
    b.tier.cmp(&a.tier)
        .then(b.score.cmp(&a.score))
        .then(b.app.launch_count.cmp(&a.app.launch_count))
});
```

### Pattern 5: System Command Routing

**What:** Route at the top of `search()` — if query starts with `>`, filter the 4 static system commands by the suffix. Never mix with app results.

```rust
// Source: CONTEXT.md locked decisions
const SYSTEM_COMMANDS: &[(&str, &str)] = &[
    ("system:lock",     "Lock"),
    ("system:shutdown", "Shutdown"),
    ("system:restart",  "Restart"),
    ("system:sleep",    "Sleep"),
];

fn search_system_commands(query_suffix: &str) -> Vec<SearchResult> {
    let q = query_suffix.trim().to_lowercase();
    SYSTEM_COMMANDS.iter()
        .filter(|(_, name)| q.is_empty() || name.to_lowercase().contains(&q))
        .map(|(id, name)| SearchResult {
            id: id.to_string(),
            name: name.to_string(),
            icon_path: "system_command.png".to_string(),
            path: String::new(),
            kind: "system".to_string(),
        })
        .collect()
}
```

### Pattern 6: system_command.png Copy-on-Startup

**What:** Mirror the `ensure_generic_icon()` pattern from Phase 3 exactly.

```rust
// Source: indexer.rs ensure_generic_icon() — established Phase 3 pattern
static SYSTEM_COMMAND_ICON: &[u8] = include_bytes!("../icons/system_command.png");

pub fn ensure_system_command_icon(data_dir: &Path) -> std::io::Result<()> {
    let icons_dir = data_dir.join("icons");
    std::fs::create_dir_all(&icons_dir)?;
    let dest = icons_dir.join("system_command.png");
    if !dest.exists() {
        std::fs::write(&dest, SYSTEM_COMMAND_ICON)?;
    }
    Ok(())
}
```

The `system_command.png` file must be committed to `src-tauri/icons/` before building.

### Pattern 7: Tauri Command Handler

**What:** `#[tauri::command]` function that reads managed state and returns `Vec<SearchResult>`.

```rust
// Source: Phase 3 established pattern (reindex command) + Tauri v2 docs
#[tauri::command]
pub fn search(
    query: String,
    index_state: tauri::State<SearchIndexState>,
) -> Vec<SearchResult> {
    if query.is_empty() {
        return vec![];  // SRCH-01: empty query returns empty list
    }

    // System command routing (SRCH-04)
    if query.starts_with('>') {
        let suffix = query.trim_start_matches('>').trim_start();
        return search_system_commands(suffix);
    }

    // App search (SRCH-01, SRCH-02, SRCH-03)
    let index = index_state.0.read().unwrap_or_else(|e| e.into_inner());
    let mut results = score_and_rank(&query, &index.apps);
    results.truncate(50);  // SRCH-03
    results
}
```

### Anti-Patterns to Avoid

- **Using `Nucleo` struct (high-level) inside the command handler:** The `Nucleo` struct is designed for TUI loops with background threadpools. Calling `tick()` inside a Tauri command adds latency and complexity with no benefit for a 300-3000 item dataset. Use `nucleo-matcher` Pattern API directly.
- **DB hit per keystroke:** The index is built once at startup and cached. Never call `get_all_apps()` inside `search()`.
- **Holding RwLock write during scoring:** Scoring is read-only. Acquire `read()` not `write()`. Only `rebuild_index()` acquires `write()`.
- **Panicking on poisoned lock:** Use `unwrap_or_else(|e| e.into_inner())` for lock poisoning — return stale results rather than crash.
- **Returning score field:** CONTEXT.md is explicit: no `score` field in the result struct. Phase 5 receives an ordered list only.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Fuzzy string matching algorithm | Custom edit-distance/trigram scorer | `nucleo_matcher::pattern::Pattern` | Smith-Waterman + Unicode normalization; same as helix; tested on millions of inputs |
| Case-insensitive Unicode comparison | `to_lowercase()` comparisons | `CaseMatching::Ignore` + `Normalization::Smart` in Pattern::parse | Handles Unicode folding correctly (not just ASCII) |
| Parallel scoring | Rayon par_iter over apps | `nucleo-matcher` single-thread is fast enough for < 10K items | Sub-1ms for 3000 items; adding rayon adds scheduling overhead at this scale |

**Key insight:** For a local app launcher with 300-3000 indexed apps, `nucleo-matcher`'s synchronous API scores all items in < 1ms. The background threadpool model is designed for 100K+ item datasets (file finders). Avoid over-engineering.

---

## Common Pitfalls

### Pitfall 1: Wrong Nucleo API Layer
**What goes wrong:** Using the high-level `Nucleo` struct (with `injector`, `tick`, `snapshot`) inside `search()`. The `tick()` method has a timeout parameter and is designed for a render-loop, not a command handler — behavior is undefined if called outside its intended lifecycle.
**Why it happens:** `nucleo` re-exports `Matcher` from `nucleo-matcher`, and the high-level struct is the most visible API. Docs describe it as the "main entry point."
**How to avoid:** Import directly from `nucleo_matcher::pattern::Pattern` and `nucleo_matcher::Matcher`. Use `Cargo.toml` `nucleo = "^0.5"` (already present) which pulls `nucleo-matcher 0.3.1` as a dep.
**Warning signs:** Code with `nucleo.tick()`, `nucleo.injector()`, `nucleo.snapshot()` inside a Tauri command.

### Pitfall 2: Utf32String Import Path
**What goes wrong:** `Utf32String` is in `nucleo_matcher` not `nucleo`. If importing via the `nucleo` re-export, the path may differ.
**Why it happens:** `nucleo` re-exports some but not all types from `nucleo-matcher`.
**How to avoid:** Use `use nucleo_matcher::{Matcher, Config, Utf32String}` and `use nucleo_matcher::pattern::{Pattern, CaseMatching, Normalization}`. Add `nucleo-matcher = "0.3"` to Cargo.toml directly if import paths fail.
**Warning signs:** `use nucleo::Utf32String` compile errors; E0432 unresolved import.

### Pitfall 3: RwLock Poisoning on Panic
**What goes wrong:** If `rebuild_index()` panics while holding the write lock, subsequent `read()` calls return `Err(PoisonError)` and the search breaks permanently.
**Why it happens:** Rust's `RwLock` marks itself poisoned on thread panic during write.
**How to avoid:** In the search command, use `.read().unwrap_or_else(|e| e.into_inner())` — this recovers the guard from a poisoned lock and returns stale results. In rebuild, ensure the write critical section is infallible.
**Warning signs:** `unwrap()` on `RwLock::read()` — panics on poisoned lock at runtime.

### Pitfall 4: Acronym Match Over-Matching
**What goes wrong:** Naive acronym detection (e.g., every letter in query must appear as an initial) matches too aggressively. "vs" matches "Visual Studio" AND "Video Studio" AND many more, all promoted to tier 2 above better fuzzy matches.
**Why it happens:** First-letter extraction is simplistic; multi-word queries compound the problem.
**How to avoid:** Only apply `MatchTier::Acronym` when `query.len() >= 2` AND the computed initials string starts with the full query (exact prefix on initials). If in doubt, demote to fuzzy tier — acronym is a nice-to-have.
**Warning signs:** Short single-character queries returning unexpected results at high rank.

### Pitfall 5: Empty Pattern Scoring
**What goes wrong:** `Pattern::parse("")` against a haystack may return `Some(0)` or `None` depending on crate version, causing all items to appear or disappear for empty queries.
**Why it happens:** Nucleo-matcher's behavior for empty patterns is not guaranteed stable.
**How to avoid:** Guard at the top of `search()`: `if query.is_empty() { return vec![]; }`. This is already a locked decision in CONTEXT.md.
**Warning signs:** `search("")` returning a full list of apps.

### Pitfall 6: Missing system_command.png at Compile Time
**What goes wrong:** `include_bytes!("../icons/system_command.png")` fails at compile time if the file doesn't exist in `src-tauri/icons/`.
**Why it happens:** `include_bytes!` is evaluated at compile time, not runtime.
**How to avoid:** Create the placeholder PNG in Wave 0 (stub phase) before writing any code that references it. Commit it to git.
**Warning signs:** `error: couldn't read ../icons/system_command.png: No such file or directory`.

---

## Code Examples

Verified patterns from official sources and established codebase:

### Complete search() Skeleton

```rust
// Source: CONTEXT.md locked decisions + nucleo-matcher docs
use nucleo_matcher::{
    Matcher, Config, Utf32String,
    pattern::{Pattern, CaseMatching, Normalization},
};
use serde::Serialize;
use std::sync::{Arc, RwLock};
use crate::db::AppRecord;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub icon_path: String,
    pub path: String,
    pub kind: String,
}

pub struct SearchIndex {
    pub apps: Vec<AppRecord>,
}

pub struct SearchIndexState(pub Arc<RwLock<SearchIndex>>);

static SYSTEM_COMMAND_ICON: &[u8] = include_bytes!("../icons/system_command.png");

const SYSTEM_COMMANDS: &[(&str, &str)] = &[
    ("system:lock",     "Lock"),
    ("system:shutdown", "Shutdown"),
    ("system:restart",  "Restart"),
    ("system:sleep",    "Sleep"),
];

#[tauri::command]
pub fn search(
    query: String,
    index_state: tauri::State<SearchIndexState>,
) -> Vec<SearchResult> {
    if query.is_empty() {
        return vec![];
    }

    if query.starts_with('>') {
        let suffix = query.trim_start_matches('>').trim_start();
        return search_system_commands(suffix);
    }

    let index = index_state.0.read().unwrap_or_else(|e| e.into_inner());
    let mut results = score_and_rank(&query, &index.apps);
    results.truncate(50);
    results
}
```

### init_search_index and rebuild_index

```rust
// Source: CONTEXT.md integration points
use rusqlite::Connection;
use std::sync::Mutex;
use crate::db::{DbState, get_all_apps};

pub fn init_search_index(app: &tauri::AppHandle, db: &Arc<Mutex<Connection>>) {
    let apps = {
        let conn = db.lock().unwrap();
        get_all_apps(&conn).unwrap_or_default()
    };
    let index = SearchIndex { apps };
    app.manage(SearchIndexState(Arc::new(RwLock::new(index))));
}

/// Called from the reindex() background thread after run_full_index() completes.
pub fn rebuild_index(app: &tauri::AppHandle, db: &Arc<Mutex<Connection>>) {
    let apps = {
        let conn = db.lock().unwrap();
        get_all_apps(&conn).unwrap_or_default()
    };
    let new_index = SearchIndex { apps };
    if let Ok(state) = app.try_state::<SearchIndexState>() {
        let mut guard = state.0.write().unwrap_or_else(|e| e.into_inner());
        *guard = new_index;
    }
}
```

### score_and_rank Implementation

```rust
// Source: nucleo-matcher Pattern API + SRCH-02 requirements
struct ScoredResult<'a> {
    tier: u8,       // 2=prefix, 1=acronym, 0=fuzzy
    score: u32,
    app: &'a AppRecord,
}

fn score_and_rank<'a>(query: &str, apps: &'a [AppRecord]) -> Vec<SearchResult> {
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
    let q_lower = query.to_lowercase();

    let mut scored: Vec<ScoredResult<'a>> = apps.iter()
        .filter_map(|app| {
            let haystack = Utf32String::from(app.name.as_str());
            pattern.score(haystack.slice(..), &mut matcher)
                .map(|s| ScoredResult {
                    tier: match_tier(&q_lower, &app.name.to_lowercase()),
                    score: s as u32,
                    app,
                })
        })
        .collect();

    scored.sort_unstable_by(|a, b| {
        b.tier.cmp(&a.tier)
            .then(b.score.cmp(&a.score))
            .then(b.app.launch_count.cmp(&a.app.launch_count))
    });

    scored.into_iter()
        .map(|r| SearchResult {
            id: r.app.id.clone(),
            name: r.app.name.clone(),
            icon_path: r.app.icon_path.clone().unwrap_or_else(|| "generic.png".to_string()),
            path: r.app.path.clone(),
            kind: "app".to_string(),
        })
        .collect()
}

fn match_tier(q_lower: &str, name_lower: &str) -> u8 {
    if name_lower.starts_with(q_lower) {
        2  // prefix
    } else if q_lower.len() >= 2 && is_acronym_match(q_lower, name_lower) {
        1  // acronym
    } else {
        0  // fuzzy
    }
}

fn is_acronym_match(query: &str, name: &str) -> bool {
    let initials: String = name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_lowercase();
    initials.starts_with(query)
}
```

### System Command Filtering

```rust
// Source: CONTEXT.md locked decision
fn search_system_commands(suffix: &str) -> Vec<SearchResult> {
    let q = suffix.to_lowercase();
    SYSTEM_COMMANDS.iter()
        .filter(|(_, name)| q.is_empty() || name.to_lowercase().contains(&q))
        .map(|(id, name)| SearchResult {
            id: id.to_string(),
            name: name.to_string(),
            icon_path: "system_command.png".to_string(),
            path: String::new(),
            kind: "system".to_string(),
        })
        .collect()
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `fuzzy-matcher` crate | `nucleo-matcher` | ~2023 | 6x faster; better Unicode; same fzf scoring |
| Manual Smith-Waterman | `Pattern::parse() + score()` | 2023 | No custom impl needed |
| Spin-wait for background threads | Synchronous Pattern API for small datasets | N/A (design choice) | No tick/snapshot complexity in command handlers |

**Deprecated/outdated:**
- `fuzzy-matcher` crate: replaced by nucleo ecosystem — do not add as dependency
- High-level `Nucleo` struct in command handlers: correct tool for TUI render loops, wrong tool for command-handler search

---

## Open Questions

1. **nucleo-matcher Utf32String import path from nucleo re-exports**
   - What we know: `nucleo-matcher 0.3.1` is a transitive dep of `nucleo ^0.5`. `nucleo` re-exports some types.
   - What's unclear: Whether `Utf32String` is accessible via `nucleo::Utf32String` or requires a direct `nucleo_matcher` import. The docs.rs page for `nucleo 0.5.0` doesn't enumerate re-exports explicitly.
   - Recommendation: Add `nucleo-matcher = "0.3"` as a direct dependency in Cargo.toml (Wave 0 task) and import from `nucleo_matcher` directly. This is explicit and avoids re-export ambiguity. One extra line in Cargo.toml is worth the clarity.

2. **Pattern::score() return type: `Option<u16>` vs `Option<u32>`**
   - What we know: The `Matcher::fuzzy_match()` returns `Option<u16>`. `Pattern::score()` returns `Option<u32>` per the 0.3.1 docs.
   - What's unclear: Whether this changed between minor versions. The `score as u32` cast may be unnecessary if already u32.
   - Recommendation: Compile and check — cast defensively `score as u32` will work regardless.

3. **AppRecord icon_path may be None for recently-added apps**
   - What we know: `get_all_apps()` returns `Option<String>` for `icon_path`. Icon extraction is async in Phase 3.
   - What's unclear: How often `None` occurs in practice vs `Some("generic.png")`.
   - Recommendation: `unwrap_or_else(|| "generic.png".to_string())` in the SearchResult mapping — safe fallback already shown in examples above.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`#[test]`) |
| Config file | None — `cargo test` in `src-tauri/` |
| Quick run command | `cargo test search` (runs only search module tests) |
| Full suite command | `cargo test` (all 20+ existing tests + new search tests) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SRCH-01 | `search("")` returns `vec![]` | unit | `cargo test test_search_empty_returns_empty` | ❌ Wave 0 |
| SRCH-01 | `search("ch")` returns results with nucleo scores | unit | `cargo test test_search_fuzzy_returns_matches` | ❌ Wave 0 |
| SRCH-02 | Exact prefix match ranks above fuzzy | unit | `cargo test test_search_prefix_beats_fuzzy` | ❌ Wave 0 |
| SRCH-02 | Acronym match ranks above pure fuzzy | unit | `cargo test test_search_acronym_tier` | ❌ Wave 0 |
| SRCH-02 | Higher launch_count wins ties within same tier | unit | `cargo test test_search_mru_tiebreak` | ❌ Wave 0 |
| SRCH-03 | Results capped at 50 | unit | `cargo test test_search_capped_at_50` | ❌ Wave 0 |
| SRCH-04 | `search(">")` returns all 4 system commands | unit | `cargo test test_search_system_prefix_all` | ❌ Wave 0 |
| SRCH-04 | `search("> sh")` returns shutdown + sleep | unit | `cargo test test_search_system_prefix_filtered` | ❌ Wave 0 |
| SRCH-04 | `search(">lo")` returns lock (no space required) | unit | `cargo test test_search_system_no_space` | ❌ Wave 0 |
| SRCH-04 | System command results exclude all app results | unit | `cargo test test_search_system_no_app_mixing` | ❌ Wave 0 |
| SRCH-05 | System results carry `kind: "system"` | unit | `cargo test test_system_result_kind` | ❌ Wave 0 |
| SRCH-05 | System results carry `icon_path: "system_command.png"` | unit | `cargo test test_system_result_icon` | ❌ Wave 0 |
| SRCH-05 | System results carry `path: ""` | unit | `cargo test test_system_result_path_empty` | ❌ Wave 0 |

**Note on Tauri command testing:** `#[tauri::command]` functions use managed state — unit tests call `score_and_rank()` and `search_system_commands()` directly (pure functions). Tauri state injection is tested via the existing pattern in indexer.rs (in-memory DB). No Tauri `AppHandle` mock needed if search logic is separated from state retrieval.

### Sampling Rate

- **Per task commit:** `cargo test search` (runs search module tests only, < 5s)
- **Per wave merge:** `cargo test` (full suite, all modules)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `src-tauri/icons/system_command.png` — placeholder PNG asset (16x16 or 32x32, any valid PNG)
- [ ] `nucleo-matcher = "0.3"` added to `[dependencies]` in `Cargo.toml` (direct dep for explicit imports)
- [ ] Search tests scaffolded in `search.rs` with `#[cfg(test)]` block (all tests start RED/`todo!()`)

---

## Sources

### Primary (HIGH confidence)

- `docs.rs/nucleo/0.5.0/nucleo/` — Nucleo struct, Injector, Snapshot, Status API
- `docs.rs/nucleo-matcher/0.3.1/nucleo_matcher/` — Matcher, Config, Utf32String, Pattern API
- `docs.rs/nucleo-matcher/0.3.1/nucleo_matcher/pattern/struct.Pattern.html` — Pattern::parse, match_list, score signatures
- `docs.rs/nucleo/0.5.0/nucleo/struct.Nucleo.html` — tick, injector, snapshot method signatures
- `D:\develop\projects\riftle\.claude\worktrees\musing-brown\src-tauri\Cargo.toml` — confirmed nucleo 0.5.0 + nucleo-matcher 0.3.1 in lockfile
- `D:\develop\projects\riftle\.claude\worktrees\musing-brown\src-tauri\src\indexer.rs` — established patterns: include_bytes!, ensure_generic_icon, managed state wiring
- `D:\develop\projects\riftle\.claude\worktrees\musing-brown\src-tauri\src\lib.rs` — Phase 3 integration pattern for setup() and invoke_handler

### Secondary (MEDIUM confidence)

- `docs.rs/nucleo/0.5.0/nucleo/struct.Injector.html` — push() signature with Utf32String closure
- `docs.rs/nucleo/0.5.0/nucleo/struct.Snapshot.html` — matched_item_count, matched_items iteration
- WebSearch cross-verified: nucleo-matcher is the correct low-level API; high-level Nucleo struct is for TUI render loops

### Tertiary (LOW confidence)

- `Utf32String::from(&str)` constructor form — inferred from API design and nucleo-matcher 0.3.1 docs; **must verify at compile time in Wave 0**
- `Pattern::score()` return type is `Option<u32>` — docs say u32 for Pattern but Matcher::fuzzy_match returns u16; verify with rustc

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — nucleo 0.5.0 is already declared in Cargo.toml; nucleo-matcher 0.3.1 is the transitive dep; API confirmed via docs.rs
- Architecture: HIGH — follows established Phase 3 patterns exactly (include_bytes!, ensure_icon, managed state, Arc<RwLock>); CONTEXT.md locked decisions are clear
- Pitfalls: HIGH — Utf32String import path ambiguity and lock poisoning are real Rust issues; empty pattern pitfall verified against CONTEXT.md decision
- MRU weighting formula: MEDIUM — three-tier + launch_count secondary sort is the natural reading of SRCH-02; actual formula not specified in requirements, Claude's discretion applies
- nucleo-matcher Utf32String exact constructor: LOW — must verify at compile time

**Research date:** 2026-03-06
**Valid until:** 2026-09-06 (nucleo is considered stable; core matcher implementation unlikely to change)
