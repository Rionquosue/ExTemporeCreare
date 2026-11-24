//! The graphics state and the operators that draw: coordinates, paths, colours, painting,
//! clipping and shadings. Painted paths become document objects as they are drawn ([`Out`]).

mod paint;
mod strips;

#[cfg(test)]
pub(crate) use paint::tilings_compared;

/// Width of a zero-width line (the thinnest a device draws), points.
pub(super) const HAIRLINE: f64 = 0.25;

pub(super) const FAR_AWAY: &str = "objects far outside the page were left out";
pub(super) const TOO_MUCH: &str = "the file draws more than VectorCraft reads: the rest was left out";
