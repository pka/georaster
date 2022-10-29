//! GeoTIFF / COG file reader.

use std::io::{Read, Seek};
use tiff::decoder::{Decoder, DecodingResult};
use tiff::tags::Tag;
use tiff::{TiffError, TiffResult};

/// GeoTIFF file reader
pub struct GeoTiffReader<R: Read + Seek> {
    decoder: Decoder<R>,
    pub geo_keys: Option<Vec<u32>>,
    pub geo_params: Option<String>,
    pixel_scale: Option<Vec<f64>>,
    model_transformation: Option<Vec<f64>>,
    tie_points: Option<Vec<f64>>,
}

#[derive(PartialEq, Debug)]
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
}

impl<R: Read + Seek + Send> GeoTiffReader<R> {
    /// Open GeoTIFF and read header information
    pub fn open(src: R) -> TiffResult<Self> {
        let mut decoder = Decoder::new(src)?;

        // Read GeoTIFF tags
        let geo_keys = decoder.get_tag_u32_vec(Tag::GeoKeyDirectoryTag).ok();
        let geo_params = decoder.get_tag_ascii_string(Tag::GeoAsciiParamsTag).ok();
        let pixel_scale = decoder.get_tag_f64_vec(Tag::ModelPixelScaleTag).ok();
        let model_transformation = decoder.get_tag_f64_vec(Tag::ModelTransformationTag).ok();
        let tie_points = decoder.get_tag_f64_vec(Tag::ModelTiepointTag).ok();
        // GeoDoubleParamsTag,
        // GdalNodata,

        let reader = GeoTiffReader {
            decoder,
            geo_keys,
            geo_params,
            pixel_scale,
            model_transformation,
            tie_points,
        };

        Ok(reader)
    }

    /// Image dimensions
    pub fn dimensions(&mut self) -> (u32, u32) {
        self.decoder.dimensions().unwrap()
    }

    pub fn colortype(&mut self) -> Option<tiff::ColorType> {
        self.decoder.colortype().ok()
    }

    pub fn origin(&self) -> Option<[f64; 2]> {
        match &self.tie_points {
            Some(tie_points) if tie_points.len() == 6 => Some([tie_points[3], tie_points[4]]),
            _ => self.model_transformation.as_ref().map(|t| [t[3], t[7]]),
        }
    }

    pub fn pixel_size(&self) -> Option<[f64; 2]> {
        match &self.pixel_scale {
            Some(mps) => Some([mps[0], -mps[1]]),
            None => self.model_transformation.as_ref().map(|t| [t[0], t[5]]),
        }
    }

    pub fn read_pixel(&mut self, x: u32, y: u32) -> RasterValue {
        let image_dims = self.dimensions();
        if x >= image_dims.0 || y >= image_dims.1 {
            return RasterValue::NoData;
        }
        let chunk_dims = self.decoder.chunk_dimensions();
        let image = TileAttributes::from_dims(image_dims, chunk_dims);
        let chunk_index = image.get_chunk_index(x, y);
        let offset = image.get_chunk_offset(x, y);
        let chunk = self.decoder.read_chunk(chunk_index).unwrap();
        raster_value(&chunk, offset)
    }

    /// Returns an Iterator over the pixels of an image part.
    /// The iterator yields the coordinates of each pixel
    /// along with their value
    pub fn pixels(&mut self, x: u32, y: u32, width: u32, height: u32) -> Pixels<R> {
        let image_dims = self.dimensions();
        let chunk_dims = self.decoder.chunk_dimensions();
        let dims = TileAttributes::from_dims(image_dims, chunk_dims);
        Pixels {
            decoder: &mut self.decoder,
            chunk: Err(TiffError::LimitsExceeded),
            offset: 0,
            x,
            y,
            col: 0,
            row: 0,
            dims,
            min_x: x,
            min_y: y,
            max_x: x + width,
            max_y: y + height,
        }
    }

    pub fn read_cog(&mut self) {
        // Good format description:
        // https://medium.com/planet-stories/reading-a-single-tiff-pixel-without-any-tiff-tools-fcbd43d8bd24

        let tiles = self.decoder.tile_count().unwrap();
        dbg!(tiles);
        dbg!(self.decoder.chunk_dimensions());

        for tile in 0..tiles {
            // tiles in row major order
            dbg!(self.decoder.chunk_data_dimensions(tile));
            match self.decoder.read_chunk(tile).unwrap() {
                DecodingResult::U8(res) => {
                    let _sum: u64 = res.into_iter().map(<u64>::from).sum();
                }
                _ => panic!("Wrong bit depth"),
            }
        }
    }

    pub fn read_overviews(&mut self) {
        while self.decoder.more_images() {
            self.decoder.next_image().unwrap();
            dbg!(self.decoder.dimensions().unwrap());
            if let Ok(subfile_type) = self.decoder.get_tag_u64(Tag::NewSubfileType) {
                dbg!(subfile_type);
            }
        }
    }
}

/// Raster iterator
pub struct Pixels<'a, R: Read + Seek> {
    decoder: &'a mut Decoder<R>,
    chunk: TiffResult<DecodingResult>,
    offset: usize,
    x: u32,
    y: u32,
    col: u32,
    row: u32,
    dims: TileAttributes,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
}

impl<'a, R: Read + Seek> Iterator for Pixels<'a, R> {
    type Item = (u32, u32, RasterValue);

