use anyhow::{bail, Result};

use crate::scene::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseInOutQuad,
    EaseOutCubic,
}

impl Easing {
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe<T> {
    pub time: f32,
    pub value: T,
    pub easing_to_next: Easing,
}

impl<T> Keyframe<T> {
    pub fn new(time: f32, value: T, easing_to_next: Easing) -> Self {
        Self {
            time,
            value,
            easing_to_next,
        }
    }
}

pub trait Lerp: Sized + Copy {
    fn lerp(a: Self, b: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        a + (b - a) * t
    }
}

impl Lerp for Vec2 {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        Vec2 {
            x: a.x + (b.x - a.x) * t,
            y: a.y + (b.y - a.y) * t,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Track<T> {
    keyframes: Vec<Keyframe<T>>,
}

impl<T: Lerp> Track<T> {
    pub fn new(keyframes: Vec<Keyframe<T>>) -> Result<Self> {
        if keyframes.is_empty() {
            bail!("track must have at least one keyframe");
        }

        for i in 1..keyframes.len() {
            if keyframes[i].time <= keyframes[i - 1].time {
                bail!("keyframe times must be strictly increasing");
            }
        }

        Ok(Self { keyframes })
    }

    pub fn from_constant(value: T) -> Self {
        Self {
            keyframes: vec![Keyframe::new(0.0, value, Easing::Linear)],
        }
    }

    pub fn sample(&self, t: f32) -> T {
        let first = &self.keyframes[0];
        let last = &self.keyframes[self.keyframes.len() - 1];

        if t <= first.time {
            return first.value;
        }
        if t >= last.time {
            return last.value;
        }

        let mut idx = 0;
        for i in 0..self.keyframes.len() - 1 {
            if t >= self.keyframes[i].time && t < self.keyframes[i + 1].time {
                idx = i;
                break;
            }
        }

        let k0 = &self.keyframes[idx];
        let k1 = &self.keyframes[idx + 1];
        let span = k1.time - k0.time;
        let u = if span > 0.0 { (t - k0.time) / span } else { 0.0 };
        let eased = k0.easing_to_next.apply(u);
        T::lerp(k0.value, k1.value, eased)
    }
}
