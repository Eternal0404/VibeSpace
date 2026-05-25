---
title: Warp Terminal — Project Index
category: index
tags: [project, overview, warp, terminal, rust, tauri]
created: 2026-05-25
updated: 2026-05-25
sources: [README.md, WARP.md]
---

# Warp Terminal — Project Index

> [!ABSTRACT]
> Warp is an **agentic development environment born out of the terminal** — a Rust-based terminal emulator with a custom WarpUI framework, AI-powered coding agents, cloud synchronization, and cross-platform deployment.

## Project Metadata

| Field | Value |
|-------|-------|
| **Name** | Warp Terminal |
| **Language** | Rust (primary), C/C++/Swift/Kotlin (platform) |
| **Framework** | Tauri 1.x + custom WarpUI framework |
| **License** | AGPL v3 (client), MIT (UI framework) |
| **Repository** | github.com/warpdotdev/warp |
| **Rust Toolchain** | 1.92.0 |
| **Build** | `./script/run`, `./script/bootstrap` |

## Architecture Overview

```
app/              # Main Tauri binary + Rust source
  └── src/
      ├── ai/         # AI agent, prompts, indexer
      ├── terminal/   # PTY, shell, input handling
      ├── editor/     # Code editor (vim mode)
      ├── workspaces/ # Multi-workspace support
      ├── preferences/# Settings management
      └── ...         # 30+ more modules

crates/          # 60+ Rust library crates
  ├── warpui/       # Custom Flutter-inspired UI framework
  ├── warp_terminal/ # Terminal emulation (VTE)
  ├── ai/            # AI agent integration
  ├── graphql/       # GraphQL client
  ├── persistence/   # Diesel/SQLite database
  └── ...           # 55+ more crates

resources/       # Bundled skills, Linux packaging
specs/           # 70+ feature specs (product + tech pairs)
images/          # Brand assets
```

## Key Documentation Files

| File | Purpose | Priority |
|------|---------|----------|
| `WARP.md` | Engineering guide | **Required** |
| `README.md` | Project overview + build | **Required** |
| `FAQ.md` | Agent usage, contribution | Reference |
| `CONTRIBUTING.md` | Oz agent workflow | Reference |
| `TERAX.md` | Architecture doc for Terex (reference) | Reference |
| `ROADMAP.md` | Planned features (terax) | Reference |

## Crate Categories

### UI Framework
- `warpui` / `warpui_core` / `warpui_extras` — Custom Flutter-inspired UI

### Terminal & Editor
- `warp_terminal` — Terminal emulation (VTE grid)
- `editor` — Text editor with vim mode
- `warp_completer` — Completion system

### AI & Code
- `ai` — AI agent, prompts, context, code review
- `computer_use` — Computer use for agents
- `warp_ripgrep` — Codebase search integration
- `mcp` — Model Context Protocol

### Networking
- `graphql` — Cynic-based GraphQL client
- `websocket` — WebSocket support
- `firebase` — Firebase integration

### Data & Storage
- `persistence` — Diesel/SQLite
- `settings` — Settings management
- `virtual_fs` — Virtual filesystem for tests

## Agent Skills

### Common Skills (from warpdotdev/common-skills)
15 common skills: brandalf, check-impl-against-spec, council, create-pr, fix-errors, implement-specs, pr-walkthrough, reproduce-bug-report, resolve-merge-conflicts, review-pr, spec-driven-implementation, write-product-spec, write-tech-spec

### Repository-Specific Skills (in `.agents/`)
19 local skills: warp-ui-guidelines, warp-integration-test, triage-issue-local, rust-unit-tests, review-pr-local, reproduce-bug-report-local, remove-feature-flag, promote-feature, changelog-draft (9-step complex workflow), add-telemetry, add-feature-flag

## Build Commands

```bash
./script/bootstrap     # Platform setup + skills install
./script/run           # Build and run Warp
./script/presubmit    # fmt, clippy, tests

# Connect to local warp-server
cargo run --features with_local_server
```

## Core Patterns

| Pattern | Location | Notes |
|---------|----------|-------|
| WarpUI Entity-Component-Handle | `crates/warpui/` | Global App owns views via handles |
| Terminal Model Locking | `app/src/terminal/` | **CRITICAL**: model.lock() deadlock risk |
| Feature Flags | `crates/warp_core/src/features.rs` | Variants in FeatureFlag enum |
| Database | `crates/persistence/` | Diesel/SQLite schema |
| GraphQL Code Gen | `crates/graphql/` | cynic-based |

## Git History

- Last 3 commits preserved (--depth 1 clone)
- Full history available via git clone without --depth

---

## Wiki Pages

| Category | Pages |
|----------|-------|
| entities | crates-index, warpui-architecture, warp-terminal-crate, ai-crate, graphql-crate, persistence-crate |
| concepts | warpui-pattern, terminal-locking, feature-flags, ai-agent-system, warp-drive-cloud-sync, workflows-oz-agent |
| source | warp-md, contributing-md, specs-overview, command-signatures, skills-lock |

## Log

See `log.md` for full activity history.
