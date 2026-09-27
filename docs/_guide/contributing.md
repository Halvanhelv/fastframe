---
title: Contributing
description: How pieces move into fastframe, the cross-platform and testing rules, and the checks to run.
nav_order: 3
---

fastframe grows by extraction. The full rules for contributors and coding
agents are in [AGENTS.md](https://github.com/crmne/fastframe/blob/main/AGENTS.md);
this page is the short version.

## Extracting a piece

- A piece moves in only once at least two apps have it. One app's need stays
  in that app.
- Start from the apps' existing code and behaviour, not a new design. Read how
  each app does it today and keep what works in all of them.
- Keep the public API small and named for what it does. Every public item is
  documented (`missing_docs` is on), and the crate README carries a usage
  snippet. The README is also the crate's page on this site.
- Depend on published crates.io versions, preferring dependencies already in
  the apps' trees. Justify each new dependency in a `Cargo.toml` comment.
- Moving each app onto the crate is a separate change in that app's
  repository.

## Cross-platform

- Every crate compiles on Linux, macOS and Windows. Platform code sits behind
  `cfg(target_os = ...)`, and every public item has a definition on every
  platform, even if it only returns a default or an `Unsupported` error.
- Parsers and mappings stay pure and platform-independent, so their tests run
  on all three CI platforms. Only the thin reader that talks to the OS is
  platform-specific.

## Tests

- Every behaviour has a focused test, and changed behaviour gets a regression
  test.
- Tests never touch the network, D-Bus or the registry, and never spawn
  processes. Inject readers (closures or pure functions over captured output)
  instead.

## Checks

Run all of these before sending a change:

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo clippy --locked -p fastframe-log --all-targets --no-default-features -- -D warnings
cargo test --locked --all-targets
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
cargo test --locked -p fastframe-log --no-default-features
RUSTDOCFLAGS='-D warnings' cargo doc --locked --all-features --no-deps
```

## This site

The site lives in `docs/` and is built with
[Jekyll VitePress](https://jekyll-vitepress.dev). The crate pages and the
migration guide are generated from each crate's `README.md` and from
`MIGRATION.md`, so edit those rather than anything under `docs/`. To preview:

```sh
cd docs
bundle install
bundle exec jekyll serve --livereload
```
