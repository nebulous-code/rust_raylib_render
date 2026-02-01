use std::path::PathBuf;

use chrono::Utc;

#[derive(Debug, Clone)]
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub duration_secs: u32,
    pub output_dir: PathBuf,
    pub output_name: String,
}

impl Config {
    pub fn total_frames(&self) -> u32 {
        self.fps.saturating_mul(self.duration_secs)
    }

    pub fn output_path(&self) -> PathBuf {
        self.output_dir.join(&self.output_name)
    }
}

impl Default for Config {
    fn default() -> Self {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        Self {
            width: 800,
            height: 600,
            fps: 30,
            duration_secs: 60,
            output_dir: PathBuf::from("output"),
            output_name: format!("render_{timestamp}.mp4"),
        }
    }
}
