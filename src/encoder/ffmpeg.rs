use std::io::Write;
use std::process::{Child, ChildStdin, Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::config::Config;

pub struct FfmpegEncoder {
    child: Child,
    stdin: Option<ChildStdin>,
    width: u32,
    height: u32,
    fps: u32,
}

impl FfmpegEncoder {
    pub fn start(config: &Config) -> Result<Self> {
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y")
            .arg("-loglevel")
            .arg("error")
            .arg("-f")
            .arg("rawvideo")
            .arg("-pix_fmt")
            .arg("rgba")
            .arg("-s")
            .arg(format!("{}x{}", config.width, config.height))
            .arg("-r")
            .arg(config.fps.to_string())
            .arg("-i")
            .arg("-")
            .arg("-vf")
            .arg("vflip")
            .arg("-c:v")
            .arg("libx264")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg("-crf")
            .arg("18")
            .arg(&config.output_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());

        let mut child = cmd
            .spawn()
            .context("failed to spawn ffmpeg (is it on PATH?)")?;
        let stdin = child
            .stdin
            .take()
            .context("failed to open ffmpeg stdin")?;

        Ok(Self {
            child,
            stdin: Some(stdin),
            width: config.width,
            height: config.height,
            fps: config.fps,
        })
    }

    pub fn write_frame(&mut self, frame: &[u8]) -> Result<()> {
        let expected = (self.width * self.height * 4) as usize;
        if frame.len() != expected {
            bail!(
                "frame size mismatch: got {}, expected {}",
                frame.len(),
                expected
            );
        }

        let stdin = self
            .stdin
            .as_mut()
            .context("ffmpeg stdin already closed")?;
        stdin.write_all(frame).context("failed to write frame")?;
        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        if let Some(mut stdin) = self.stdin.take() {
            let _ = stdin.flush();
        }

        let status = self.child.wait().context("failed to wait for ffmpeg")?;
        if !status.success() {
            bail!("ffmpeg exited with status {}", status);
        }

        Ok(())
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }
}
