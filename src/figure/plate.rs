//! Canonical drawing plate. Geometry is fitted into this frame.
//! Letter type is CSS on `.letter-layer` (`cqmin` of the on-screen figure).

/// User-unit stand-in for layout (gaps, collision).
pub(crate) const FONT: f64 = 56.0;
pub(crate) const GAP: f64 = FONT * 0.72;
pub(crate) const MARGIN: f64 = FONT * 1.35;
pub(crate) const STROKE: f64 = 1.4;
pub(crate) const DOT: f64 = FONT * 0.08;
pub(crate) const CHAR_W: f64 = FONT * 0.58;
/// Letter-center radius from the mark, in `em` of the letter type.
/// Inner edge of a ~1em glyph then clears the point (~0.28em).
pub(crate) const LETTER_R: f64 = 0.78;

#[derive(Clone, Copy)]
pub(crate) struct Size {
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct Fit {
    pub ox: f64,
    pub oy: f64,
    pub scale: f64,
    pub size: Size,
}

impl Fit {
    pub fn map(self, p: super::geom::V2) -> super::geom::V2 {
        super::geom::V2::new(self.ox + p.x * self.scale, self.oy + p.y * self.scale)
    }

    pub fn dist(self, d: f64) -> f64 {
        d * self.scale
    }
}
