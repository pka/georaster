//! GeoTIFF / COG file reader.

use std::io::{Read, Seek};
use tiff::decoder::{ifd, Decoder, DecodingResult};
use tiff::tags::{PhotometricInterpretation, Tag};
use tiff::{TiffError, TiffResult};

/// GeoTIFF file reader
pub struct GeoTiffReader<R: Read + Seek> {
    decoder: Decoder<R>,
    pub geo_keys: Option<Vec<u32>>,
    pub geo_params: Option<String>,
    pixel_scale: Option<Vec<f64>>,
    model_transformation: Option<Vec<f64>>,
    tie_points: Option<Vec<f64>>,
    pub photometric_interpretation: Option<PhotometricInterpretation>,
}

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
        let photometric_interpretation = match decoder.get_tag(Tag::PhotometricInterpretation) {
            Ok(ifd::Value::Short(v)) => PhotometricInterpretation::from_u16(v),
            Ok(ifd::Value::Unsigned(v)) => PhotometricInterpretation::from_u16(v as u16),
            _ => None,
        };

        let reader = GeoTiffReader {
            decoder,
            geo_keys,
            geo_params,
            pixel_scale,
            model_transformation,
            tie_points,
            photometric_interpretation,
        };

        Ok(reader)
    }

    /// Image dimensions.
    pub fn dimensions(&mut self) -> Option<(u32, u32)> {
        self.decoder.dimensions().ok()
    }

    /// Image dimensions or (0, 0) if undefined.
    pub fn dimensions_or_zero(&mut self) -> (u32, u32) {
        self.dimensions().unwrap_or((0, 0))
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

    /// Returns the default chunk size for the current image.
    pub fn chunk_dimensions(&self) -> (u32, u32) {
        self.decoder.chunk_dimensions()
    }

    /// Samples per pixel
    fn spp(&mut self) -> usize {
        match self.decoder.colortype() {
            Ok(tiff::ColorType::Gray(_)) => 1,
            Ok(tiff::ColorType::RGB(_)) => 3,
            Ok(tiff::ColorType::RGBA(_)) => 4,
            _ => 1, // unsupported
        }
    }

    /// Return raster value at offset x/y
    pub fn read_pixel(&mut self, x: u32, y: u32) -> RasterValue {
        let image_dims = self.dimensions_or_zero();
        if x >= image_dims.0 || y >= image_dims.1 {
            return RasterValue::NoData;
        }
        let chunk_dims = self.decoder.chunk_dimensions();
        let image = TileAttributes::from_dims(image_dims, chunk_dims);
        let chunk_index = image.get_chunk_index(x, y);
        let spp = self.spp();
        let offset = image.get_chunk_offset(x, y, spp);
        let chunk = self.decoder.read_chunk(chunk_index).unwrap();
        raster_value(&chunk, offset, spp)
    }

    /// Returns an Iterator over the pixels of an image part.
    /// The iterator yields the coordinates of each pixel
    /// along with their value
    pub fn pixels(&mut self, x: u32, y: u32, width: u32, height: u32) -> Pixels<R> {
        let image_dims = self.dimensions_or_zero();
        let chunk_dims = self.decoder.chunk_dimensions();
        let dims = TileAttributes::from_dims(image_dims, chunk_dims);
        let spp = self.spp();
        Pixels {
            decoder: &mut self.decoder,
            chunk: Err(TiffError::LimitsExceeded),
            offset: 0,
            x,
            y,
            col: 0,
            row: 0,
            dims,
            spp,
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
    // Samples per pixel (Gray=1, RGB (single band) = 3, etc.)
    spp: usize,
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
            self.read_chunk();
        } else {
            // Iterate within chunks (tiles/stripes) from left to right, top to bottom
            let (w, h) = (self.dims.tile_width as u32, self.dims.tile_length as u32);
            if self.x % w < w - 1 && self.x + 1 < self.max_x {
                self.x += 1;
                self.offset += self.spp;
            } else if self.y % h + 1 < h && self.y + 1 < self.max_y {
                // next row in chunk
                self.y += 1;
                self.x = (self.col * w).max(self.min_x);
                self.offset = self.dims.get_chunk_offset(self.x, self.y, self.spp);
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
                self.read_chunk();
            }
        }
        let val = raster_value(self.chunk.as_ref().unwrap(), self.offset, self.spp);
        Some((self.x, self.y, val))
    }
}

