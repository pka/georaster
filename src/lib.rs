//! Library for accessing geospatial raster images.

pub mod geo;
pub mod geotiff;
pub mod pmtiles;

pub use geo::Coordinate;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeorasterError {
    #[error("Io error - {0}")]
    Io(#[from] std::io::Error),
    #[error("Tiff error - {0}")]
    Tiff(#[from] tiff::TiffError),
    #[error("Image error - {0}")]
    Image(#[from] image::ImageError),
    #[error("PMTiles error - {0}")]
    Pmt(#[from] pmt::PmtError),
}

pub type GeorasterResult<T> = Result<T, GeorasterError>;
