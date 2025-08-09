//! GeoTIFF / COG file reader.
// TIFF File Format: https://www.fileformat.info/format/tiff/egff.htm
// GeoTIFF standard: http://docs.opengeospatial.org/is/19-008r4/19-008r4.html
// GDAL TIFF driver: https://gdal.org/drivers/raster/gtiff.html
// GDAL COG driver: https://gdal.org/drivers/raster/cog.html

use crate::GeorasterResult;
use std::fmt;
use std::io::{Read, Seek};
use tiff::decoder::{ifd, Decoder, DecodingResult};
use tiff::tags::{PhotometricInterpretation, PlanarConfiguration, Tag};
use tiff::{TiffError, TiffResult};

use crate::Coordinate;

/// GeoTIFF file reader
pub struct GeoTiffReader<R: Read + Seek> {
    decoder: Decoder<R>,
    band_idx: u8,
    images: Vec<ImageInfo>,
    /// Current image in Decoder
    cur_image_idx: usize,
    pub geo_keys: Option<Vec<u32>>,
    pub geo_params: Option<String>,
    pixel_scale: Option<Vec<f64>>,
    model_transformation: Option<Vec<f64>>,
    tie_points: Option<Vec<f64>>,
}

/// Image information from TIFF IFD
#[derive(Debug)]
pub struct ImageInfo {
    /// Image dimensions.
    pub dimensions: Option<(u32, u32)>,
    pub colortype: Option<tiff::ColorType>,
    // https://awaresystems.be/imaging/tiff/tifftags/photometricinterpretation.html
    pub photometric_interpretation: Option<PhotometricInterpretation>,
    // https://awaresystems.be/imaging/tiff/tifftags/planarconfiguration.html
    pub planar_config: Option<PlanarConfiguration>,
    pub samples: u8,
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

impl<R: Read + Seek + Send> GeoTiffReader<R> {
    /// Open GeoTIFF and read header information
    pub fn open(src: R) -> GeorasterResult<Self> {
        let mut decoder = Decoder::new(src)?;

        // Read GeoTIFF tags
        let geo_keys = decoder.get_tag_u32_vec(Tag::GeoKeyDirectoryTag).ok();
        let geo_params = decoder.get_tag_ascii_string(Tag::GeoAsciiParamsTag).ok();
        let pixel_scale = decoder.get_tag_f64_vec(Tag::ModelPixelScaleTag).ok();
        let model_transformation = decoder.get_tag_f64_vec(Tag::ModelTransformationTag).ok();
        let tie_points = decoder.get_tag_f64_vec(Tag::ModelTiepointTag).ok();
        let _geo_double_params = decoder.get_tag_f64_vec(Tag::GeoDoubleParamsTag).ok();
        let _nodata = decoder.get_tag_ascii_string(Tag::GdalNodata).ok();

        // Read all IFDs
        let mut images = Vec::new();
        loop {
            images.push(ImageInfo::decode(&mut decoder));
            if decoder.more_images() {
                decoder.next_image().expect("Read image info")
            } else {
                break;
            }
        }
        let cur_image_idx = images.len() - 1;

        let reader = GeoTiffReader {
            decoder,
            band_idx: 0,
            images,
            cur_image_idx,
            geo_keys,
            geo_params,
            pixel_scale,
            model_transformation,
            tie_points,
        };

        Ok(reader)
    }

    /// Infos about images.
    pub fn images(&self) -> &Vec<ImageInfo> {
        &self.images
    }

    /// info for current image.
    pub fn image_info(&self) -> &ImageInfo {
        &self.images[self.cur_image_idx]
    }

