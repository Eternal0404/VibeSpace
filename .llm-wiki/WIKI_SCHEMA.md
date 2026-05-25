# LLM Wiki Schema — Warp Terminal

## Purpose
This wiki is a persistent, compounding knowledge base built and maintained by an LLM agent. It sits between raw sources and the user — knowledge is compiled once and kept current, not re-derived on every query.

## Architecture

### Three Layers
1. **Raw sources** — Immutable source documents (README, WARP.md, specs, code)
2. **Wiki** — LLM-generated markdown pages (summaries, entity pages, comparisons, synthesis)
3. **Schema** — This file (AGENTS.md equivalent: conventions, workflows, structure)

### Directory Structure
```
.llm-wiki/
├── sources/        # Processed source excerpts and quotes
├── pages/          # LLM-generated wiki pages
│   ├── index.md    # Content catalog
│   ├── log.md     # Chronological activity log
│   ├── entities/   # Individual pages (crates, modules, concepts)
│   ├── concepts/   # Conceptual synthesis pages
│   └── source/    # Source-processed pages
├── index.md        # Main wiki catalog
└── log.md         # Activity log
```

## Wiki Page Conventions

### Frontmatter (Required on every page)
```yaml
---
title: Page Title
category: entity | concept | source | index
tags: [tag1, tag2]
created: YYYY-MM-DD
updated: YYYY-MM-DD
sources: [source-file-reference]
---
```

### Category Definitions
- **`entity`**: Single-object pages (crate, module, function, file)
- **`concept`**: Multi-object synthesis (architecture, patterns, comparisons)
- **`source`**: Processed raw source (summary of a README section, spec, etc.)
- **`index`**: Catalog pages (this index, module index)

### Naming
- Use kebab-case for file names: `warp-ui-framework.md`
- Entity pages match actual names: `warp-terminal-crate.md`
- Concept pages are descriptive: `terminal-model-locking-patterns.md`

### Content Rules
1. Every page must have frontmatter
2. Every key claim should cite a source (inline link to source file)
3. Use `> Quote` for direct quotes from source documents
4. Flag contradictions between pages using `> [!CAUTION]` callouts
5. Link related pages using `[[Wiki Link]]` Obsidian syntax

## Ingest Workflow

When processing a new source:
1. Read the source file(s)
2. Create a `source/<source-name>.md` page summarizing key points
3. Update `index.md` with new entries
4. Update relevant existing entity/concept pages
5. Append entry to `log.md`

## Query Workflow

When answering a question:
1. Read `index.md` to find relevant pages
2. Drill into those pages and read them
3. Synthesize an answer with citations
4. If the answer is valuable, file it back as a new wiki page

## Lint Workflow

Periodically health-check the wiki:
1. Look for orphan pages (no inbound links)
2. Flag contradictions between pages
3. Note stale claims superseded by newer sources
4. Suggest missing cross-references and data gaps

## Key Pages

| Page | Purpose |
|------|---------|
| `index.md` | Master catalog of all wiki pages |
| `log.md` | Chronological append-only log of all activity |
| `entities/crates-index.md` | Map of all 60+ Rust crates |
| `concepts/warpui-architecture.md` | WarpUI Entity-Component-Handle pattern |
| `concepts/terminal-model-locking.md` | Critical deadlock patterns |
| `concepts/feature-flags.md` | Feature flag system |
| `concepts/ai-agent-system.md` | AI agent architecture |
| `source/warp-md.md` | Core WARP.md engineering guide |
| `source/command-signatures.md` | Shell command completion system |

## Core Source Documents

| Document | Must-Read Sections |
|----------|---------------------|
| `WARP.md` | WarpUI pattern, terminal locking, feature flags, coding style |
| `FAQ.md` | Agent usage, contribution workflow, licensing |
| `CONTRIBUTING.md` | Oz agent workflow, spec requirements |
| `README.md` | Project overview, build commands |
| `specs/` | Feature specs (product + tech pairs) |

## Update Policy

- Update page frontmatter `updated` date on every modification
- Log every ingest, query, and lint pass in `log.md`
- Cross-reference aggressively — the wiki compounds through connections
- Never delete from log; append only
