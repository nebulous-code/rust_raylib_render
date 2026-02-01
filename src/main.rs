mod config;
mod encoder;
mod renderer;

use anyhow::{bail, Result};

use config::Config;
use encoder::FfmpegEncoder;
use renderer::BouncingBallRenderer;

fn main() -> Result<()> {
    let config = Config::default();
    let total_frames = config.total_frames();
    if total_frames == 0 {
        bail!("fps * duration must be > 0");
    }

    let mut renderer = BouncingBallRenderer::new(&config)?;
    let mut encoder = FfmpegEncoder::start(&config)?;

    for frame_index in 0..total_frames {
        if renderer.window_should_close() {
            break;
        }
        let frame = renderer.render_frame(frame_index, total_frames)?;
        encoder.write_frame(&frame)?;
    }

    encoder.finish()?;
    Ok(())
}