    /// Load image info into reader
    pub fn seek_to_image(&mut self, index: usize) -> GeorasterResult<()> {
        self.decoder.seek_to_image(index)?;
        self.cur_image_idx = index;
        Ok(())
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

    pub fn select_raster_band(&mut self, band: u8) -> TiffResult<()> {
        if band < 1 || band > self.num_bands() {
            return Err(TiffError::LimitsExceeded);
        }
        self.band_idx = band - 1;
        Ok(())
    }

    /// Image dimensions or (0, 0) if undefined.
    fn dimensions_or_zero(&mut self) -> (u32, u32) {
        self.decoder.dimensions().unwrap_or((0, 0))
    }

    /// Returns the default chunk size for the current image.
    fn chunk_dimensions(&self) -> (u32, u32) {
        self.decoder.chunk_dimensions()
    }

    /// band count of current image.
    fn num_bands(&mut self) -> u8 {
        self.image_info().samples
    }

    /// Samples per pixel
    fn spp(&mut self) -> u8 {
        match self.image_info().planar_config {
            Some(PlanarConfiguration::Planar) => 1,
            _ => {
                match self.decoder.colortype() {
                    Ok(tiff::ColorType::Gray(_)) => 1,
                    Ok(tiff::ColorType::RGB(_)) => 3,
                    Ok(tiff::ColorType::RGBA(_)) => 4,
                    _ => 1, // unsupported
                }
            }
        }
    }

    /// Return raster value at offset x/y
    ///
    /// ```rust
    /// use std::{fs::File, io::BufReader};
    /// use georaster::geotiff::GeoTiffReader;
    ///
    /// let img_file = BufReader::new(File::open("data/tiff/utm.tif").unwrap());
    /// let mut tiff = GeoTiffReader::open(img_file).unwrap();
    ///
    /// let value = tiff.read_pixel(0, 0);
    /// ```
    pub fn read_pixel(&mut self, x: u32, y: u32) -> RasterValue {
        let image_dims = self.dimensions_or_zero();
        if x >= image_dims.0 || y >= image_dims.1 {
            return RasterValue::NoData;
        }
        let chunk_dims = self.chunk_dimensions();
        let tiles =
            TileAttributes::from_dims(image_dims, chunk_dims, self.image_info().planar_config);
        let chunk_index = tiles.get_chunk_index(x, y, self.band_idx);
        let spp = self.spp();
        let offset = tiles.get_chunk_offset(chunk_index, x, y, spp);
        let chunk = self.decoder.read_chunk(chunk_index).unwrap();
        raster_value(&chunk, offset, spp)
    }

    /// Return raster value at geographical location
    ///
    /// This function converts a geolocation to the corresponding pixel location
    /// and reads it with the `read_pixel` function
    ///
    /// ```rust
    /// use std::{fs::File, io::BufReader};
    /// use georaster::{Coordinate, geotiff::GeoTiffReader};
    ///
    /// let img_file = BufReader::new(File::open("data/tiff/utm.tif").unwrap());
    /// let mut tiff = GeoTiffReader::open(img_file).unwrap();
    ///
    /// let location = Coordinate { x: 0.0, y: 0.0 };
    /// let value = tiff.read_pixel_at_location(location);
    /// ```
    pub fn read_pixel_at_location(&mut self, coord: impl Into<Coordinate>) -> RasterValue {
        if let Some((x, y)) = self.coord_to_pixel(coord) {
            self.read_pixel(x, y)
        } else {
            RasterValue::NoData
        }
    }

    /// Returns an Iterator over the pixels of an image part.
    /// The iterator yields the coordinates of each pixel
    /// along with their value
    pub fn pixels(&mut self, x: u32, y: u32, width: u32, height: u32) -> Pixels<R> {
        let image_dims = self.dimensions_or_zero();
        let chunk_dims = self.decoder.chunk_dimensions();
        let dims =
            TileAttributes::from_dims(image_dims, chunk_dims, self.image_info().planar_config);
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
            band_idx: self.band_idx,
            min_x: x,
            min_y: y,
            max_x: x + width,
            max_y: y + height,
        }
    }

    /// Converts a `Coordinate` into pixel based on the geoinformation in the tiff
    ///
    /// Returns the `None` variant when geoinformation is not available.
    pub fn coord_to_pixel(&self, coord: impl Into<Coordinate>) -> Option<(u32, u32)> {
        let (origin_x, origin_y) = self.origin()?.into();
        let (pixel_size_x, pixel_size_y) = self.pixel_size()?.into();
        let coord = coord.into();
        Some((
            ((coord.x - origin_x) / pixel_size_x).round() as u32,
            ((coord.y - origin_y) / pixel_size_y).round() as u32,
        ))
    }

