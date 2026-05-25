---
title: Feature Flag System
category: concept
tags: [feature-flags, warp-core, rollout, rust]
created: 2026-05-25
updated: 2026-05-25
sources: [WARP.md, crates/warp_core/src/features.rs]
---

# Feature Flag System

> [!ABSTRACT]
> Warp uses a `FeatureFlag` enum in `warp_core/src/features.rs` to manage gradual feature rollouts. Feature flags are **preferred over `#[cfg(...)]` gates** because they allow runtime rollout without recompilation and are dogfood-testable before general release.

## How It Works

```rust
// In crates/warp_core/src/features.rs
pub enum FeatureFlag {
    YourNewFeature,
    AnotherFeature,
    // ...
}
```

Usage:
```rust
if FeatureFlag::YourNewFeature.is_enabled() {
    // show new feature
}
```

## Flag Rollout Channels

| Channel | Purpose |
|---------|---------|
| `DOGFOOD_FLAGS` | Enabled by default for Warp developers (internal dogfood) |
| `PREVIEW_FLAGS` | Enabled for preview channel users |
| `RELEASE_FLAGS` | Enabled for general release |

## Adding a New Flag

1. Add variant to `FeatureFlag` enum in `crates/warp_core/src/features.rs`
2. Add default to appropriate channel (usually `DOGFOOD_FLAGS` first)
3. Use `FeatureFlag::YourFlag.is_enabled()` in code
4. Move to `PREVIEW_FLAGS` when ready for testing
5. Move to `RELEASE_FLAGS` for general availability

## Why Not `#[cfg(...)]`?

| Aspect | Feature Flags | Compile-time cfg |
|--------|-------------|-----------------|
| Rollout control | Runtime, per-channel | Compile-time, all-or-nothing |
| Testing | Can enable for dogfood only | Full rebuild needed |
| Logs | Can track enable/disable | Cannot toggle at runtime |
| Gradual rollout | Yes (preview → release) | No |

## Code Style

```rust
// GOOD: Runtime-checked feature flag
if FeatureFlag::SuperchargedCompletion.is_enabled() {
    use_supercharged_completion();
}

// BAD: Compile-time gate (harder to test, no gradual rollout)
#[cfg(feature = "supercharged")]
use_supercharged_completion();
```

---

> [!SEE ALSO]
> - [[crates-index]] — warp_core crate
> - [[warpui-architecture]] — WarpUI pattern
