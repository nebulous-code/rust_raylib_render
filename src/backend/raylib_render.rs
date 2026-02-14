use std::path::Path;

use anyhow::{bail, Context, Result};
use raylib::consts::{PixelFormat, TraceLogLevel};
use raylib::prelude::*;
use std::time::Instant;

use crate::backend::resources::ResourceCache;
use crate::backend::text_render::draw_text_block;
use crate::scene::{Color, Object, Shape, Transform, Vec2};
use crate::timeline::{SampledScene, Timeline};

pub struct RaylibRender {
    rl: RaylibHandle,
    thread: RaylibThread,
    render_texture: RenderTexture2D,
    width: u32,
    height: u32,
    bg: Color,
    cache: ResourceCache,
}

impl RaylibRender {
    pub fn new(width: u32, height: u32, bg: Color) -> Result<Self> {
        Self::new_with_log_level(width, height, bg, TraceLogLevel::LOG_ERROR)
    }

    pub fn new_with_log_level(
        width: u32,
        height: u32,
        bg: Color,
        log_level: TraceLogLevel,
    ) -> Result<Self> {
        let (mut rl, thread) = raylib::init()
            .size(width as i32, height as i32)
            .log_level(log_level)
            .title("Rust Render (offline)")
            .build();

        let render_texture = rl
            .load_render_texture(&thread, width, height)
            .context("failed to create render texture")?;

        Ok(Self {
            rl,
            thread,
            render_texture,
            width,
            height,
            bg,
            cache: ResourceCache::new(),
        })
    }

    pub fn render_timeline_rgba(
        &mut self,
        timeline: &Timeline,
        start_time: f32,
        end_time: f32,
        mut on_frame: impl FnMut(f32, &[u8]) -> Result<()>,
    ) -> Result<()> {
        self.render_timeline_rgba_with_progress(
            timeline,
            start_time,
            end_time,
            None,
            |t, rgba| on_frame(t, rgba),
        )
    }

    pub fn render_timeline_rgba_with_progress(
        &mut self,
        timeline: &Timeline,
        start_time: f32,
        end_time: f32,
        progress: Option<RenderProgress>,
        mut on_frame: impl FnMut(f32, &[u8]) -> Result<()>,
    ) -> Result<()> {
        if start_time < 0.0 || end_time <= start_time || end_time > timeline.duration {
            bail!("start/end time must satisfy 0 <= start < end <= duration");
        }

        let frames = ((end_time - start_time) * timeline.fps as f32).floor() as u32;
        let progress = progress.unwrap_or_default();
        let mut last_progress_frame = 0u32;
        let mut last_100_frame = 0u32;
        let mut last_100_time = Instant::now();
        let mut per_frame_secs = None;
        let overall_start = Instant::now();

        for i in 0..frames {
            let t = start_time + i as f32 / timeline.fps as f32;
            let scene = timeline.sample(t)?;
            let rgba = self.render_scene_to_rgba(&scene)?;
            on_frame(t, &rgba)?;

            if progress.enabled {
                let frame_idx = i + 1;
                if frame_idx - last_100_frame >= 100 {
                    let elapsed = last_100_time.elapsed().as_secs_f32();
                    let window = frame_idx - last_100_frame;
                    per_frame_secs = Some(elapsed / window as f32);
                    last_100_frame = frame_idx;
                    last_100_time = Instant::now();
                }

                if frame_idx - last_progress_frame >= progress.log_every_frames {
                    last_progress_frame = frame_idx;
                    let percent = frame_idx as f32 / frames.max(1) as f32 * 100.0;
                    let mut line = format!("frames: {frame_idx}/{frames} ({percent:.1}%)");

                    if progress.show_time {
                        let elapsed_secs = overall_start.elapsed().as_secs_f32();
                        let rendered_secs = frame_idx as f32 / timeline.fps as f32;
                        let total_secs = frames as f32 / timeline.fps as f32;
                        line.push_str(&format!(
                            " time {}/{}",
                            format_hms(rendered_secs),
                            format_hms(total_secs)
                        ));
                        if progress.show_eta {
                            if let Some(pf) = per_frame_secs {
                                let remaining_frames = frames.saturating_sub(frame_idx);
                                let eta = remaining_frames as f32 * pf;
                                line.push_str(&format!(" eta {}", format_hms(eta)));
                            } else {
                                line.push_str(&format!(" elapsed {}", format_hms(elapsed_secs)));
                            }
                        }
                    }

                    println!("{line}");
                }
            }
        }

        Ok(())
    }

