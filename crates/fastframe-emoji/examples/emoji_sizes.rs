//! Renders a line of text and emoji at several sizes through the real egui
//! path (fonts from `fastframe-fonts`, the system emoji font, the
//! [`fastframe_emoji::EmojiPlugin`]) into a PNG, with a small software
//! rasterizer, so emoji sizing can be checked by eye without a window:
//!
//! ```sh
//! cargo run -p fastframe-emoji --example emoji_sizes -- sizes.png
//! ```
//!
//! A grey line marks each row's baseline. The emoji should all be one size
//! in a row, sit on the text the same way, and take one emoji's width.

use std::collections::HashMap;
use std::io::Write as _;

use egui::epaint::{ClippedPrimitive, Primitive, Vertex};
use egui::{Color32, ColorImage, FontId, Pos2, Rect, Stroke, TextureId, vec2};

const LINE: &str = "Text 😀 family 👨‍👩‍👧‍👦 coder 👩🏽‍💻 pride 🏳️‍🌈 one 1️⃣ hash #️⃣ italy 🇮🇹 thumbs 👍🏽 end";
const SIZES: [f32; 4] = [10.0, 13.0, 16.0, 24.0];
const PIXELS_PER_POINT: f32 = 2.0;
const WIDTH: f32 = 980.0;

fn main() -> std::io::Result<()> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "emoji-sizes.png".to_owned());
    fastframe_emoji::EmojiSetup::default()
        .system(true)
        .synchronous(true)
        .install();
    fastframe_emoji::warm_up();

    let ctx = egui::Context::default();
    ctx.set_fonts(fastframe_fonts::FontSetup::default().definitions());
    ctx.set_visuals(egui::Visuals::dark());
    ctx.set_zoom_factor(PIXELS_PER_POINT);
    ctx.add_plugin(fastframe_emoji::EmojiPlugin::default());

    let height = 30.0 + SIZES.iter().map(|size| size * 2.2 + 8.0).sum::<f32>();
    let screen = Rect::from_min_size(Pos2::ZERO, vec2(WIDTH, height));
    let mut textures: HashMap<TextureId, ColorImage> = HashMap::new();
    let mut output = None;
    // A first frame to settle fonts, a second to draw.
    for _ in 0..2 {
        let input = egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        };
        let mut full = ctx.run_ui(input, |ui| {
            let painter = ui.painter();
            painter.rect_filled(screen, 0.0, Color32::from_gray(24));
            let mut y = 16.0;
            for size in SIZES {
                let galley = painter.layout_no_wrap(
                    format!("{size}  {LINE}"),
                    FontId::proportional(size),
                    Color32::from_gray(230),
                );
                let origin = Pos2::new(12.0, y);
                let baseline = origin.y + galley.rows[0].pos.y + galley.rows[0].glyphs[0].pos.y;
                painter.hline(
                    origin.x..=origin.x + galley.size().x,
                    baseline,
                    Stroke::new(1.0 / PIXELS_PER_POINT, Color32::from_gray(90)),
                );
                painter.galley(origin, galley.clone(), Color32::from_gray(230));
                y += size * 2.2 + 8.0;
            }
        });
        apply(&mut textures, &full.textures_delta);
        full.textures_delta.clear();
        output = Some(full);
    }
    let output = output.expect("two frames ran");
    let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
    let (w, h) = (
        (WIDTH * PIXELS_PER_POINT) as usize,
        (height * PIXELS_PER_POINT) as usize,
    );
    let mut canvas = vec![[0.0f32; 4]; w * h];
    for ClippedPrimitive {
        clip_rect,
        primitive,
    } in &primitives
    {
        let Primitive::Mesh(mesh) = primitive else {
            continue;
        };
        let Some(texture) = textures.get(&mesh.texture_id) else {
            continue;
        };
        let clip = Rect::from_min_max(
            (clip_rect.min.to_vec2() * PIXELS_PER_POINT).to_pos2(),
            (clip_rect.max.to_vec2() * PIXELS_PER_POINT).to_pos2(),
        );
        for triangle in mesh.indices.as_chunks::<3>().0 {
            let corners = [0, 1, 2].map(|i| mesh.vertices[triangle[i] as usize]);
            fill(&mut canvas, w, h, clip, &corners, texture);
        }
    }
    let mut bytes = Vec::with_capacity(w * h * 4);
    for pixel in &canvas {
        bytes.extend(pixel.map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8));
    }
    let file = std::io::BufWriter::new(std::fs::File::create(&path)?);
    let mut encoder = png::Encoder::new(file, w as u32, h as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut writer| writer.write_image_data(&bytes))
        .map_err(std::io::Error::other)?;
    writeln!(std::io::stdout(), "wrote {path}")
}