impl<'a, R: Read + Seek> Pixels<'a, R> {
    fn read_chunk(&mut self) {
        let chunk_index = self.dims.get_chunk_index(self.x, self.y);
        self.row = chunk_index / self.dims.tiles_across() as u32;
        self.col = chunk_index % self.dims.tiles_across() as u32;
        self.chunk = self.decoder.read_chunk(chunk_index);
        self.offset = self.dims.get_chunk_offset(self.x, self.y, self.spp);
    }
}

fn raster_value(chunk: &DecodingResult, offset: usize, spp: usize) -> RasterValue {
    match chunk {
        DecodingResult::U8(chunk) => match spp {
            3 => {
                if let (Some(r), Some(g), Some(b)) = (
                    chunk.get(offset),
                    chunk.get(offset + 1),
                    chunk.get(offset + 2),
                ) {
                    Some(RasterValue::Rgb8(*r, *g, *b))
                } else {
                    None
                }
            }
            4 => {
                if let (Some(r), Some(g), Some(b), Some(a)) = (
                    chunk.get(offset),
                    chunk.get(offset + 1),
                    chunk.get(offset + 2),
                    chunk.get(offset + 3),
                ) {
                    Some(RasterValue::Rgba8(*r, *g, *b, *a))
                } else {
                    None
                }
            }
            _ => chunk.get(offset).map(|v| RasterValue::U8(*v)),
        },
        DecodingResult::U16(chunk) => match spp {
            3 => {
                if let (Some(r), Some(g), Some(b)) = (
                    chunk.get(offset),
                    chunk.get(offset + 1),
                    chunk.get(offset + 2),
                ) {
                    Some(RasterValue::Rgb16(*r, *g, *b))
                } else {
                    None
                }
            }
            4 => {
                if let (Some(r), Some(g), Some(b), Some(a)) = (
                    chunk.get(offset),
                    chunk.get(offset + 1),
                    chunk.get(offset + 2),
                    chunk.get(offset + 3),
                ) {
                    Some(RasterValue::Rgba16(*r, *g, *b, *a))
                } else {
                    None
                }
            }
            _ => chunk.get(offset).map(|v| RasterValue::U16(*v)),
        },
        DecodingResult::U32(chunk) => chunk.get(offset).map(|v| RasterValue::U32(*v)),
        DecodingResult::U64(chunk) => chunk.get(offset).map(|v| RasterValue::U64(*v)),
        DecodingResult::F32(chunk) => chunk.get(offset).map(|v| RasterValue::F32(*v)),
        DecodingResult::F64(chunk) => chunk.get(offset).map(|v| RasterValue::F64(*v)),
        DecodingResult::I8(chunk) => chunk.get(offset).map(|v| RasterValue::I8(*v)),
        DecodingResult::I16(chunk) => chunk.get(offset).map(|v| RasterValue::I16(*v)),
        DecodingResult::I32(chunk) => chunk.get(offset).map(|v| RasterValue::I32(*v)),
        DecodingResult::I64(chunk) => chunk.get(offset).map(|v| RasterValue::I64(*v)),
    }
    .unwrap_or(RasterValue::NoData)
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
    fn get_chunk_offset(&self, x: u32, y: u32, spp: usize) -> usize {
        assert!(x < self.image_width as u32);
        assert!(y < self.image_height as u32);
        let w = self.tile_width.min(self.image_width);
        let h = self.tile_length.min(self.image_height);
        let x_offset = x as usize % w;
        let y_offset = y as usize % h;
        let offset = y_offset * w + x_offset;
        offset * spp
    }
}