    fn next(&mut self) -> Option<(u32, u32, RasterValue)> {
        if self.chunk.is_err() {
            if self.x >= self.dims.image_width as u32 || self.y >= self.dims.image_height as u32 {
                return None;
            }
            let chunk_index = self.dims.get_chunk_index(self.x, self.y);
            self.row = chunk_index / self.dims.tiles_across() as u32;
            self.col = chunk_index % self.dims.tiles_across() as u32;
            self.chunk = self.decoder.read_chunk(chunk_index);
            self.offset = self.dims.get_chunk_offset(self.x, self.y);
        } else {
            let (w, h) = (self.dims.tile_width as u32, self.dims.tile_length as u32);
            if self.x % w < w - 1 && self.x + 1 < self.max_x {
                self.x += 1;
                self.offset += 1;
            } else if self.y % h + 1 < h && self.y + 1 < self.max_y {
                self.y += 1;
                self.x = (self.col * w).max(self.min_x);
                self.offset = self.dims.get_chunk_offset(self.x, self.y);
            } else {
                // next chunk
                if self.x + 1 < self.max_x {
                    self.x += 1;
                    self.y = (self.row * h).max(self.min_y);
                } else if self.y + 1 < self.max_y {
                    self.y += 1;
                    self.x = self.min_x;
                } else {
                    return None;
                }
                let chunk_index = self.dims.get_chunk_index(self.x, self.y);
                self.row = chunk_index / self.dims.tiles_across() as u32;
                self.col = chunk_index % self.dims.tiles_across() as u32;
                self.chunk = self.decoder.read_chunk(chunk_index);
                self.offset = self.dims.get_chunk_offset(self.x, self.y);
            }
        }
        let val = raster_value(self.chunk.as_ref().unwrap(), self.offset);
        Some((self.x, self.y, val))
    }
}

fn raster_value(chunk: &DecodingResult, offset: usize) -> RasterValue {
    match chunk {
        DecodingResult::U8(chunk) => chunk
            .get(offset)
            .map(|v| RasterValue::U8(*v))
            .unwrap_or(RasterValue::NoData),
        DecodingResult::U16(chunk) => chunk
            .get(offset)
            .map(|v| RasterValue::U16(*v))
            .unwrap_or(RasterValue::NoData),
        DecodingResult::U32(chunk) => chunk
            .get(offset)
            .map(|v| RasterValue::U32(*v))
            .unwrap_or(RasterValue::NoData),
        DecodingResult::U64(chunk) => chunk
            .get(offset)
            .map(|v| RasterValue::U64(*v))
            .unwrap_or(RasterValue::NoData),
        DecodingResult::F32(chunk) => chunk
            .get(offset)
            .map(|v| RasterValue::F32(*v))
            .unwrap_or(RasterValue::NoData),
        DecodingResult::F64(chunk) => chunk
            .get(offset)
            .map(|v| RasterValue::F64(*v))
            .unwrap_or(RasterValue::NoData),
        DecodingResult::I8(chunk) => chunk
            .get(offset)
            .map(|v| RasterValue::I8(*v))
            .unwrap_or(RasterValue::NoData),
        DecodingResult::I16(chunk) => chunk
            .get(offset)
            .map(|v| RasterValue::I16(*v))
            .unwrap_or(RasterValue::NoData),
        DecodingResult::I32(chunk) => chunk
            .get(offset)
            .map(|v| RasterValue::I32(*v))
            .unwrap_or(RasterValue::NoData),
        DecodingResult::I64(chunk) => chunk
            .get(offset)
            .map(|v| RasterValue::I64(*v))
            .unwrap_or(RasterValue::NoData),
    }
}

// Tile calculation helper from image-tiff
/// Computed values useful for tile decoding
pub(crate) struct TileAttributes {
    pub image_width: usize,
    pub image_height: usize,

    pub tile_width: usize,
    pub tile_length: usize,
}

impl TileAttributes {
    pub fn from_dims(image_dims: (u32, u32), chunk_dims: (u32, u32)) -> Self {
        TileAttributes {
            image_width: image_dims.0 as usize,
            image_height: image_dims.1 as usize,
            tile_width: chunk_dims.0 as usize,
            tile_length: chunk_dims.1 as usize,
        }
    }
    pub fn tiles_across(&self) -> usize {
        (self.image_width + self.tile_width - 1) / self.tile_width
    }
    // pub fn tiles_down(&self) -> usize {
    //     (self.image_height + self.tile_length - 1) / self.tile_length
    // }
    // fn padding_right(&self) -> usize {
    //     self.tile_width - self.image_width % self.tile_width
    // }
    // fn padding_down(&self) -> usize {
    //     self.tile_length - self.image_height % self.tile_length
    // }

    // pub fn get_padding(&self, tile: usize) -> (usize, usize) {
    //     let row = tile / self.tiles_across();
    //     let column = tile % self.tiles_across();

    //     let padding_right = if column == self.tiles_across() - 1 {
    //         self.padding_right()
    //     } else {
    //         0
    //     };

    //     let padding_down = if row == self.tiles_down() - 1 {
    //         self.padding_down()
    //     } else {
    //         0
    //     };

    //     (padding_right, padding_down)
    // }
    /// Return tile or stripe index of a pixel
    fn get_chunk_index(&self, x: u32, y: u32) -> u32 {
        assert!(x < self.image_width as u32);
        assert!(y < self.image_height as u32);
        let x_chunks = x as usize / self.tile_width;
        let y_chunks = y as usize / self.tile_length;
        let chunk_index = y_chunks * self.tiles_across() + x_chunks;
        chunk_index as u32
    }

    /// Return offset of a pixel in tile or stripe
    fn get_chunk_offset(&self, x: u32, y: u32) -> usize {
        assert!(x < self.image_width as u32);
        assert!(y < self.image_height as u32);
        let x_offset = x as usize % self.tile_width;
        let y_offset = y as usize % self.tile_length;
        let offset = y_offset * self.tile_width + x_offset;
        offset
    }
}
