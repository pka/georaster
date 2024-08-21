#[cfg(feature = "geo-crate")]
use geo::{Coord, coord};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Coordinate
{
    pub x: f64,
    pub y: f64,
}

impl From<(f64, f64)> for Coordinate {
    fn from(coords: (f64, f64)) -> Self {
        Coordinate {
            x: coords.0,
            y: coords.1,
        }
    }
}

impl From<[f64; 2]> for Coordinate {
    fn from(coords: [f64; 2]) -> Self {
        Coordinate {
            x: coords[0],
            y: coords[1],
        }
    }
}


#[cfg(feature = "geo-crate")]
impl Into<Coord> for Coordinate {
    fn into(self) -> Coord {
        coord! { x: self.x, y: self.y }
    }
}

#[cfg(feature = "geo-crate")]
impl From<Coord> for Coordinate {
    fn from(coord: Coord) -> Coordinate {
        Coordinate { x: coord.x, y: coord.y }
    }
}