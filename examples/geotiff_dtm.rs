use georaster::geotiff::{GeoTiffReader, RasterValue};
use std::fs::File;
use std::io::BufReader;

fn main() {
    let img_file =
        BufReader::new(File::open("data/tiff/N265E425.tif").expect("Cannot find test image!"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Cannot create decoder");
    // Decoder {
    //     reader: SmartReader {
    //         reader: File {
    //             fd: 3,
    //             path: "data/tiff/N265E425.tif",
    //             read: true,
    //             write: false,
    //         },
    //         byte_order: LittleEndian,
    //     },
    //     bigtiff: false,
    //     limits: Limits {
    //         decoding_buffer_size: 268435456,
    //         ifd_value_size: 1048576,
    //         intermediate_buffer_size: 134217728,
    //         _non_exhaustive: (),
    //     },
    //     current_chunk: 0,
    //     next_ifd: Some(
    //         708,
    //     ),
    //     ifd_offsets: [
    //         192,
    //         708,
    //     ],
    //     seen_ifds: {
    //         192,
    //         708,
    //     },
    //     image: Image {
    //         ifd: Some(
    //             {
    //                 PlanarConfiguration: Entry { type_: SHORT, count: 1, offset: [1, 0, 0, 0, 0, 0, 0, 0] },
    //                 TileLength: Entry { type_: SHORT, count: 1, offset: [0, 2, 0, 0, 0, 0, 0, 0] },
    //                 Compression: Entry { type_: SHORT, count: 1, offset: [8, 0, 0, 0, 0, 0, 0, 0] },
    //                 TileOffsets: Entry { type_: LONG, count: 100, offset: [172, 5, 0, 0, 0, 0, 0, 0] },
    //                 ModelPixelScaleTag: Entry { type_: DOUBLE, count: 3, offset: [170, 1, 0, 0, 0, 0, 0, 0] },
    //                 Predictor: Entry { type_: SHORT, count: 1, offset: [1, 0, 0, 0, 0, 0, 0, 0] },
    //                 SamplesPerPixel: Entry { type_: SHORT, count: 1, offset: [1, 0, 0, 0, 0, 0, 0, 0] },
    //                 ModelTiepointTag: Entry { type_: DOUBLE, count: 6, offset: [194, 1, 0, 0, 0, 0, 0, 0] },
    //                 ImageWidth: Entry { type_: SHORT, count: 1, offset: [136, 19, 0, 0, 0, 0, 0, 0] },
    //                 BitsPerSample: Entry { type_: SHORT, count: 1, offset: [16, 0, 0, 0, 0, 0, 0, 0] },
    //                 ImageLength: Entry { type_: SHORT, count: 1, offset: [136, 19, 0, 0, 0, 0, 0, 0] },
    //                 SampleFormat: Entry { type_: SHORT, count: 1, offset: [1, 0, 0, 0, 0, 0, 0, 0] },
    //                 GeoDoubleParamsTag: Entry { type_: DOUBLE, count: 6, offset: [122, 2, 0, 0, 0, 0, 0, 0] },
    //                 GeoKeyDirectoryTag: Entry { type_: SHORT, count: 68, offset: [242, 1, 0, 0, 0, 0, 0, 0] },
    //                 PhotometricInterpretation: Entry { type_: SHORT, count: 1, offset: [1, 0, 0, 0, 0, 0, 0, 0] },
    //                 TileWidth: Entry { type_: SHORT, count: 1, offset: [0, 2, 0, 0, 0, 0, 0, 0] },
    //                 GeoAsciiParamsTag: Entry { type_: ASCII, count: 25, offset: [170, 2, 0, 0, 0, 0, 0, 0] },
    //                 GdalNodata: Entry { type_: ASCII, count: 2, offset: [48, 0, 0, 0, 0, 0, 0, 0] },
    //                 TileByteCounts: Entry { type_: LONG, count: 100, offset: [60, 7, 0, 0, 0, 0, 0, 0] },
    //             },
    //         ),
    //         width: 5000,
    //         height: 5000,
    //         bits_per_sample: [
    //             16,
    //         ],
    //         samples: 1,
    //         sample_format: [
    //             Uint,
    //         ],
    //         photometric_interpretation: BlackIsZero,
    //         compression_method: Deflate,
    //         predictor: None,
    //         jpeg_tables: None,
    //         chunk_type: Tile,
    //         strip_decoder: None,
    //         tile_attributes: Some(
    //             TileAttributes {
    //                 image_width: 5000,
    //                 image_height: 5000,
    //                 tile_width: 512,
    //                 tile_length: 512,
    //             },
    //         ),
    //         chunk_offsets: [
    //             8560602,
    //
    //             16735122,
    //         ],
    //         chunk_bytes: [
    //             57698,
    //
    //             46224,
    //         ],
    //     },
    // }

    assert_eq!(tiff.dimensions(), (5000, 5000));
    assert_eq!(tiff.colortype(), Some(tiff::ColorType::Gray(16)));
    assert_eq!(tiff.origin(), Some([4250000.0, 2700000.0]));
    assert_eq!(tiff.pixel_size(), Some([10.0, -10.0]));
    assert_eq!(
        tiff.geo_params,
        Some("ETRS89_ETRS_LAEA|ETRS89|".to_string())
    );

    assert_eq!(tiff.read_pixel(0, 0), RasterValue::U16(636));

    let max_height = tiff
        .pixels(2500, 3000, 100, 100)
        .map(|(_x, _y, h)| if let RasterValue::U16(v) = h { v } else { 0 })
        .max();
    assert_eq!(max_height, Some(2161));
}