    /// Converts a pixel position into geocoordinates
    ///
    /// Returns the `None` variant when geoinformation is not available.
    pub fn pixel_to_coord(&self, x: u32, y: u32) -> Option<Coordinate> {
        let (origin_x, origin_y) = self.origin()?.into();
        let (pixel_size_x, pixel_size_y) = self.pixel_size()?.into();
        Some(Coordinate {
            x: x as f64 * pixel_size_x + origin_x,
            y: y as f64 * pixel_size_y + origin_y,
        })
    }
}

impl ImageInfo {
    pub fn decode<R: Read + Seek + Send>(decoder: &mut Decoder<R>) -> Self {
        let dimensions = decoder.dimensions().ok();
        let colortype = decoder.colortype().ok();
        let photometric_interpretation = match decoder.get_tag(Tag::PhotometricInterpretation) {
            Ok(ifd::Value::Short(v)) => PhotometricInterpretation::from_u16(v),
            Ok(ifd::Value::Unsigned(v)) => PhotometricInterpretation::from_u16(v as u16),
            _ => None,
        };
        let planar_config = match decoder.get_tag(Tag::PlanarConfiguration) {
            Ok(ifd::Value::Unsigned(v)) => PlanarConfiguration::from_u16(v as u16),
            _ => None,
        };

        let samples = decoder
            .find_tag(Tag::SamplesPerPixel)
            .unwrap()
            .map(ifd::Value::into_u16)
            .transpose()
            .unwrap()
            .unwrap_or(1)
            .try_into()
            .unwrap();

        // https://awaresystems.be/imaging/tiff/tifftags/newsubfiletype.html
        // https://gdal.org/drivers/raster/gtiff.html#internal-nodata-masks
        let _subfile_type = decoder.get_tag_u64(Tag::NewSubfileType);

        ImageInfo {
            dimensions,
            colortype,
            photometric_interpretation,
            planar_config,
            samples,
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
    spp: u8,
    band_idx: u8,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
}

impl<R: Read + Seek> Iterator for Pixels<'_, R> {
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
                self.offset += self.spp as usize;
            } else if self.y % h + 1 < h && self.y + 1 < self.max_y {
                // next row in chunk
                self.y += 1;
                self.x = (self.col * w).max(self.min_x);
                let chunk_index = self.dims.get_chunk_index(self.x, self.y, self.band_idx);
                self.offset = self
                    .dims
                    .get_chunk_offset(chunk_index, self.x, self.y, self.spp);
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

impl<R: Read + Seek> Pixels<'_, R> {
    fn read_chunk(&mut self) {
        let chunk_index = self.dims.get_chunk_index(self.x, self.y, self.band_idx);
        self.row = chunk_index / self.dims.tiles_across() as u32;
        self.col = chunk_index % self.dims.tiles_across() as u32;
        self.chunk = self.decoder.read_chunk(chunk_index);
        self.offset = self
            .dims
            .get_chunk_offset(chunk_index, self.x, self.y, self.spp);
    }
}

fn raster_value(chunk: &DecodingResult, offset: usize, spp: u8) -> RasterValue {
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

    planar_config: PlanarConfiguration,
}

impl TileAttributes {
    pub fn from_dims(
        image_dims: (u32, u32),
        chunk_dims: (u32, u32),
        planar_config: Option<PlanarConfiguration>,
    ) -> Self {
        TileAttributes {
            image_width: image_dims.0 as usize,
            image_height: image_dims.1 as usize,
            tile_width: chunk_dims.0 as usize,
            tile_length: chunk_dims.1 as usize,
            planar_config: planar_config.unwrap_or(PlanarConfiguration::Chunky),
        }
    }
    pub fn tiles_across(&self) -> usize {
        self.image_width.div_ceil(self.tile_width)
    }
    pub fn tiles_down(&self) -> usize {
        self.image_height.div_ceil(self.tile_length)
    }
    fn padding_right(&self) -> usize {
        (self.tile_width - self.image_width % self.tile_width) % self.tile_width
    }
    fn padding_down(&self) -> usize {
        (self.tile_length - self.image_height % self.tile_length) % self.tile_length
    }
    pub fn get_padding(&self, tile: usize) -> (usize, usize) {
        let row = tile / self.tiles_across();
        let column = tile % self.tiles_across();

        let padding_right = if column == self.tiles_across() - 1 {
            self.padding_right()
        } else {
            0
        };

        let padding_down = if row == self.tiles_down() - 1 {
            self.padding_down()
        } else {
            0
        };

        (padding_right, padding_down)
    }

    /// Return tile or stripe index of a pixel
    fn get_chunk_index(&self, x: u32, y: u32, band: u8) -> u32 {
        let x = x as usize;
        let y = y as usize;
        let band = band as usize;
        assert!(x < self.image_width);
        assert!(y < self.image_height);
        let band_offset = match self.planar_config {
            PlanarConfiguration::Planar => (self.image_height / self.tile_length) * band,
            _ => 0,
        };
        let x_chunks = x / self.tile_width;
        let y_chunks = y / self.tile_length;
        let chunk_index = band_offset + y_chunks * self.tiles_across() + x_chunks;
        chunk_index as u32
    }

    /// Return offset of a pixel in tile or stripe
    fn get_chunk_offset(&self, idx: u32, x: u32, y: u32, spp: u8) -> usize {
        let (padding_right, _padding_down) = self.get_padding(idx as usize);
        let x = x as usize;
        let y = y as usize;
        let spp = spp as usize;
        let w = self.tile_width - padding_right;
        let x_offset = x % self.tile_width;
        let y_offset = y % self.tile_length;
        let offset = y_offset * w + x_offset;
        offset * spp
    }
}
