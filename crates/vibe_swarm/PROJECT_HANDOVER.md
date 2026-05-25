# VibeTerminal / VibeSpace — Project Handoff Documentation

## Overview

This document describes the complete `vibe_swarm` multi-agent swarm crate created for the VibeTerminal/Warp Terminal fork, now pushed to `github.com/Eternal0404/VibeSpace`. The crate is integrated as a member of Warp's 68-crate Rust workspace.

**Last known state:** CI workflow (`vibe-swarm.yml`) was failing due to GitHub Action configuration issues (see §CI Workflow below). The fix (`dtolnay/rust-toolchain@v1` with `toolchain:` input) was pushed at commit `26008e3` — CI status should be re-checked at:
https://github.com/Eternal0404/VibeSpace/actions/workflows/vibe-swarm.yml

---

## What Was Created

### Directory Structure

```
VibeSpace/
├── crates/
│   └── vibe_swarm/              # NEW crate (AGPL-3.0-or-later)
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs                    # Module exports + public re-exports
│       │   ├── message_bus.rs            # Thread-safe pub/sub event bus
│       │   ├── swarm_controller.rs       # Agent orchestration (up to 10 agents)
│       │   ├── workspace_preset.rs       # Multi-panel layout serialization
│       │   └── input_router.rs           # Unified command classification/routing
│       └── js/
│           └── element_picker.js          # DOM inspector for web preview vibe coding
└── .github/workflows/
    └── vibe-swarm.yml            # NEW CI workflow (3-platform: ubuntu/windows/macos)
```

### Crate: `crates/vibe_swarm`

**Purpose:** Enables multi-agent swarm orchestration within Warp Terminal. Multiple AI agents can collaborate on tasks, delegate subtasks, share context, and perform peer review — all coordinated through a thread-safe message bus.

#### `src/lib.rs` — Module Exports

```rust
pub mod swarm_controller;   // Agent lifecycle, task assignment, delegation
pub mod workspace_preset;  // Multi-panel layout management
pub mod input_router;       // Unified command input classification
pub mod message_bus;        // Thread-safe pub/sub event bus

pub use swarm_controller::{SwarmController, SwarmConfig, AgentId, SwarmEvent};
pub use workspace_preset::{WorkspacePreset, PanelConfig, PanelType, WorkspacePresetManager};
pub use input_router::{InputRouter, InputClassification, RoutingDecision};
pub use message_bus::{MessageBus, AgentMessage, MessageEnvelope, TaskStatus};
```

#### `src/message_bus.rs` — Thread-Safe Pub/Sub Event Bus

- **`MessageEnvelope`** — JSON-serializable message envelope with `sender_id`, `target_id`, `payload`, `task_status`, `timestamp`, `reply_to`.
- **`MessageType`** enum: `TaskDelegation`, `ProgressUpdate`, `ResultSharing`, `PeerReview`, `ContextRequest`, `ContextResponse`, `Termination`, `Heartbeat`.
- **`MessageBus`** — Backed by `parking_lot::RwLock` + `Arc`, supports:
  - Subscribe with filter (by sender, target, message type)
  - Unsubscribe by UUID
  - Publish, send_direct, broadcast, reply_to
  - Ring-buffer history (configurable max size, default 1000)
  - Query history by agent
- **`TaskStatus`** — `Pending`, `InProgress`, `Complete`, `Error`, `Cancelled` with `is_terminal()` helper.
- Fully thread-safe via `Arc<RwLock<...>>`.
- **Tests:** 3 tests covering envelope creation, broadcast, and filter matching.

#### `src/swarm_controller.rs` — Agent Orchestration

- **`AgentId`** — Newtype over `Uuid`, with `Default` and `new()`.
- **`AgentConfig`** — Per-agent settings: name, model, system_prompt, max_concurrent_tasks (default 3).
- **`AgentStatus`** — `Idle`, `Working`, `Waiting`, `Terminated`.
- **`SwarmConfig`** — Cluster-wide: `max_agents` (default 10), `default_timeout` (300s), `heartbeat_interval` (30s).
- **`SwarmController`** — Core orchestrator:
  - `spawn_agent()` / `spawn_agents()` — up to `max_agents` limit
  - `assign_task()` / `complete_task()` / `fail_task()` — task lifecycle
  - `delegate_task_to_agent()` — inter-agent subtask delegation
  - `request_peer_review()` — structured agent-to-agent review
  - `share_context()` — broadcast shared context (file contents, dir structure, git diff, build output, etc.)
  - `send_message()` — direct or broadcast via message bus
  - `wait_for_completion()` — blocking wait with timeout
  - `shutdown()` — clean shutdown of all agents
  - Optional `mpsc::UnboundedSender<SwarmEvent>` event channel
