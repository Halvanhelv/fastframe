# System font and colour emoji: design note

Approved by Carmine on 2026-09-29. ZapFast draws colour emoji today
(`src/emoji.rs`), Spotifast wants them (it draws a monochrome bundled face
now), and both use `fastframe-fonts`. This note covers what moves here and
how each app chooses.

## Two independent choices

An app picks its interface font and its emoji separately. Spotifast keeps
Inter and wants system colour emoji; ZapFast wants both from the system.

| Choice | Crate | Knob |
| --- | --- | --- |
| Interface font | `fastframe-fonts` | `FontSetup::primary(Primary::Inter)` (default) or `Primary::System` |
| Emoji | `fastframe-emoji` (new) | `EmojiSetup::default().system(true).bundled(bytes)` |

No user-facing setting is needed; each app decides in code.

## Crate placement

- **System interface font** goes into `fastframe-fonts`, which already
  decides which fonts an app registers and already asks CoreText, fontconfig
  and the Windows font directories for fallbacks. `Primary::System` puts the
  platform's interface face (San Francisco, Segoe UI, fontconfig's
  `system-ui`/`sans-serif`) where Inter goes, at each registered weight, and
  realigns the script fallbacks to its baseline. It falls back to Inter when
  the face cannot be found or read. Inter moves behind a default `inter`
  feature, so an app that never draws it can leave the 880 KB out.
- **Colour emoji** get a crate of their own, `fastframe-emoji`: a different
  job (pictures, not outlines), its own dependencies (harfrust, png, and on
  Windows the `windows` crate) and its own egui glue. `fastframe-fonts` stays
  free of them.

## Public API

```rust
// Once, at startup (cheap: it only records the choice).
fastframe_emoji::EmojiSetup::default()
    .system(true)                 // the platform's emoji font
    .bundled(NOTO_COLOR_EMOJI)    // the app's own fallback, optional
    .install();
std::thread::spawn(fastframe_emoji::warm_up); // find and map the fonts

// egui glue, as ZapFast uses it today:
fastframe_emoji::append(ui, &mut job, &mut placements, text, &format);
fastframe_emoji::paint(ui, &galley, origin, &placements);
fastframe_emoji::paint_cluster(ui, cluster, rect);
let (job, clusters) = fastframe_emoji::editor_job(text, &format);

// Outside egui (Spotifast's pixel text): a premultiplied RGBA picture.
let picture = fastframe_emoji::get()?.render("👍🏽", 48);
```

Segmentation (`pieces`, `is_emoji`, `only_emoji`) moves with it, unchanged.
The bundled font is the app's bytes, not the crate's: ZapFast already ships
Noto Color Emoji (10 MB) and Spotifast does not want it.

## Per-platform drawing

