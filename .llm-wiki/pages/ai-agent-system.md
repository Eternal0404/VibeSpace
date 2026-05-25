---
title: AI Agent System Architecture
category: concept
tags: [ai, agent, llm, context, computer-use]
created: 2026-05-25
updated: 2026-05-25
sources: [WARP.md, crates/ai/, app/src/ai/]
---

# AI Agent System Architecture

> [!ABSTRACT]
> Warp's AI agent system (`crates/ai/`, `app/src/ai/`) provides AI-powered coding assistance integrated directly into the terminal. It uses **context chunks**, **code review**, **arborium** for code analysis, and **natural language detection** for agent mode activation.

## Components

### crates/ai/ — AI Crate

Core AI agent implementation:
- Prompt management
- Context chunking and embedding
- Code review integration
- arborium for codebase-aware analysis
- natural_language_detection (NLD) for detecting agent mode intent

### app/src/ai/ — App-level AI Integration

- Chat UI
- Session management
- AI indexer for codebase awareness
- Prompt injection and management

## Key Features

| Feature | Description |
|---------|-------------|
| **Natural Language Detection** | Detects when user wants agent mode vs regular terminal |
| **Context Chunks** | Codebase broken into indexed chunks for relevant retrieval |
| **Code Review** | AI-powered review of code changes |
| **Arborium Integration** | Semantic code analysis engine |
| **Indexing** | Builds searchable index of repository |

## Workflow

1. User enters natural language request
2. NLD detects agent mode intent (vs regular shell command)
3. AI indexer retrieves relevant context chunks from codebase
4. LLM receives prompt + context chunks
5. Agent executes tools/actions in terminal
6. Results streamed back to user in chat UI

## Related Crates

| Crate | Role |
|-------|------|
| `ai` | Agent core |
| `computer_use` | Agent actions (file operations, shell, etc.) |
| `natural_language_detection` | Intent detection |
| `warp_ripgrep` | Codebase search |
| `repo_metadata` | Repository metadata extraction |
| `mcp` | Model Context Protocol |

---

> [!SEE ALSO]
> - [[crates-index]] — AI crates
> - [[computer-use]] — (when page exists)
