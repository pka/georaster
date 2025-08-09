use bytes::Bytes;
use pmtiles::{AsyncPmTilesReader, PmtError, PmtResult, TileCoord};

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

    pub async fn get_tile(&self, z: u8, x: u32, y: u32) -> PmtResult<Option<Bytes>> {
        let coord = TileCoord::new(z, x, y).ok_or(PmtError::InvalidEntry)?;
        self.reader.get_tile(coord).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tile() {
        let pmtiles = PmtilesRasterReader::open("data/ch-nw.pmtiles")
            .await
            .unwrap();
        // Chasseral 47.133037, 7.059309 1607m
        assert_eq!(
            pmtiles
                .get_tile(12, 2128, 1438)
                .await
                .unwrap()
                .unwrap()
                .len(),
            316992
        );
        //assert_eq!(get_tile(10, 532, 359).await.unwrap().len(), 316992);
    }
}
