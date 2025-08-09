use crate::{Coordinate, GeorasterResult, RasterValue};
use image::{DynamicImage, GenericImageView, ImageReader, Pixel};
use pmt::{AsyncPmTilesReader, PmtError, TileCoord};
use std::io::Cursor;
use tile_grid::{tms, BoundingBox, Xyz};

/// PMTiles raster reader
pub struct PmtilesRasterReader {
    reader: AsyncPmTilesReader<pmt::MmapBackend>,
    tms: tile_grid::Tms,
}

impl PmtilesRasterReader {
    pub async fn open(name: &str) -> GeorasterResult<Self> {
        let tms = tms().lookup("WebMercatorQuad").unwrap();
        // Use `new_with_cached_path` for better performance
        let reader = AsyncPmTilesReader::new_with_path(name).await?;
        Ok(Self { reader, tms })
    }

    pub async fn get_tile(&self, xyz: &Xyz) -> GeorasterResult<DynamicImage> {
        let coord =
            TileCoord::new(xyz.z, xyz.x as u32, xyz.y as u32).ok_or(PmtError::InvalidEntry)?;
        let bytes = self
            .reader
            .get_tile(coord)
            .await?
            .ok_or(PmtError::InvalidEntry)?;

        let img = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()?
            .decode()?;
        Ok(img)
    }

    /// Return raster value at geographical location
    pub async fn get_pixel_at(
        &self,
        z: u8,
        coord: impl Into<Coordinate>,
    ) -> GeorasterResult<RasterValue> {
        let coord = coord.into();
        let xyz = self.tms.tile(coord.x, coord.y, z)?;
        let tile = self.get_tile(&xyz).await?;
        let bounds = self.tms.bounds(&xyz)?;
        if let Some((px, py)) = self.coord_to_pixel(&bounds, coord, tile.width(), tile.height()) {
            Ok(tile.get_pixel(px, py).into())
        } else {
            Err(PmtError::InvalidEntry.into())
        }
    }

    fn coord_to_pixel(
        &self,
        bounds: &BoundingBox,
        coord: impl Into<Coordinate>,
        w: u32,
        h: u32,
    ) -> Option<(u32, u32)> {
        let (origin_x, origin_y) = (bounds.left, bounds.top);
        let pixel_size_x = (bounds.right - bounds.left).abs() / w as f64;
        let pixel_size_y = (bounds.top - bounds.bottom).abs() / h as f64;
        let coord = coord.into();
        Some((
            ((coord.x - origin_x) / pixel_size_x).round() as u32,
            // ((coord.y - origin_y) / pixel_size_y).round() as u32,
            ((origin_y - coord.y) / pixel_size_y).round() as u32,
        ))
    }
}

impl<T: Pixel<Subpixel = u8>> From<T> for RasterValue {
    fn from(pixel: T) -> Self {
        match (T::CHANNEL_COUNT, pixel.to_rgba().0) {
            (3, [r, g, b, _a]) => RasterValue::Rgb8(r, g, b),
            (4, [r, g, b, a]) => RasterValue::Rgba8(r, g, b, a),
            _ => unimplemented!(), // TODO: Howto impl for u16, etc.?
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_tiles() -> PmtilesRasterReader {
        PmtilesRasterReader::open("data/ch-nw.pmtiles")
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_tile() {
        let pmtiles = test_tiles().await;
        // Chasseral 47.133037, 7.059309 1607m
        assert_eq!(
            pmtiles
                .get_tile(&Xyz::new(2128, 1438, 12))
                .await
                .unwrap()
                .width(),
            512
        );
        //assert_eq!(get_tile(&Xyz::new(532, 359, 10)).await.unwrap().len(), 316992);
    }

    #[tokio::test]
    async fn invalid_tiles() {
        let pmtiles = test_tiles().await;
        assert_eq!(
            pmtiles
                .get_tile(&Xyz::new(0, 0, 12))
                .await
                .err()
                .unwrap()
                .to_string(),
            "PMTiles error - Invalid PMTiles entry"
        );
    }

    #[tokio::test]
    async fn test_pixel() {
        let pmtiles = test_tiles().await;
        // Chasseral 47.133037, 7.059309 1607m
        assert_eq!(
            pmtiles
                .get_pixel_at(12, (7.059309, 47.133037))
                .await
                .unwrap()
                .height(),
            1598.5294117647063
        );
    }
}
