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
fastframe-text = { git = "https://github.com/crmne/fastframe", tag = "v0.2.3" }
fastframe-fonts = { git = "https://github.com/crmne/fastframe", tag = "v0.2.3" }
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
upstream releases. A library cannot pin those: Cargo applies `[patch]` only in
the root workspace of the app being built, so each app keeps its own
`[patch.crates-io]` section. fastframe crates depend on the published egui
versions, and they must compile against both the release and the apps' fork.

Every app on fastframe should use the revisions the apps use, so it gets the
same fixes. Copy this into the app's root `Cargo.toml`:

```toml
[patch.crates-io]
ecolor = { git = "https://github.com/crmne/egui", rev = "61a38149a4010ff9b8257dc6a03b98818b98fd02" }
eframe = { git = "https://github.com/crmne/egui", rev = "61a38149a4010ff9b8257dc6a03b98818b98fd02" }
egui = { git = "https://github.com/crmne/egui", rev = "61a38149a4010ff9b8257dc6a03b98818b98fd02" }
egui-wgpu = { git = "https://github.com/crmne/egui", rev = "61a38149a4010ff9b8257dc6a03b98818b98fd02" }
egui-winit = { git = "https://github.com/crmne/egui", rev = "61a38149a4010ff9b8257dc6a03b98818b98fd02" }
egui_extras = { git = "https://github.com/crmne/egui", rev = "61a38149a4010ff9b8257dc6a03b98818b98fd02" }
egui_glow = { git = "https://github.com/crmne/egui", rev = "61a38149a4010ff9b8257dc6a03b98818b98fd02" }
emath = { git = "https://github.com/crmne/egui", rev = "61a38149a4010ff9b8257dc6a03b98818b98fd02" }
epaint = { git = "https://github.com/crmne/egui", rev = "61a38149a4010ff9b8257dc6a03b98818b98fd02" }
epaint_default_fonts = { git = "https://github.com/crmne/egui", rev = "61a38149a4010ff9b8257dc6a03b98818b98fd02" }
winit = { git = "https://github.com/crmne/winit", rev = "fb8b24c3ec3f2c499daa92aeb4e783a8efa2e973" }
```

Patch every egui crate the app uses from the same revision, so they share one
`emath` and `epaint` (add `egui_kittest` if the app's tests use it), and move
egui and winit together: the egui revision is built against that winit, and
moving one alone does not build. Cargo warns about a patch for a crate the
app does not use (`egui_extras`, say); leave that line out.

What the forks fix:

- **Paste and file drops together on Wayland.** winit offers the clipboard and
  reports dropped files through one data device, and egui-winit uses it:
  Hyprland sends the selection and drags only to a client's first data device,
  so a second one for the clipboard broke paste (crmne/spotifast#614).
- **No freeze when a Wayland window is hidden.** eframe paces frames by the
  compositor's frame callbacks and keeps running when they stop, and winit
  reports the xdg-shell `suspended` state as `Occluded` (emilk/egui#8631,
  rust-windowing/winit#4709, rust-windowing/winit#4710).
- **No busy loop while waiting for a redraw** (emilk/egui#8398).
- **Right-to-left text shaped in its direction** (emilk/egui#8577).
- **Each emoji one emoji wide.** Families, skin tones, keycaps and flags are laid
  out as one emoji, so `fastframe-emoji`'s pictures, the cursor and widget sizes
  agree.
- **No runaway resizing when a window is dragged between monitors of different
  scale** on Windows.
- **macOS Quit through close requests**, behind a winit feature an app opts into.

On stock egui and winit, fastframe still builds and works, without these
fixes.

A future `forks.toml`, with a generator that writes each app's patch section
and a CI check that they agree, will keep those pins in one place. It does not
exist yet; until then, this section has the current revisions, and a fastframe
release that needs newer ones says so in its notes.