/// Applies a frame's texture changes, whole images and patches.
fn apply(textures: &mut HashMap<TextureId, ColorImage>, delta: &egui::TexturesDelta) {
    for (id, change) in delta
        .set
        .iter()
        .flat_map(|(id, changes)| changes.iter().map(move |c| (id, c)))
    {
        let egui::ImageData::Color(image) = &change.image;
        match change.pos {
            None => {
                textures.insert(*id, (**image).clone());
            }
            Some([x, y]) => {
                let Some(target) = textures.get_mut(id) else {
                    continue;
                };
                for row in 0..image.size[1] {
                    for column in 0..image.size[0] {
                        let at = (y + row) * target.size[0] + x + column;
                        if let Some(pixel) = target.pixels.get_mut(at) {
                            *pixel = image.pixels[row * image.size[0] + column];
                        }
                    }
                }
            }
        }
    }
}

/// Bilinear sample of a premultiplied texture, as linear-filtered GPUs do.
fn sample(texture: &ColorImage, uv: Pos2) -> [f32; 4] {
    let [tw, th] = texture.size;
    let x = (uv.x * tw as f32 - 0.5).clamp(0.0, tw as f32 - 1.0);
    let y = (uv.y * th as f32 - 0.5).clamp(0.0, th as f32 - 1.0);
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (x1, y1) = ((x0 + 1).min(tw - 1), (y0 + 1).min(th - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let get = |x: usize, y: usize| {
        let c = texture.pixels[y * tw + x];
        [c.r(), c.g(), c.b(), c.a()].map(|v| f32::from(v) / 255.0)
    };
    let (a, b, c, d) = (get(x0, y0), get(x1, y0), get(x0, y1), get(x1, y1));
    std::array::from_fn(|i| {
        (a[i] * (1.0 - fx) + b[i] * fx) * (1.0 - fy) + (c[i] * (1.0 - fx) + d[i] * fx) * fy
    })
}

/// Fills one triangle, blending premultiplied colour over the canvas.
fn fill(
    canvas: &mut [[f32; 4]],
    w: usize,
    h: usize,
    clip: Rect,
    corners: &[Vertex; 3],
    texture: &ColorImage,
) {
    let p = corners.map(|v| (v.pos.to_vec2() * PIXELS_PER_POINT).to_pos2());
    let area = (p[1] - p[0]).x * (p[2] - p[0]).y - (p[1] - p[0]).y * (p[2] - p[0]).x;
    if area.abs() < 1e-6 {
        return;
    }
    let min_x = p
        .iter()
        .map(|q| q.x)
        .fold(f32::MAX, f32::min)
        .max(clip.min.x)
        .max(0.0);
    let max_x = p
        .iter()
        .map(|q| q.x)
        .fold(f32::MIN, f32::max)
        .min(clip.max.x)
        .min(w as f32);
    let min_y = p
        .iter()
        .map(|q| q.y)
        .fold(f32::MAX, f32::min)
        .max(clip.min.y)
        .max(0.0);
    let max_y = p
        .iter()
        .map(|q| q.y)
        .fold(f32::MIN, f32::max)
        .min(clip.max.y)
        .min(h as f32);
    for y in min_y.floor() as usize..max_y.ceil() as usize {
        for x in min_x.floor() as usize..max_x.ceil() as usize {
            let q = Pos2::new(x as f32 + 0.5, y as f32 + 0.5);
            let edge = |a: Pos2, b: Pos2| (b - a).x * (q - a).y - (b - a).y * (q - a).x;
            let weights = [edge(p[1], p[2]), edge(p[2], p[0]), edge(p[0], p[1])].map(|e| e / area);
            if weights.iter().any(|&weight| weight < -1e-4) {
                continue;
            }
            let uv = corners
                .iter()
                .zip(weights)
                .fold(Pos2::ZERO, |sum, (v, weight)| sum + v.uv.to_vec2() * weight);
            let colour: [f32; 4] = std::array::from_fn(|i| {
                corners
                    .iter()
                    .zip(weights)
                    .map(|(v, weight)| {
                        let c = v.color;
                        f32::from([c.r(), c.g(), c.b(), c.a()][i]) / 255.0 * weight
                    })
                    .sum()
            });
            let texel = sample(texture, uv);
            let source: [f32; 4] = std::array::from_fn(|i| colour[i] * texel[i]);
            let target = &mut canvas[y * w + x];
            for i in 0..4 {
                target[i] = source[i] + target[i] * (1.0 - source[3]);
            }
        }
    }
}
