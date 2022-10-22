//! GeoTIFF / COG file reader.

use std::io::{Read, Seek};
use tiff::decoder::{Decoder, DecodingResult};
use tiff::tags::Tag;

/// GeoTIFF file reader
pub struct GeoTiffReader<R: Read + Seek> {
    decoder: Decoder<R>,
    pub geo_keys: Option<Vec<u32>>,
    pub geo_params: Option<String>,
    pixel_scale: Option<Vec<f64>>,
    model_transformation: Option<Vec<f64>>,
    tie_points: Option<Vec<f64>>,
}

impl<R: Read + Seek + Send> GeoTiffReader<R> {
    /// Open GeoTIFF and read header information
    pub fn open(src: R) -> std::io::Result<Self> {
        let mut decoder = Decoder::new(src).expect("Cannot create decoder");

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

    /// Return tile or stripe index containing a pixel
    fn get_chunk_index(&self, x: u64, y: u64) -> u32 {
        /// Computed values useful for tile decoding
        // struct TileAttributes {
        //     pub image_width: usize,
        //     pub image_height: usize,

        //     pub tile_width: usize,
        //     pub tile_length: usize,
        // }

        // impl TileAttributes {
        //     pub fn tiles_across(&self) -> usize {
        //         (self.image_width + self.tile_width - 1) / self.tile_width
        //     }
        //     pub fn tiles_down(&self) -> usize {
        //         (self.image_height + self.tile_length - 1) / self.tile_length
        //     }
        //     fn padding_right(&self) -> usize {
        //         self.tile_width - self.image_width % self.tile_width
        //     }
        //     fn padding_down(&self) -> usize {
        //         self.tile_length - self.image_height % self.tile_length
        //     }

        //     pub fn get_padding(&self, tile: usize) -> (usize, usize) {
        //         let row = tile / self.tiles_across();
        //         let column = tile % self.tiles_across();

        //         let padding_right = if column == self.tiles_across() - 1 {
        //             self.padding_right()
        //         } else {
        //             0
        //         };

        //         let padding_down = if row == self.tiles_down() - 1 {
        //             self.padding_down()
        //         } else {
        //             0
        //         };

        //         (padding_right, padding_down)
        //     }
        // }
        todo!()
    }

    pub fn read_cog(&mut self) {
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

        while self.decoder.more_images() {
            self.decoder.next_image().unwrap();
            dbg!(self.decoder.dimensions().unwrap());
            if let Ok(subfile_type) = self.decoder.get_tag_u64(Tag::NewSubfileType) {
                dbg!(subfile_type);
            }
        }
    }

    pub fn read_dtm(&mut self) {
        let tiles = self.decoder.tile_count().unwrap();

        for tile in 0..tiles {
            match self.decoder.read_chunk(tile).unwrap() {
                DecodingResult::U16(res) => {
                    let _sum: u64 = res.into_iter().map(<u64>::from).sum();
                }
                _ => panic!("Wrong bit depth"),
            }
        }
    }
}
