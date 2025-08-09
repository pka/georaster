//! Library for accessing geospatial raster images.

pub mod geo;
pub mod geotiff;
#[cfg(feature = "pmtiles")]
pub mod pmtiles;

pub use geo::Coordinate;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeorasterError {
    #[error("Raster value type error")]
    ValueRange,
    #[error("Io error - {0}")]
    Io(#[from] std::io::Error),
    #[error("Tiff error - {0}")]
    Tiff(#[from] tiff::TiffError),
    #[cfg(feature = "pmtiles")]
    #[error("Image error - {0}")]
    Image(#[from] image::ImageError),
    #[cfg(feature = "pmtiles")]
    #[error("PMTiles error - {0}")]
    Pmt(#[from] pmt::PmtError),
    #[cfg(feature = "pmtiles")]
    #[error("TMS error - {0}")]
    Tms(#[from] tile_grid::TmsError),
}

pub type GeorasterResult<T> = Result<T, GeorasterError>;

#[derive(PartialEq, Debug)]
#[non_exhaustive]
pub enum RasterValue {
    NoData,
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Rgb8(u8, u8, u8),
    Rgba8(u8, u8, u8, u8),
    Rgb16(u16, u16, u16),
    Rgba16(u16, u16, u16, u16),
}

impl fmt::Display for RasterValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RasterValue::U8(v) => write!(f, "{v}"),
            RasterValue::U16(v) => write!(f, "{v}"),
            RasterValue::U32(v) => write!(f, "{v}"),
            RasterValue::U64(v) => write!(f, "{v}"),
            RasterValue::F32(v) => write!(f, "{v}"),
            RasterValue::F64(v) => write!(f, "{v}"),
            RasterValue::I8(v) => write!(f, "{v}"),
            RasterValue::I16(v) => write!(f, "{v}"),
            RasterValue::I32(v) => write!(f, "{v}"),
            RasterValue::I64(v) => write!(f, "{v}"),
            RasterValue::Rgb8(r, g, b) => write!(f, "({r},{g},{b})"),
            RasterValue::Rgb16(r, g, b) => write!(f, "({r},{g},{b})"),
            RasterValue::Rgba8(r, g, b, a) => write!(f, "({r},{g},{b},{a})"),
            RasterValue::Rgba16(r, g, b, a) => write!(f, "({r},{g},{b},{a})"),
            _ => write!(f, "<NoData>"),
        }
    }
}

impl TryFrom<RasterValue> for u64 {
    type Error = GeorasterError;

    fn try_from(value: RasterValue) -> Result<Self, Self::Error> {
        match value {
            RasterValue::U8(v) => Ok(v as u64),
            RasterValue::U16(v) => Ok(v as u64),
            RasterValue::U32(v) => Ok(v as u64),
            RasterValue::U64(v) => Ok(v),
            _ => Err(GeorasterError::ValueRange),
        }
    }
}

impl TryFrom<RasterValue> for i64 {
    type Error = GeorasterError;

    fn try_from(value: RasterValue) -> Result<Self, Self::Error> {
        match value {
            RasterValue::U8(v) => Ok(v as i64),
            RasterValue::U16(v) => Ok(v as i64),
            RasterValue::U32(v) => Ok(v as i64),
            RasterValue::I8(v) => Ok(v as i64),
            RasterValue::I16(v) => Ok(v as i64),
            RasterValue::I32(v) => Ok(v as i64),
            RasterValue::I64(v) => Ok(v),
            _ => Err(GeorasterError::ValueRange),
        }
    }
}

impl TryFrom<RasterValue> for f64 {
    type Error = GeorasterError;

    fn try_from(value: RasterValue) -> Result<Self, Self::Error> {
        match value {
            RasterValue::U8(v) => Ok(v as f64),
            RasterValue::U16(v) => Ok(v as f64),
            RasterValue::U32(v) => Ok(v as f64),
            RasterValue::U64(v) => Ok(v as f64),
            RasterValue::F32(v) => Ok(v as f64),
            RasterValue::F64(v) => Ok(v),
            RasterValue::I8(v) => Ok(v as f64),
            RasterValue::I16(v) => Ok(v as f64),
            RasterValue::I32(v) => Ok(v as f64),
            RasterValue::I64(v) => Ok(v as f64),
            _ => Err(GeorasterError::ValueRange),
        }
    }
}

fn decode_terrarium_rgb(r: u16, g: u16, b: u16) -> f64 {
    ((r as f64) * 256. + (g as f64) + (b as f64) / 255.0) - 32768.
}

fn decode_mapbox_rgb(r: u16, g: u16, b: u16) -> f64 {
    ((r as f64) * 256. * 256. + (g as f64) * 256. + (b as f64)) / 10.0 - 10000.0
}

impl RasterValue {
    /// Decode Terrarium encoded RGB values to height in meters
    pub fn height(&self) -> f64 {
        match self {
            RasterValue::Rgb8(r, g, b) => decode_terrarium_rgb(*r as u16, *g as u16, *b as u16),
            RasterValue::Rgba8(r, g, b, _a) => {
                decode_terrarium_rgb(*r as u16, *g as u16, *b as u16)
            }
            RasterValue::Rgb16(r, g, b) => decode_terrarium_rgb(*r, *g, *b),
            RasterValue::Rgba16(r, g, b, _a) => decode_terrarium_rgb(*r, *g, *b),
            RasterValue::U8(v) => *v as f64,
            RasterValue::U16(v) => *v as f64,
            RasterValue::U32(v) => *v as f64,
            RasterValue::U64(v) => *v as f64,
            RasterValue::F32(v) => *v as f64,
            RasterValue::F64(v) => *v,
            RasterValue::I8(v) => *v as f64,
            RasterValue::I16(v) => *v as f64,
            RasterValue::I32(v) => *v as f64,
            RasterValue::I64(v) => *v as f64,
            RasterValue::NoData => f64::NAN,
        }
    }

    /// Decode Mapbox encoded RGB values to height in meters
    pub fn height_mb(&self) -> f64 {
        match self {
            RasterValue::Rgb8(r, g, b) => decode_mapbox_rgb(*r as u16, *g as u16, *b as u16),
            RasterValue::Rgba8(r, g, b, _a) => decode_mapbox_rgb(*r as u16, *g as u16, *b as u16),
            RasterValue::Rgb16(r, g, b) => decode_mapbox_rgb(*r, *g, *b),
            RasterValue::Rgba16(r, g, b, _a) => decode_mapbox_rgb(*r, *g, *b),
            RasterValue::U8(v) => *v as f64,
            RasterValue::U16(v) => *v as f64,
            RasterValue::U32(v) => *v as f64,
            RasterValue::U64(v) => *v as f64,
            RasterValue::F32(v) => *v as f64,
            RasterValue::F64(v) => *v,
            RasterValue::I8(v) => *v as f64,
            RasterValue::I16(v) => *v as f64,
            RasterValue::I32(v) => *v as f64,
            RasterValue::I64(v) => *v as f64,
            RasterValue::NoData => f64::NAN,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_conversion() {
        assert_eq!(
            i64::try_from(RasterValue::U32(u32::MAX)).unwrap(),
            4294967295
        );
        assert_eq!(f64::try_from(RasterValue::I64(-1)).unwrap(), -1.0);
        assert_eq!(
            u64::try_from(RasterValue::U32(u32::MAX)).unwrap(),
            4294967295
        );
        assert_eq!(u64::try_from(RasterValue::NoData).ok(), None);
    }

    #[test]
    fn height_conversion() {
        assert_eq!(RasterValue::U32(1243).height(), 1243.);
        assert_eq!(RasterValue::I64(-1).height(), -1.0);
        assert_eq!(RasterValue::Rgb8(134, 65, 215).height(), 1601.843137254902);
        assert!(RasterValue::NoData.height().is_nan());
    }
}