Every source answers one question: "draw this whole cluster as one picture,
or say no". A cluster is tried whole, then without U+FE0F, then with its
trailing parts dropped (from PR #229), and every source gets each form
before a part is dropped, so the bundled font's exact picture beats the
system font's partial one.

| Platform | System source | How sequences join | Pixels |
| --- | --- | --- | --- |
| Linux | fontconfig's `emoji` match, else a scan of the font directories for a colour bitmap face (Noto Color Emoji, Twemoji, JoyPixels...) | harfrust applies GSUB | CBDT PNGs through skrifa |
| macOS | `/System/Library/Fonts/Apple Color Emoji.ttc`, memory-mapped | harfrust applies AAT `morx` (PR #279) | sbix PNGs through skrifa |
| Windows | Segoe UI Emoji through DirectWrite (PR #229) | DirectWrite | Direct2D into a WIC bitmap (COLR) |
| All | the app's bundled font | harfrust | CBDT through skrifa |

Linux and macOS share one pure-Rust path: shape the cluster with harfrust,
require exactly one advancing glyph (zero-advance layer glyphs are allowed:
Apple builds couples from a layer under a glyph), then composite each glyph's
bitmap into the font's own cell (advance by ascent plus descent) and scale
it to the requested height. harfrust and skrifa are already in every app's
tree through epaint. That path is tested here on Linux with a subset of Noto;
the macOS font needs a Mac.

For macOS this is preferred over CoreText (PR #229): no FFI, no per-emoji
font creation, the font is mapped rather than read (only the touched strike
pages are paged in), and the code is the Linux code. The risk is that
harfrust's `morx` support misses a sequence Apple joins; PR #279 checked
flags, skin tones, families, the rainbow flag and keycaps on macOS 26, and a
miss degrades to the bundled picture, not to a broken one. CoreText stays
the fallback plan if a Mac shows gaps.

Checked on macOS 27 against CoreText: harfrust gives CoreText's glyphs for
all 3,944 RGI sequences and the 108 people facing right, and the pictures
match CoreText's pixel for pixel, with two Apple details handled here.
The people facing right are `sbix` `flip` records (another glyph's bitmap,
mirrored), which skrifa does not read. And CoreText draws every Apple
picture an eighth of an em below the position its data give, so the
pictures are framed that way too.

Windows keeps DirectWrite: Segoe UI Emoji is COLR outlines, which a
pure-Rust painter could only draw by carrying a vector rasterizer with
gradients. The drawing and the "is it one picture" test are PR #229's,
including Windows 11's families built from overlapping part glyphs, and its
lesson that the COM factories must never be released. Segoe has no country
flags, so they come from the bundled font.

## Caching and threads

- Finding and mapping the fonts happens once per process, in `warm_up`,
  which the app runs on a thread of its own at startup (ZapFast already
  does). The 190 MB Apple font and the 10 MB Noto are mapped, never read.
- A cluster's picture is made once per egui context and kept as a texture.
  `paint_cluster` never draws a new picture on the interface thread: a
  missing picture is queued to one worker thread, the placeholder stays
  transparent for that frame, and the worker asks for a repaint when the
  picture is ready. Opening the picker with a thousand unseen emoji costs the
  interface thread nothing but texture uploads.
- `prewarm(ctx, clusters)` queues a set ahead of time (the picker's first
  page). `EmojiSetup::synchronous(true)` draws inline, for tests and demo
  screenshots that must show every emoji in their first frame.
- The harfrust shaper data and the fonts are shared by every thread;
  DirectWrite factories live in a thread-local on the worker, never freed.

## Layout, selection and copy

Unchanged from ZapFast: each emoji cluster becomes one transparent
placeholder glyph (U+2B1B), scaled to the painted picture's width and pinned
to the row height, and `placements` lists the clusters in logical order. The
galley's own selection works on the placeholder, and ZapFast's
`transcript::refine` maps each placeholder back to its cluster when copying.
The editor path (`editor_job`) keeps the real text and paints over the
transparent glyphs. The picture is framed in the emoji font's own cell, so
each platform's emoji sit on the text as that platform draws them.

## System interface font

| Platform | Face | Weights |
| --- | --- | --- |
| macOS | CoreText's system UI font (`SFNS.ttf`), mapped | variable `wght`, `opsz` held at the text cut |
| Windows | Segoe UI Variable if installed, else the message font (`Segoe UI`) | variable `wght`, else the nearest static face |
| Linux | `fc-match 'system-ui:weight=N'`, once per weight (fastframe-text already asks `fc-match`) | fontconfig's own answer per weight, `wght` set when the face is variable |

Each face must draw Latin outlines or it is skipped for Inter. Script
fallbacks are realigned to the chosen face's baseline. Faces are found once
per process, like the fallbacks.

Known difference: Inter's figures are frozen tabular; San Francisco and
Segoe UI draw proportional figures by default, so timers that count will
jitter unless the app keeps a tabular face for them.

## Binary size

- Inter is 880 KB (`InterVariable.ttf`). With `default-features = false` on
  `fastframe-fonts` and `Primary::System`, it is not linked; a machine
  without a usable system face then gets egui's own fonts.
- The bundled Noto Color Emoji is 10.2 MB and dominates. It stays the
  app's choice: needed on Windows for flags and anywhere the system lacks
  a sequence. Dropping it on Linux means no colour emoji where none is
  installed.
