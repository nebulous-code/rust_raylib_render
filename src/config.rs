use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub duration_secs: u32,
    pub output_path: PathBuf,
}

impl Config {
    pub fn total_frames(&self) -> u32 {
        self.fps.saturating_mul(self.duration_secs)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            fps: 30,
            duration_secs: 60,
            output_path: PathBuf::from("output.mp4"),
        }
    }
}
