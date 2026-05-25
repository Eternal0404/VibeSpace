---
title: WarpUI Architecture — Entity-Component-Handle Pattern
category: entity
tags: [warpui, ui, framework, architecture, pattern]
created: 2026-05-25
updated: 2026-05-25
sources: [WARP.md, crates/warpui/]
---

# WarpUI Architecture — Entity-Component-Handle Pattern

> [!ABSTRACT]
> Warp's UI framework, **WarpUI**, is a Flutter-inspired custom framework using the **Entity-Component-Handle pattern**. A global `App` object owns all views and models as "entities," and views hold lightweight `ViewHandle<T>` references to access them. The framework is MIT-licensed (separate from AGPL app).

## Core Pattern

```
Global App
  ├── owns EntityHandle<A>  ──► holds state/Model A
  ├── owns EntityHandle<B>  ──► holds state/Model B
  └── ...
  
View (in render or event handler)
  ├── ViewHandle<A>  ──► accessing Entity A
  └── AppContext     ──► temporary access via .borrow()
```

### AppContext Usage
During `render()` and action `handle()` calls, `AppContext` provides temporary access to entity handles:

```rust
fn render(&self, ctx: &mut AppContext) {
    let state_a = ctx.borrow::<A>();   // access entity A
    let state_b = ctx.borrow::<B>();   // access entity B
    // ...
}
```

`AppContext` is passed by the framework — never construct it yourself.

### ViewHandles
Views hold `ViewHandle<T>` references (just an entity ID, not the data itself). This decouples views from actual state storage.

## MouseStateHandle Requirement

> [!CRITICAL]
> `MouseStateHandle` must be created **once** during view construction. Creating it per-event (e.g., in `handle_mouse_moved`) causes state loss between events.

```rust
struct MyView {
    mouse: MouseStateHandle,  // created once at construction
}

// In handle_mouse_moved:
// REJECTED: self.mouse = ctx.world().new_mouse_state();
// ACCEPTED: self.mouse.handle_moved(ctx, event);
```

## Terminal Model Locking — CRITICAL

> [!DANGER]
> Calling `model.lock()` on `TerminalModel` **can cause deadlocks**. The terminal model is shared across async contexts.

### Guidelines
1. **Verify no caller in current call stack already holds the lock** — if you're in a callback triggered by terminal code that already holds the lock, your `.lock()` will deadlock
2. **Prefer passing already-locked model references** rather than acquiring the lock inside the call
3. **Keep lock scope minimal** — do NOT hold the lock across awaits

### Correct Pattern
```rust
// BAD: Lock acquired inside async context
async fn some_call(&self) {
    let model = self.terminal.lock().await;  // DEADLOCK RISK
    model.do_something().await;
}

// GOOD: Accept already-locked model
async fn some_call(&self, mut model: MutexGuard<'_, TerminalModel>) {
    model.do_something();  // no await while locked
}
```

## WarpUI Crate Structure

| Crate | Purpose |
|-------|---------|
| `warpui` | Framework API (elements, actions, App, EntityHandle) |
| `warpui_core` | Core rendering primitives (no async, no platform deps) |
| `warpui_extras` | Extension UI components |

## Elements

WarpUI uses an Elm-inspired **elements** system for describing visual layout:

```rust
// Create element
let label = el::Label::new(ctx, "Hello");

// Compose into container
let row = el::Row::new(ctx)
    .push(label)
    .push(button);
```

Common elements: `Label`, `Button`, `Row`, `Column`, `Stack`, `TextField`, `Rect`, etc.

## Actions System

Actions are WarpUI's event system. Each action has a `handle()` method:

```rust
struct MyAction {
    value: String,
}

impl ActionTrait for MyAction {
    fn handle(&mut self, ctx: &mut AppContext, event: &Event) {
        // Handle the event
    }
}
```

Actions bubble up through the view hierarchy until handled.

## Feature Flag Integration

WarpUI components can query feature flags via `warp_core/src/features.rs`:

```rust
if FeatureFlag::YourFlag.is_enabled() {
    // render with new feature
}
```

This is preferred over `#[cfg(...)]` because it allows runtime rollout.

## UI Coding Guidelines

See `.agents/warp-ui-guidelines` skill for full details:
- Button themes (primary, secondary, destructive)
- Consistent component patterns
- Design system tokens

---

> [!SEE ALSO]
> - [[crates-index]] — All 60+ crates
> - [[terminal-locking]] — Terminal model locking patterns
> - [[feature-flags]] — Feature flag system
> - `.agents/warp-ui-guidelines` — UI coding guidelines skill
