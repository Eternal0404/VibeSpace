# Log — Warp Terminal LLM Wiki

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
