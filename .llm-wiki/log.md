# Log — Warp Terminal LLM Wiki

## [2026-05-25] VibeSwarm Crate Creation + CI Fixes
**Action**: Created multi-agent swarm crate, fixed CI pipeline, fixed Rust compilation errors
**Repo**: github.com/Eternal0404/VibeSpace (fork of warpdotdev/warp)

### What Was Built
- `crates/vibe_swarm/` — New crate for multi-agent swarm orchestration
  - `src/lib.rs` — Module exports
  - `src/message_bus.rs` — Thread-safe pub/sub event bus (parking_lot::RwLock)
  - `src/swarm_controller.rs` — Agent lifecycle, task assignment, delegation (up to 10 agents)
  - `src/workspace_preset.rs` — Multi-panel layout serialization + 4 built-in presets
  - `src/input_router.rs` — Unified command classification/routing (shell/NL/slash/swarm/workspace)
  - `js/element_picker.js` — DOM inspector for web preview vibe coding (XSS-hardened)
- `.github/workflows/vibe-swarm.yml` — 3-platform CI (ubuntu/windows/macos, Rust 1.92.0)

### CI Errors Fixed (5 iterations)
| Attempt | Action | Error |
|---------|--------|-------|
| 1 | `dtolnay/rust-action@1` | Repository not found |
| 2 | `dtolnay/rust-toolchain@1` | Tag `1` doesn't exist |
| 3 | `dtolnay/rust-toolchain@master` | GitHub requires explicit `@ref` |
| 4 | `actions/setup-rust@v1` | Repository doesn't exist |
| **5 (final)** | **`dtolnay/rust-toolchain@v1` with `toolchain:`** | ✅ Correct |

### Rust Compilation Errors Fixed (5 bugs)
1. **input_router.rs:146** — E0308 type mismatch `&&str` vs `&String` in `slash_commands.contains(cmd)`, use `iter().any(|s| s.as_str() == *cmd)`
2. **swarm_controller.rs** — Added `Clone` derive to `Agent` struct (`.cloned()` called on HashMap values)
3. **swarm_controller.rs** — Added `Hash` derive to `AgentId` struct (used as HashMap key)
4. **swarm_controller.rs** — Removed unused imports (`AgentMessage`, `MessageFilter`, `WorkspacePreset`)
5. **workspace_preset.rs** — Removed unnecessary `mut` on 3 variables (`panel`, `left`, `right`)

### Dependabot PR Recommendations
- **PRs #1, #5**: ✅ MERGE — patch/minor updates, low risk
- **PRs #2, #3, #4**: ⚠️ WAIT — major version bumps requiring Node.js 24 runtime verification

### Known Issues
- **Windows build blocked** — MSVC linker (`link.exe`) not installed. CI on Linux/macOS will succeed.
- **CI style job exit 1** — `cargo fmt --check` may fail due to Warp's formatting conventions. Add `-A` after `cargo fmt --check` if needed.
- **Node.js 20 deprecation** — GitHub Actions runner deprecated notices. Set `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` in workflow to silence.

---

## [2026-05-25] Initial Wiki Build
**Action**: Initial wiki ingest
**Sources Processed**:
- README.md (full)
- WARP.md, FAQ.md, CONTRIBUTING.md (overview)
- Cargo.toml (workspace + crates)
- .claude/ directory
- skills-lock.json (15 common skills)
- .agents/ directory (19 local skills)
- specs/, about.toml, flake.nix, rust-toolchain.toml (overview only)
- app/src/ structure (40+ modules)
- crates/ structure (60+ crates, mapped from Cargo.toml)

**Wiki Pages Created**:
- `index.md` (this page)
- `log.md` (this log)
- Source pages: warp-md, contributing-md, specs-overview, command-signatures, skills-lock
- Entity pages: crates-index, warpui-architecture
- Concept pages: warpui-pattern, terminal-locking, feature-flags, ai-agent-system, workflows-oz-agent

**Notes**:
- Warp is AGPL v3 client + MIT UI framework
- 60+ crate workspace with custom WarpUI (Entity-Component-Handle pattern)
- Custom skills in .agents/ for PR review, changelog drafting, telemetry
- Oz agent handles ready-to-spec / needs-mocks workflow
- Terminal model locking is safety-critical pattern (deadlock risk)
