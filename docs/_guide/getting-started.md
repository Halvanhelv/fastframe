---
title: Getting Started
description: Add fastframe crates to an egui app, and keep the egui and winit forks working.
nav_order: 2
---

## Requirements

- Rust 1.98 or later (the `rust-version` of every crate). An app pinned to an
  older toolchain in `rust-toolchain.toml` fails to resolve; bump it first.
- egui and eframe 0.36.

## Add a crate

fastframe is not on crates.io. Depend on a release tag from GitHub:

```toml
[dependencies]
fastframe-text = { git = "https://github.com/crmne/fastframe", tag = "v0.1.7" }
fastframe-fonts = { git = "https://github.com/crmne/fastframe", tag = "v0.1.7" }
```

Use the same tag for every fastframe crate, and move it deliberately: the
[release notes](https://github.com/crmne/fastframe/releases) name every change
an app has to make to upgrade.

## A first window

Fonts and text rendering are where most apps start. At startup, before the
first frame:

```rust
use fastframe_fonts::{FontSetup, Weight};

fn setup(ctx: &egui::Context) {
    // Inter at 400, 500, 600 and 700, and installed fonts for other scripts.
    let mut fonts = FontSetup::default().definitions();

    // Hint and antialias like the desktop does.
    let rendering = fastframe_text::detect();
    rendering.apply_to(&mut fonts);
    ctx.set_fonts(fonts);

    // After the app has set its own visuals, for both themes.
    ctx.all_styles_mut(|style| rendering.apply_to_visuals(&mut style.visuals));
}

fn title(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).font(Weight::SemiBold.font_id(16.0)));
}
```

From there, each [crate page](/fastframe-text/) shows how to wire it in. The
[migration guide](/moving-apps/) lists, app by app, what each crate replaces
in [ZapFast](https://zapfast.rocks), [Spotifast](https://spotifast.rocks),
[Solco](https://getsolco.com), [TonePush](https://docs.tonepush.rocks) and
[Chat with Work](https://chatwithwork.com), which makes
it a good set of worked examples.

## egui and winit forks

The apps build against forks of egui and winit that carry fixes waiting for
upstream releases. A library cannot pin those: Cargo applies `[patch]` only
in the root workspace of the app being built, so each app keeps its own
`[patch.crates-io]` section. fastframe crates depend on the published egui
versions, and they compile against both the release and the apps' fork.

A future `forks.toml`, with a generator that writes each app's patch section
and a CI check that they agree, will keep those pins in one place. It does not
exist yet.
