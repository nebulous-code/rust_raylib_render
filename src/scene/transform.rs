use crate::scene::animation::Track;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    pub const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    // CSS-style alpha (0.0..=1.0) with RGB in 0..=255.
    pub fn rgba_css(r: u8, g: u8, b: u8, a: f32) -> Self {
        let alpha = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
        Self { r, g, b, a: alpha }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub pos: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
    pub opacity: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            scale: Vec2::ONE,
            rotation: 0.0,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimatedTransform {
    pub position: Track<Vec2>,
    pub scale: Track<Vec2>,
    pub rotation: Track<f32>,
    pub opacity: Track<f32>,
}

impl AnimatedTransform {
    pub fn constant(transform: Transform) -> Self {
        Self {
            position: Track::from_constant(transform.pos),
            scale: Track::from_constant(transform.scale),
            rotation: Track::from_constant(transform.rotation),
            opacity: Track::from_constant(transform.opacity),
        }
    }

    pub fn sample(&self, t: f32) -> Transform {
        Transform {
            pos: self.position.sample(t),
            scale: self.scale.sample(t),
            rotation: self.rotation.sample(t),
            opacity: self.opacity.sample(t),
        }
    }
}

impl Default for AnimatedTransform {
    fn default() -> Self {
        Self::constant(Transform::default())
    }
}
