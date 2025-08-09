use crate::{GeorasterResult, RasterValue};
use image::{DynamicImage, GenericImageView, ImageReader, Pixel};
use pmt::{AsyncPmTilesReader, PmtError, TileCoord};
use std::io::Cursor;

/// PMTiles raster reader
pub struct PmtilesRasterReader {
    reader: AsyncPmTilesReader<pmt::MmapBackend>,
}

impl PmtilesRasterReader {
    pub async fn open(name: &str) -> GeorasterResult<Self> {
        // Use `new_with_cached_path` for better performance
        let reader = AsyncPmTilesReader::new_with_path(name).await?;
        Ok(Self { reader })
    }

    pub async fn get_tile(&self, z: u8, x: u32, y: u32) -> GeorasterResult<DynamicImage> {
        let coord = TileCoord::new(z, x, y).ok_or(PmtError::InvalidEntry)?;
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

    pub async fn get_pixel(
        &self,
        z: u8,
        x: u32,
        y: u32,
        px: u32,
        py: u32,
    ) -> GeorasterResult<RasterValue> {
        let tile = self.get_tile(z, x, y).await?;
        let pixel = tile.get_pixel(px, py);
        Ok(pixel.into())
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
        assert_eq!(
            pmtiles.get_pixel(12, 2128, 1438, 10, 10).await.unwrap(),
            RasterValue::Rgba8(131, 4, 183, 255)
        );
        assert_eq!(
            pmtiles
                .get_pixel(12, 2128, 1438, 10, 10)
                .await
                .unwrap()
                .height(),
            772.7176470588238
        );
        //assert_eq!(get_tile(10, 532, 359).await.unwrap().len(), 316992);
    }

    #[tokio::test]
    async fn invalid_tiles() {
        let pmtiles = test_tiles().await;
        assert_eq!(
            pmtiles.get_tile(12, 0, 0).await.err().unwrap().to_string(),
            "PMTiles error - Invalid PMTiles entry"
        );
    }

    #[tokio::test]
    async fn test_pixel() {
        let pmtiles = test_tiles().await;
        // Chasseral 47.133037, 7.059309 1607m
        assert_eq!(pmtiles.get_tile(12, 2128, 1438).await.unwrap().width(), 512);
    }
}
