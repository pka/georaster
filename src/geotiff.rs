//! GeoTIFF / COG file reader.

use std::io::{Read, Seek};
use tiff::decoder::{Decoder, DecodingResult};
use tiff::tags::Tag;
use tiff::ColorType;

/// GeoTIFF file reader
pub struct GeoTiffReader<R: Read + Seek> {
    decoder: Decoder<R>,
}

impl<R: Read + Seek + Send> GeoTiffReader<R> {
    /// Open GeoTIFF and read header information
    pub fn open(src: R) -> std::io::Result<Self> {
        let mut decoder = Decoder::new(src).expect("Cannot create decoder");

        if let Ok(geokeys) = decoder.get_tag_u32_vec(Tag::GeoKeyDirectoryTag) {
            dbg!(geokeys);
        }
        if let Ok(geo_params) = decoder.get_tag_ascii_string(Tag::GeoAsciiParamsTag) {
            dbg!(geo_params);
        }
        if let Ok(model_tiepoint) = decoder.get_tag_f64_vec(Tag::ModelTiepointTag) {
            dbg!(model_tiepoint);
        }
        if let Ok(model_pixel_scale) = decoder.get_tag_f64_vec(Tag::ModelPixelScaleTag) {
            dbg!(model_pixel_scale);
        }

        let reader = GeoTiffReader { decoder };

        Ok(reader)
    }

    /// Image dimensions
    pub fn dimensions(&mut self) -> (u32, u32) {
        self.decoder.dimensions().unwrap()
    }

    pub fn read_cog(&mut self) {
        assert_eq!(self.decoder.colortype().unwrap(), ColorType::RGB(8));

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
        assert_eq!(self.decoder.colortype().unwrap(), ColorType::Gray(16));

        let tiles = self.decoder.tile_count().unwrap();
        assert_eq!(tiles as usize, 100);

        for tile in 0..tiles {
            match self.decoder.read_chunk(tile).unwrap() {
                DecodingResult::U16(res) => {
                    let sum: u64 = res.into_iter().map(<u64>::from).sum();
                    if tile == 0 {
                        assert_eq!(sum, 173214606);
                    }
                }
                _ => panic!("Wrong bit depth"),
            }
        }
    }
}
