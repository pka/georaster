use crate::geotiff::RasterValue;
use image::{DynamicImage, GenericImageView, ImageReader, Pixel};
use pmtiles::{AsyncPmTilesReader, PmtError, PmtResult, TileCoord};
use std::io::Cursor;

/// PMTiles raster reader
pub struct PmtilesRasterReader {
    reader: AsyncPmTilesReader<pmtiles::MmapBackend>,
}

impl PmtilesRasterReader {
    pub async fn open(name: &str) -> PmtResult<Self> {
        // Use `new_with_cached_path` for better performance
        let reader = AsyncPmTilesReader::new_with_path(name).await?;
        Ok(Self { reader })
    }

    pub async fn get_tile(&self, z: u8, x: u32, y: u32) -> PmtResult<DynamicImage> {
        let coord = TileCoord::new(z, x, y).ok_or(PmtError::InvalidEntry)?;
        let bytes = self
            .reader
            .get_tile(coord)
            .await?
            .ok_or(PmtError::InvalidEntry)?;

        let img = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()?
            .decode()
            .unwrap();
        Ok(img)
    }

    pub async fn get_pixel(
        &self,
        z: u8,
        x: u32,
        y: u32,
        px: u32,
        py: u32,
    ) -> PmtResult<RasterValue> {
        let tile = self.get_tile(z, x, y).await?;
        let [r, g, b, a] = tile.get_pixel(px, py).to_rgba().0;
        Ok(RasterValue::Rgba8(r, g, b, a))
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
        assert_eq!(
            pmtiles.get_pixel(12, 2128, 1438, 10, 10).await.unwrap(),
            RasterValue::Rgba8(131, 4, 183, 255)
        );
        //assert_eq!(get_tile(10, 532, 359).await.unwrap().len(), 316992);
    }

    #[tokio::test]
    async fn test_pixel() {
        let pmtiles = test_tiles().await;
        // Chasseral 47.133037, 7.059309 1607m
        assert_eq!(pmtiles.get_tile(12, 2128, 1438).await.unwrap().width(), 512);
    }
}
