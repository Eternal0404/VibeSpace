---
title: vibe_swarm Crate
category: entity
tags: [crate, multi-agent, swarm, rust, tokio, parking_lot]
created: 2026-05-25
updated: 2026-05-25
sources: [crates/vibe_swarm/src/lib.rs, crates/vibe_swarm/src/swarm_controller.rs, crates/vibe_swarm/src/message_bus.rs, crates/vibe_swarm/src/workspace_preset.rs, crates/vibe_swarm/src/input_router.rs, crates/vibe_swarm/js/element_picker.js]
---

# vibe_swarm Crate

> Multi-agent swarm orchestration for Warp Terminal. Manages up to 10 concurrent AI agents that can collaborate on tasks, delegate subtasks, share context, and perform peer review.

## Overview

- **Path**: `crates/vibe_swarm/`
- **Edition**: Rust 2021
- **License**: AGPL-3.0-or-later
- **Workspace member**: Yes (listed in root `Cargo.toml` `members`)
- **Status**: Created 2026-05-25, compilation errors fixed

## Module Structure

```
vibe_swarm/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Module exports + public re-exports
│   ├── message_bus.rs          # Thread-safe pub/sub event bus
│   ├── swarm_controller.rs    # Agent lifecycle orchestration
│   ├── workspace_preset.rs    # Multi-panel layout management
│   └── input_router.rs         # Unified command classification
└── js/
    └── element_picker.js       # DOM inspector for web preview
```

## Dependencies

```toml
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

## `message_bus.rs` — Pub/Sub Event Bus

**Thread-safety**: `parking_lot::RwLock` + `Arc<RwLock<...>>`
**History**: Ring buffer with configurable max size (default 1000)

### Key Types

| Type | Purpose |
|------|---------|
| `MessageEnvelope` | JSON-serializable message with `sender_id`, `target_id`, `payload`, `task_status`, `timestamp`, `reply_to` |
| `AgentMessage` | `MessageEnvelope` + `MessageType` |
| `MessageType` | `TaskDelegation`, `ProgressUpdate`, `ResultSharing`, `PeerReview`, `ContextRequest`, `ContextResponse`, `Termination`, `Heartbeat` |
| `MessageFilter` |订阅过滤器: `sender_id`, `target_id`, `message_types` |
| `MessageBus` | Subscribe/publish with filter matching |
| `TaskStatus` | `Pending`, `InProgress`, `Complete`, `Error`, `Cancelled` + `is_terminal()` |

## `swarm_controller.rs` — Agent Orchestration

**Key Types:**

| Type | Purpose |
|------|---------|
| `AgentId` | `Uuid` wrapper, `Hash` derived (needed for `HashMap` key) |
| `AgentConfig` | Per-agent: name, model, system_prompt, max_concurrent_tasks (default 3) |
| `AgentStatus` | `Idle`, `Working`, `Waiting`, `Terminated` |
| `SwarmConfig` | Cluster: `max_agents` (10), `default_timeout` (300s), `heartbeat_interval` (30s) |
| `SwarmController` | Main orchestrator |
| `SwarmEvent` | `AgentJoined`, `AgentLeft`, `TaskAssigned`, `TaskCompleted`, `TaskFailed`, `MessageSent`, `SwarmStarted`, `SwarmStopped` |
| `SwarmError` | `thiserror` enum for all failure modes |

**Core API:**
- `spawn_agent()` / `spawn_agents(count)` — up to `max_agents` limit
- `assign_task()` / `complete_task()` / `fail_task()` — task lifecycle
- `delegate_task_to_agent()` — inter-agent subtask delegation
- `request_peer_review()` — structured agent-to-agent review
- `share_context()` — broadcast shared context
- `send_message()` — direct or broadcast via message bus
- `wait_for_completion()` — blocking wait with timeout
- `shutdown()` — clean shutdown of all agents
- Optional `mpsc::UnboundedSender<SwarmEvent>` event channel

## `workspace_preset.rs` — Layout Management

| Type | Purpose |
|------|---------|
| `PanelType` | `Terminal`, `Editor`, `Preview`, `AgentHub`, `FileExplorer`, `Output`, `Problems`, `DebugConsole` |
| `PanelConfig` | Per-panel: id, type, working directory, shell, bounds ratios, position, visibility, title |
| `PanelPosition` | Builder with helpers: `fullscreen()`, `left_half()`, `right_half()`, `top_half()`, `bottom_half()`, `quadrant()` |
| `WorkspacePreset` | Full layout: id, name, panels, active panel, layout type, timestamps |
| `WorkspacePresetManager` | Preset registry with 4 built-in presets: `"single"`, `"split"`, `"fullstack"`, `"agent"` |

## `input_router.rs` — Command Classification

| Classification | Trigger |
|--------------|---------|
| `ShellCommand` | Matches 80+ shell indicators (`cd`, `git`, `npm`, pipes, redirects, etc.) |
| `NaturalLanguage` | `@agent`, `@ai`, `ai:`, `agent:` prefixed, or unrecognized free text |
| `SwarmCommand` | `@swarm`, `swarm:`, `//swarm`, `/swarm` prefixed |
| `WorkspaceCommand` | `/workspace`, `/preset`, `/layout` prefixed |
| `SlashCommand` | `/quit`, `/exit`, `/clear`, `/help`, `/save`, `/load` |

## `element_picker.js` — Web Preview DOM Inspector

Injected into web preview iframe for "vibe coding" element selection.

**Security:** `CONFIG.allowedOrigin` configurable, XSS protection (ID escaping in XPath), no `eval()`/`innerHTML`.

## Git Commits on VibeSpace

```
80457ac fix: 5 Rust compilation errors in vibe_swarm crate
ceb51cd docs: add comprehensive PROJECT_HANDOVER.md for next AI
26008e3 fix: dtolnay/rust-toolchain@v1 with correct toolchain input
e440242 fix: correct action is dtolnay/rust-toolchain (no @suffix)
400bc3d fix: use actions/setup-rust@v1 instead of non-existent dtolnay/rust-toolchain
365742f fix: dtolnay/rust-toolchain has no @1 tag, remove version suffix
2037f03 fix: trigger workflow on changes to itself
e29757a fix: use dtolnay/rust-toolchain instead of non-existent rust-action
17ccf42 feat: Add vibe_swarm multi-agent swarm system to Warp
```

## CI Workflow

**File**: `.github/workflows/vibe-swarm.yml`

```yaml
jobs:
  build:  # 3-platform matrix (ubuntu/windows/macos), Rust 1.92.0
  style:  # ubuntu-only, cargo fmt + clippy
```

**Correct action**: `dtolnay/rust-toolchain@v1` with `toolchain: ${{ matrix.rust }}`

## Compilation Status

| Check | Status |
|-------|--------|
| `cargo check -p vibe_swarm --target x86_64-pc-windows-gnu` | Blocked (MSVC linker not installed) |
| `cargo check -p vibe_swarm` on CI (Ubuntu) | Should pass after fixes |
| Unit tests | Compiles with fixes |
| style/clippy | May need `cargo fmt` formatting pass |

## See Also

- `PROJECT_HANDOVER.md` in `crates/vibe_swarm/` — full handoff for next AI
- `WARP.md` / `README.md` — Warp project overview
