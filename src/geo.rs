//! Common geo types

/// 2D coordinate representation
///
/// Coordinates can be converted from and to `geo` by activating
/// the `geo-crate` feature and `geodesy` by the `geodesy-crate` feature.
///
/// ```rust
/// use georaster::Coordinate;
///
/// let lat = 50.013;
/// let lon = 160.423;
/// let coordinate1 = Coordinate::new(lat, lon);
/// let coordinate2 = Coordinate { x: lon, y: lat };
/// assert_eq!(coordinate1.x, coordinate2.x);
/// assert_eq!(coordinate1.y, coordinate2.y);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Coordinate {
    /// Longitude of the `Coordiate` type
    pub x: f64,
    /// Latitude of the `Coordiate` type
    pub y: f64,
}

impl Coordinate {
    /// Create a new coordinate from x and y
    ///
    /// ```rust
    /// use georaster::Coordinate;
    ///
    /// let lat = 50.013;
    /// let lon = 160.423;
    /// let coordinate = Coordinate::new(lat, lon);
    /// assert_eq!(coordinate.x, lon);
    /// assert_eq!(coordinate.y, lat);
    /// ```
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            x: longitude,
            y: latitude,
        }
    }
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