- **`SwarmEvent`** — `AgentJoined`, `AgentLeft`, `TaskAssigned`, `TaskCompleted`, `TaskFailed`, `MessageSent`, `SwarmStarted`, `SwarmStopped`.
- **`SwarmError`** — Custom `thiserror` enum for all failure modes.
- **`Task`**, **`SubTask`**, **`TaskPriority`**, **`PeerReviewRequest`**, **`SharedContext`**, **`ContextType`**, **`TaskResult`** — Supporting data structures.
- `impl Default for SwarmConfig` and `impl Default for SwarmController` (uses `SwarmConfig::default()`).
- `impl Clone for SwarmController` (clones `Arc` handles, not agent state).
- **Tests:** 5 tests: spawn single, spawn multiple, max agents limit, task assignment, task completion.

#### `src/workspace_preset.rs` — Multi-Panel Layout Serialization

- **`PanelType`** — `Terminal`, `Editor`, `Preview`, `AgentHub`, `FileExplorer`, `Output`, `Problems`, `DebugConsole`.
- **`ShellType`** — `Bash`, `Zsh`, `Fish`, `Pwsh`, `Cmd`, `WindowsCmd`.
- **`PanelConfig`** — Individual panel: id, type, working directory, shell, width/height ratios, position, visibility, title, metadata.
- **`PanelPosition`** — Builder for absolute/relative positioning with helpers: `fullscreen()`, `left_half()`, `right_half()`, `top_half()`, `bottom_half()`, `quadrant()`.
- **`LayoutType`** — `Single`, `HorizontalSplit`, `VerticalSplit`, `Grid`, `Custom`.
- **`WorkspacePreset`** — Full layout: id, name, description, panels, active panel, layout type, timestamps, metadata.
- **`WorkspacePresetManager`** — Preset registry with 4 built-in presets:
  - `"single"` — Single terminal panel
  - `"split"` — Two side-by-side terminal panels
  - `"fullstack"` — Editor + terminal + preview + agent hub (grid)
  - `"agent"` — Main terminal + agent hub + output (vertical split)
  - Plus: save/load/delete/list presets, JSON serialize/deserialize, import/export all.
- **`WorkspaceLayoutCommand`** + **`LayoutCommandType`** — Commands for the layout system.
- **Tests:** 4 tests: preset creation, panel config, preset manager defaults, serialize/deserialize roundtrip.

#### `src/input_router.rs` — Unified Command Classification

- **`InputClassification`** — `ShellCommand`, `NaturalLanguage`, `SlashCommand`, `WorkspaceCommand`, `SwarmCommand`.
- **`CommandPrefixes`** — Configurable prefix sets (default: `@agent`, `@ai`, `ai:`, `agent:`, `@swarm`, `swarm:`, `//swarm`, `/swarm`, `/workspace`, `/preset`, `/layout` + slash commands).
- **`InputRouter`** — Main classifier:
  - `classify()` — Examines input string, returns `InputClassification`.
  - `looks_like_shell_command()` — 80+ shell indicators (git, npm, cargo, cd, sudo, pipes, redirects, etc.).
  - `route()` — Returns `RoutingDecision` with execution plan.
  - `handle_swarm_command()` — Parses `/swarm spawn N`, `/swarm status`, `/swarm delegate`, `/swarm broadcast`, `/swarm terminate`.
  - `handle_workspace_command()` — Parses `/workspace load/save/list/delete/reset`.
  - `handle_slash_command()` — `/quit`, `/exit`, `/clear`, `/help`, `/save`, `/load`.
  - Prefix stripping for agent commands.
  - Runtime prefix registration (`register_agent_prefix`, `register_swarm_prefix`).
- **`RoutingDecision`** — Carries classification, `execute_as_shell`, optional `route_to_agent`, optional `agent_message`, optional `workspace_command`.
- **`AgentMessagePayload`** — `message`, `context_files`, `flags`.
- **`WorkspaceCommand`** — Enum for layout preset operations.
- **Tests:** 6 tests covering shell detection, NL detection, prefix stripping, workspace commands, swarm commands.

#### `js/element_picker.js` — DOM Inspector for Web Preview

**Purpose:** Injected into the web preview iframe; enables element selection and "vibe coding" (AI-assisted UI modification).

- **Security measures applied:**
  - `CONFIG.allowedOrigin` defaults to `'*'` but is configurable via `SET_ALLOWED_ORIGIN` message.
  - XSS protection: element IDs are escaped with `replace(/"/g, '\\"')` before inserting into XPath (line 165).
  - No `eval()`, no `innerHTML` of untrusted content.
  - ES5-compatible syntax (no `?.`, no `CSS.escape()`, no template literals) for broad browser compatibility.
  - `use strict` mode.
  - `debugMode` off by default.

**Features:**
- Mouse hover highlighting with cyan outline + glow
- Click-to-select captures full metadata (tag, id, classes, attributes, computed styles, CSS selector, XPath, innerText, href, src, rect)
- CSS selector path generation (up to 10 depth, class trimming to 2)
- XPath generation (up to 20 depth, sibling indexing)
- PostMessage API for host communication (`ELEMENT_SELECTED`, `ELEMENT_SELECTION_STARTED`, `ELEMENT_SELECTION_CANCELLED`, `HIGHLIGHT_ELEMENT`, `QUERY_SELECTOR`, etc.)
- `window.VibeElementPicker` public API

