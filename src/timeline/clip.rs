use anyhow::{bail, Result};

use crate::scene::{AnimatedTransform, Object};

#[derive(Debug, Clone, PartialEq)]
pub struct Clip {
    pub start: f32,
    pub end: f32,
    pub object: Object,
    pub transform: AnimatedTransform,
}

impl Clip {
    pub fn new(
        start: f32,
        end: f32,
        object: Object,
        transform: AnimatedTransform,
        duration: f32,
    ) -> Result<Self> {
        if duration <= 0.0 {
            bail!("duration must be > 0");
        }
        if start < 0.0 || end <= start || end > duration {
            bail!("clip bounds must satisfy 0 <= start < end <= duration");
        }
        Ok(Self {
            start,
            end,
            object,
            transform,
        })
    }

    pub fn is_active(&self, t: f32) -> bool {
        t >= self.start && t < self.end
    }

    pub fn local_time(&self, t: f32) -> Option<f32> {
        if self.is_active(t) {
            Some(t - self.start)
        } else {
            None
        }
    }

    pub fn clamped_local_time(&self, t: f32) -> f32 {
        if t <= self.start {
            0.0
        } else if t >= self.end {
            self.end - self.start
        } else {
            t - self.start
        }
    }
}