    pub fn render_scene_to_rgba(&mut self, scene: &SampledScene) -> Result<Vec<u8>> {
        self.cache.preload_for_scene(&mut self.rl, &self.thread, scene)?;

        {
            let mut d = self
                .rl
                .begin_texture_mode(&self.thread, self.render_texture.as_mut());
            d.clear_background(to_raylib_color(self.bg, 1.0));

            for layer in &scene.layers {
                for clip in &layer.clips {
                    draw_object(
                        &mut d,
                        &self.cache,
                        self.width,
                        self.height,
                        &clip.object,
                        &clip.transform,
                    )?;
                }
            }
        }

        capture_rgba(&self.render_texture, self.width, self.height)
    }
}

fn draw_object(
    d: &mut impl RaylibDraw,
    cache: &ResourceCache,
    width: u32,
    height: u32,
    object: &Object,
    transform: &Transform,
) -> Result<()> {
    match object {
        Object::Shape(shape) => draw_shape(d, width, height, shape, transform),
        Object::Image(image) => draw_image(d, cache, width, height, &image.path, transform),
        Object::Text(text) => draw_text_block(d, cache, width, height, text, transform),
    }
}

fn draw_shape(
    d: &mut impl RaylibDraw,
    width: u32,
    height: u32,
    shape: &Shape,
    transform: &Transform,
) -> Result<()> {
    let center = graph_to_screen(transform.pos, width, height);
    let color = to_raylib_color(
        match shape {
            Shape::Circle { color, .. } => *color,
            Shape::Rect { color, .. } => *color,
        },
        transform.opacity,
    );

    match shape {
        Shape::Circle { radius, .. } => {
            let scaled = radius * transform.scale.x.max(0.0);
            d.draw_circle_v(center, scaled, color);
        }
        Shape::Rect { width: w, height: h, .. } => {
            let w = w * transform.scale.x;
            let h = h * transform.scale.y;
            let rec = Rectangle::new(center.x, center.y, w, h);
            let origin = Vector2::new(w / 2.0, h / 2.0);
            d.draw_rectangle_pro(rec, origin, transform.rotation, color);
        }
    }

    Ok(())
}

fn draw_image(
    d: &mut impl RaylibDraw,
    cache: &ResourceCache,
    width: u32,
    height: u32,
    path: &Path,
    transform: &Transform,
) -> Result<()> {
    let texture = cache.get_texture(path)?;
    let tex_w = texture.width as f32;
    let tex_h = texture.height as f32;

    let w = tex_w * transform.scale.x;
    let h = tex_h * transform.scale.y;
    let center = graph_to_screen(transform.pos, width, height);

    let source = Rectangle::new(0.0, 0.0, tex_w, tex_h);
    let dest = Rectangle::new(center.x, center.y, w, h);
    let origin = Vector2::new(w / 2.0, h / 2.0);

    let tint = to_raylib_color(Color::WHITE, transform.opacity);
    d.draw_texture_pro(texture, source, dest, origin, transform.rotation, tint);
    Ok(())
}

fn graph_to_screen(pos: Vec2, width: u32, height: u32) -> Vector2 {
    Vector2::new(width as f32 / 2.0 + pos.x, height as f32 / 2.0 - pos.y)
}

fn to_raylib_color(color: Color, opacity: f32) -> raylib::prelude::Color {
    let alpha = (color.a as f32 * opacity.clamp(0.0, 1.0))
        .round()
        .clamp(0.0, 255.0) as u8;
    raylib::prelude::Color::new(color.r, color.g, color.b, alpha)
}

fn capture_rgba(render_texture: &RenderTexture2D, expected_w: u32, expected_h: u32) -> Result<Vec<u8>> {
    let mut image = unsafe { raylib::ffi::LoadImageFromTexture(*render_texture.texture().as_ref()) };

    let result = (|| {
        if image.data.is_null() {
            bail!("raylib returned null image data");
        }

        if image.format != PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32 {
            unsafe {
                raylib::ffi::ImageFormat(
                    &mut image,
                    PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
                );
            }
        }

        if image.data.is_null() {
            bail!("image data was null after format conversion");
        }

        let width = image.width as u32;
        let height = image.height as u32;
        if width != expected_w || height != expected_h {
            bail!(
                "capture size mismatch: got {}x{}, expected {}x{}",
                width,
                height,
                expected_w,
                expected_h
            );
        }

        let len = (width * height * 4) as usize;
        let bytes = unsafe { std::slice::from_raw_parts(image.data as *const u8, len) };
        Ok(bytes.to_vec())
    })();

    unsafe {
        raylib::ffi::UnloadImage(image);
    }

    result
}

#[derive(Debug, Clone, Copy)]
pub struct RenderProgress {
    pub enabled: bool,
    pub log_every_frames: u32,
    pub show_time: bool,
    pub show_eta: bool,
}

impl Default for RenderProgress {
    fn default() -> Self {
        Self {
            enabled: false,
            log_every_frames: 100,
            show_time: true,
            show_eta: true,
        }
    }
}

fn format_hms(seconds: f32) -> String {
    let total = seconds.max(0.0).round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}