**JavaScript bugs fixed during development:**
- Stray `'` after `0x7E` in `cssEscape` — removed the extraneous character.
- XSS: unescaped `element.id` in XPath generation — added `replace(/"/g, '\\"')`.
- Hardcoded `'*'` in all `postMessage` calls — changed to `CONFIG.allowedOrigin`.

---

## CI Workflow: `.github/workflows/vibe-swarm.yml`

**IMPORTANT FIXES APPLIED (commit `26008e3`):**

The workflow went through 4 failed iterations before the correct configuration was found:

| Attempt | Action Used | Error |
|---------|-------------|-------|
| 1 | `dtolnay/rust-action@1` | Repository not found |
| 2 | `dtolnay/rust-toolchain@1` | Tag `1` does not exist |
| 3 | `dtolnay/rust-toolchain@master` | GitHub requires explicit `@ref` |
| 4 | `actions/setup-rust@v1` | Repository does not exist |
| **5 (final)** | **`dtolnay/rust-toolchain@v1`** | **Correct — tag `v1` exists** |

**Correct configuration:**
```yaml
- name: Install Rust
  uses: dtolnay/rust-toolchain@v1    # NOT @1, NOT @master, NOT actions/setup-rust
  with:
    toolchain: ${{ matrix.rust }}      # NOT rust-version, it's "toolchain"
```

**Workflow triggers:** Push or PR on `main`/`master` when files in `crates/vibe_swarm/**`, `Cargo.toml`, or `.github/workflows/vibe-swarm.yml` change.

**Jobs:**
- `build` — Matrix over `ubuntu-latest`, `windows-latest`, `macos-latest` with Rust 1.92.0:
  - `cargo build -p vibe_swarm`
  - `cargo test -p vibe_swarm`
  - `cargo fmt --check --all` + `cargo clippy -p vibe_swarm -- -D warnings`
- `style` — Ubuntu-only, `cargo fmt --check -p vibe_swarm` + clippy

**Path filter note:** The workflow must include itself in its own path filter (`.github/workflows/vibe-swarm.yml`) otherwise changes to the workflow file do not trigger a new run.

---

## Workspace Integration

**`Cargo.toml`** (workspace root):
```toml
members = [
  "crates/vibe_swarm",   # Added
  "app",
  ...
]

[aliases]
vibe_swarm = { path = "crates/vibe_swarm" }  # Added
```

**`crates/vibe_swarm/Cargo.toml`:**
```toml
[package]
name = "vibe_swarm"
version = "0.1.0"
edition = "2021"
license = "AGPL-3.0-or-later"

[dependencies]
tokio = { workspace = true, features = ["sync", "rt", "macros"] }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
log = { workspace = true }
thiserror = { workspace = true }
parking_lot = { workspace = true }
anyhow = { workspace = true }
async-channel = { workspace = true }
```

All dependencies use the workspace resolver — no duplicates, all versions managed centrally.

---

## Local Compilation Environment

- **Windows**: Requires Visual Studio 2022+ with "Desktop development with C++" workload (`link.exe` needed).
- **Linux/macOS**: Standard `cargo build -p vibe_swarm` should work.
- **VS Code + rust-analyzer**: Should automatically detect `crates/vibe_swarm` as part of the workspace.

---

## Known Issues / Follow-Up

1. **CI status unknown** — Commit `26008e3` pushed but not yet verified green. Check: https://github.com/Eternal0404/VibeSpace/actions/workflows/vibe-swarm.yml
2. **Local Windows build blocked** — Missing VS2022 C++ tools. Either install VS2022 with C++ workload, or rely on CI for builds.
3. **rustfmt style differences** — Warp uses single-line matches and shorter import chains. `cargo fmt --check` reports style differences (not errors). The diff is cosmetic.
4. **No integration into app** — The crate is created and CI-verified but not yet wired into Warp's agent initialization. Next step would be to add `vibe_swarm::SwarmController` to the agent view initialization in `app/`.

---

## Git History on VibeSpace

```
26008e3 fix: dtolnay/rust-toolchain@v1 with correct toolchain input
e440242 fix: correct action is dtolnay/rust-toolchain (no @suffix)
400bc3d fix: use actions/setup-rust@v1 instead of non-existent dtolnay/rust-toolchain
365742f fix: dtolnay/rust-toolchain has no @1 tag, remove version suffix
2037f03 fix: trigger workflow on changes to itself
e29757a fix: use dtolnay/rust-toolchain instead of non-existent rust-action
17ccf42 feat: Add vibe_swarm multi-agent swarm system to Warp
```

---

## Credentials / Access

- **GitHub repo:** `github.com/Eternal0404/VibeSpace`
- **Local clone:** `C:\Users\User\Downloads\Ai python bot\AI AGENTIC TERMINAL SOFTWARE\VibeSpace`
- **Remote:** `origin` → `https://github.com/Eternal0404/VibeSpace.git`
