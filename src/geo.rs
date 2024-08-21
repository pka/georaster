//! Common geo types
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Coordinate {
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
use geo::{coord, Coord};

#[cfg(feature = "geo-crate")]
impl Into<Coord> for Coordinate {
    fn into(self) -> Coord {
        coord! { x: self.x, y: self.y }
    }
}

#[cfg(feature = "geo-crate")]
impl From<Coord> for Coordinate {
    fn from(coord: Coord) -> Coordinate {
        Coordinate {
            x: coord.x,
            y: coord.y,
        }
    }
}

#[cfg(feature = "geodesy-crate")]
use geodesy::{Coor2D, CoordinateTuple};

#[cfg(feature = "geodesy-crate")]
impl Into<Coor2D> for Coordinate {
    fn into(self) -> Coor2D {
        Coor2D::raw(self.x, self.y)
    }
}

#[cfg(feature = "geodesy-crate")]
impl From<Coor2D> for Coordinate {
    fn from(coor2d: Coor2D) -> Coordinate {
        Coordinate {
            x: coor2d.x(),
            y: coor2d.y(),
        }
    }
}
