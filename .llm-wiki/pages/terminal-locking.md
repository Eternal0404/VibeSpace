---
title: Terminal Model Locking — Critical Deadlock Patterns
category: concept
tags: [terminal, deadlock, locking, rust, async, model]
created: 2026-05-25
updated: 2026-05-25
sources: [WARP.md, app/src/terminal/]
---

# Terminal Model Locking — Critical Deadlock Patterns

> [!DANGER]
> This is a **safety-critical pattern**. Incorrect `model.lock()` usage causes deadlocks that are difficult to debug. Read this page before touching any terminal model code.

## The Problem

`TerminalModel` is protected by a `Mutex` and shared across async contexts in the terminal subsystem. Calling `.lock()` incorrectly while the lock is already held (by the current call stack or by callers) causes **permanent deadlock**.

## Locking Rules

### Rule 1: Check Your Call Stack

Before calling `.lock()`, verify that **no caller in the current call stack already holds the lock**. This is usually non-obvious because async code can be triggered from deep call stacks.

**Dangerous pattern**: acquiring lock inside async context that was called from code that already holds the lock:

```rust
// Called from terminal input handler (which may hold the lock)
async fn terminal_input_callback(input: &str) {
    let model = terminal.lock().await;  // DEADLOCK if input handler holds it
    model.process_input(input);
}
```

### Rule 2: Accept Already-Locked Models

Prefer to accept an **already-locked model guard** as a parameter rather than acquiring the lock inside the call:

```rust
// GOOD: Takes ownership of an existing lock guard
async fn process_input(model: MutexGuard<'_, TerminalModel>, input: &str) {
    model.add_input(input);
    model.schedule_render();  // no await while holding lock
}

// CALLEE decides not to lock again
async fn on_terminal_input(model: &Mutex<TerminalModel>, input: &str) {
    let guard = model.lock().await;
    process_input(guard, input).await;  // guard already locked
}
```

### Rule 3: Never Await While Holding the Lock

Do NOT perform any await operation while holding a terminal model lock guard. Awaiting can cause the task to yield, allowing another task to attempt to acquire the lock, leading to deadlock.

```rust
// BAD: await while holding lock
async fn bad_pattern(model: MutexGuard<'_, TerminalModel>) {
    model.something();
    do_async_io().await;  // YIELDS — another task may deadlock here
    model.something_else();
}

// GOOD: Drop lock before await
async fn good_pattern(model: &Mutex<TerminalModel>, input: &str) {
    {
        let mut model = model.lock().await;
        model.add_input(input);
    }  // lock dropped before async operation
    do_async_io().await;
}
```

### Rule 4: Keep Lock Scope Minimal

Keep the lock scope as small as possible. Extreme example: don't hold the terminal model lock across an HTTP round-trip.

## Best Practices Summary

| Practice | Do | Don't |
|----------|-----|-------|
| Lock acquisition | Accept already-locked guard | `.lock()` deep in async call stacks |
| Awaiting | Drop lock before await | Hold lock across await |
| Scope | Minimal, scoped | Long-running operations while locked |
| Call stack | Audit for existing locks | Assume lock is not held |

## Debugging Deadlocks

Signs of terminal model deadlock:
- Terminal becomes unresponsive
- No output appears despite input
- CPU usage drops to zero (all tasks blocked)

If you suspect a deadlock: check the call chain for any `model.lock()` calls that may have been called from code path that already holds the lock.

---

> [!SEE ALSO]
> - [[warpui-architecture]] — WarpUI Entity-Component-Handle pattern
> - [[terminal-locking]] — (this concept)
> - `WARP.md` — Engineering guide (authoritative source)
