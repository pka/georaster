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

#[cfg(feature = "geo")]
mod geo {
    use super::Coordinate;
    use geo::{coord, Coord};

    impl From<Coordinate> for Coord {
        fn from(val: Coordinate) -> Self {
            coord! { x: val.x, y: val.y }
        }
    }

    impl From<Coord> for Coordinate {
        fn from(coord: Coord) -> Coordinate {
            Coordinate {
                x: coord.x,
                y: coord.y,
            }
        }
    }

    #[test]
    fn geo_conversion() {
        use geo::{coord, Coord};

        let coord = coord! { x: 1.2345, y: 6.7890 };
        let coordinate: Coordinate = coord.clone().into();

        assert_eq!(coord.x, coordinate.x);
        assert_eq!(coord.y, coordinate.y);

        let coordinate = Coordinate {
            x: 12.345,
            y: 67.890,
        };
        let coord: Coord = coordinate.clone().into();

        assert_eq!(coord.x, coordinate.x);
        assert_eq!(coord.y, coordinate.y);
    }
}

#[cfg(feature = "geodesy")]
mod geodesy {
    use super::Coordinate;
    use geodesy::{Coor2D, CoordinateTuple};

    impl From<Coordinate> for Coor2D {
        fn from(val: Coordinate) -> Self {
            Coor2D::raw(val.x, val.y)
        }
    }

    impl From<Coor2D> for Coordinate {
        fn from(coor2d: Coor2D) -> Coordinate {
            Coordinate {
                x: coor2d.x(),
                y: coor2d.y(),
            }
        }
    }

    #[test]
    fn geodesy_conversion() {
        use geodesy::{Coor2D, CoordinateTuple};

        let coor2d = Coor2D::geo(1.2345, 6.7890);
        let coordinate: Coordinate = coor2d.clone().into();

        assert_eq!(coor2d.x(), coordinate.x);
        assert_eq!(coor2d.y(), coordinate.y);

        let coordinate = Coordinate {
            x: 12.345,
            y: 67.890,
        };
        let coor2d: Coor2D = coordinate.clone().into();

        assert_eq!(coor2d.x(), coordinate.x);
        assert_eq!(coor2d.y(), coordinate.y);
    }
}
