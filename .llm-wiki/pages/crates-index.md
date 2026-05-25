---
title: Crates Index — All 60+ Rust Crates
category: entity
tags: [rust, crates, workspace, architecture]
created: 2026-05-25
updated: 2026-05-25
sources: [Cargo.toml, crates/]
---

# Crates Index — All 60+ Rust Crates

> [!ABSTRACT]
> Warp uses a **Cargo workspace with 60+ member crates** organized into logical categories: UI framework, terminal/editor, AI, networking, data/storage, shell, platform, and utilities.

## Workspace Root (Cargo.toml)

```toml
[workspace]
members = ["crates/*", "app/"]
default-members = [11 core crates]
```

## Default Members (11 Core Crates)

| Crate | Path | Purpose |
|-------|------|---------|
| `app` | `app/` | Main Tauri application binary |
| `channel_versions` | `crates/channel_versions/` | Release channel management |
| `command` | `crates/command/` | Command parsing |
| `editor` | `crates/editor/` | Text editing (vim mode) |
| `graphql` | `crates/graphql/` | GraphQL client |
| `markdown_parser` | `crates/markdown_parser/` | Markdown parsing |
| `sum_tree` | `crates/sum_tree/` | Sum tree data structure |
| `warpui` | `crates/warpui/` | Custom UI framework |
| `warp_completer` | `crates/warp_completer/` | Completion system |
| `warp_terminal` | `crates/warp_terminal/` | Terminal emulation |
| `warp_util` | `crates/warp_util/` | Utilities |

## All Crates by Category

### UI Framework
| Crate | Purpose |
|-------|---------|
| `warpui` | Custom Flutter-inspired UI framework |
| `warpui_core` | Core UI primitives and rendering |
| `warpui_extras` | UI extensions |

### Terminal & Editor
| Crate | Purpose |
|-------|---------|
| `warp_terminal` | Terminal emulation (VTE grid rendering) |
| `editor` | Text editor with vim mode |
| `warp_completer` | Completion system (v1/v2 signatures) |
| `vim` | Vim emulation |
| `syntax_tree` | Syntax tree parsing |
| `languages` | Language support |
| `lsp` | Language Server Protocol |

### AI & Code
| Crate | Purpose |
|-------|---------|
| `ai` | AI agent, prompts, context, code review, indexer |
| `computer_use` | Computer use for agents |
| `natural_language_detection` | NLD for agent mode |
| `warp_ripgrep` | Ripgrep integration for codebase search |
| `repo_metadata` | Repository metadata extraction |
| `mcp` | Model Context Protocol |

### Networking & Communication
| Crate | Purpose |
|-------|---------|
| `graphql` | GraphQL client (cynic-based) |
| `warp_graphql_schema` | GraphQL schema definitions |
| `websocket` | WebSocket support |
| `http_client` | HTTP client |
| `http_server` | HTTP server |
| `firebase` | Firebase integration |
| `ipc` | Inter-process communication |
| `jsonrpc` | JSON-RPC |

### Data & Storage
| Crate | Purpose |
|-------|---------|
| `persistence` | Database (Diesel/SQLite) + migrations |
| `settings` | Settings management |
| `settings_value` | Settings value types with derive macros |
| `settings_value_derive` | Derive macros for settings values |
| `warp_files` | File operations |
| `virtual_fs` | Virtual filesystem for testing |
| `asset_cache` | Asset caching |
| `warp_assets` | Asset embedding |
| `asset_macro` | Asset macro utilities |

### Shell & Terminal
| Crate | Purpose |
|-------|---------|
| `command` | Command parsing and execution |
| `command-signatures-v2` | Shell command signatures |
| `warp_cli` | CLI utilities |
| `node_runtime` | Node.js runtime |
| `input_classifier` | Input classification |

### Platform Integration
| Crate | Purpose |
|-------|---------|
| `app-installation-detection` | App installation detection |
| `onboarding` | Onboarding flow |
| `voice_input` | Voice input handling |
| `prevent_sleep` | Prevent system sleep |
| `managed_secrets` / `managed_secrets_wasm` | Secret management |
| `isolation_platform` | Platform isolation |
| `watcher` | File watching |
| `remote_server` | Remote server/SSH support |
| `warp_server_client` | Server client |

### Utilities
| Crate | Purpose |
|-------|---------|
| `warp_core` | Core utilities + platform abstractions |
| `warp_util` | General utilities |
| `warp_logging` | Logging infrastructure |
| `warp_features` | Feature flags |
| `warp_web_event_bus` | Web event bus |
| `sum_tree` | Sum tree data structure |
| `string-offset` | String offset handling |
| `fuzzy_match` | Fuzzy matching |
| `field_mask` | Field mask utilities |
| `channel_versions` | Channel version management |
| `handlebars` | Handlebars templating |
| `markdown_parser` | Markdown parsing |
| `simple_logger` | Simple logging |
| `integration` | Integration test framework |

### WASM & JavaScript
| Crate | Purpose |
|-------|---------|
| `serve-wasm` | WASM serving helper |
| `warp_js` | JavaScript integration |
| `ui_components` | UI component library |

## Key Dependencies (Workspace-level)

| Category | Crates |
|----------|--------|
| **Async** | tokio, async-trait, async-stream |
| **Web** | reqwest, hyper, graphql-ws-client |
| **UI** | wgpu (graphics), winit (windowing), font-kit |
| **Serialization** | serde, serde_json, bincode |
| **Database** | diesel (SQLite) |
| **AI/ML** | arborium (code analysis), natural_language_detection |
| **Logging** | sentry, tracing, log |

## Crate Naming Conventions

- `warp_*` prefix for Warp-owned crates
- `_` not `-` in crate names
- `*-value` suffix for types that derive serialization

## Notable Crate Relationships

```
warpui (UI framework)
  └── warpui_core (primitives)
  └── warpui_extras (extensions)

persistence (DB)
  └── settings (settings management)
  └── settings_value (value types)

ai (AI agent)
  ├── computer_use (agent actions)
  ├── natural_language_detection (NLD)
  └── repo_metadata (repo info extraction)

graphql (GraphQL client)
  └── warp_graphql_schema (schema definitions)
```

---

> [!SEE ALSO]
> - [[warpui-architecture]] — WarpUI Entity-Component-Handle pattern
> - [[ai-agent-system]] — AI agent architecture
> - [[persistence-crate]] — Database schema and migrations
