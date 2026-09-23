/// Plane point, y-up.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct V2 {
    pub x: f64,
    pub y: f64,
}

impl V2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y)
    }

    pub fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y)
    }

    pub fn mul(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s)
    }

    pub fn dot(self, o: Self) -> f64 {
        self.x * o.x + self.y * o.y
    }

    pub fn len(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn dist(self, o: Self) -> f64 {
        self.sub(o).len()
    }

    pub fn unit(self) -> Self {
        let l = self.len();
        if l < 1e-9 {
            Self::new(1.0, 0.0)
        } else {
            self.mul(1.0 / l)
        }
    }

    /// Point on the ray from `self` through `other`, at distance `dist` from `self`.
    pub fn toward(self, other: Self, dist: f64) -> Self {
        self.add(other.sub(self).unit().mul(dist))
    }

    pub fn polar(self, r: f64, deg: f64) -> Self {
        let a = deg.to_radians();
        Self::new(self.x + r * a.cos(), self.y + r * a.sin())
    }
}

/// Preferred side of the mark. The letter center stays at a fixed radius.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Place {
    Auto,
    Left,
    Right,
    Above,
    Below,
    AboveLeft,
    AboveRight,
    BelowLeft,
    BelowRight,
    /// Degrees from +x, y-up (same as `V2::polar`).
    Deg(f64),
}

impl Place {
    pub(crate) fn angle(self) -> Option<f64> {
        use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};
        match self {
            Place::Auto => None,
            Place::Right => Some(0.0),
            Place::AboveRight => Some(FRAC_PI_4),
            Place::Above => Some(FRAC_PI_2),
            Place::AboveLeft => Some(FRAC_PI_2 + FRAC_PI_4),
            Place::Left => Some(PI),
            Place::BelowLeft => Some(-PI + FRAC_PI_4),
            Place::Below => Some(-FRAC_PI_2),
            Place::BelowRight => Some(-FRAC_PI_4),
            Place::Deg(d) => Some(d.to_radians()),
        }
    }
}
