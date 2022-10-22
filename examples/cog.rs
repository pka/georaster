use georaster::geotiff::GeoTiffReader;
use std::fs::File;
use std::io::BufReader;

fn main() {
    // https://gdal.org/drivers/raster/cog.html
    let img_file = BufReader::new(File::open("imagery/seen.tif").expect("Cannot find test image!"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Cannot create decoder");
    // Decoder {
    //     reader: SmartReader {
    //         reader: File {
    //             fd: 3,
    //             path: "imagery/seen.tif",
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
    //         562,
    //     ),
    //     ifd_offsets: [
    //         192,
    //         562,
    //     ],
    //     seen_ifds: {
    //         192,
    //         562,
    //     },
    //     image: Image {
    //         ifd: Some(
    //             {
    //                 TileByteCounts: Entry { type_: LONG, count: 8, offset: [174, 3, 0, 0, 0, 0, 0, 0] },
    //                 ModelPixelScaleTag: Entry { type_: DOUBLE, count: 3, offset: [146, 1, 0, 0, 0, 0, 0, 0] },
    //                 TileOffsets: Entry { type_: LONG, count: 8, offset: [142, 3, 0, 0, 0, 0, 0, 0] },
    //                 SampleFormat: Entry { type_: SHORT, count: 3, offset: [140, 1, 0, 0, 0, 0, 0, 0] },
    //                 ModelTiepointTag: Entry { type_: DOUBLE, count: 6, offset: [170, 1, 0, 0, 0, 0, 0, 0] },
    //                 GeoKeyDirectoryTag: Entry { type_: SHORT, count: 32, offset: [218, 1, 0, 0, 0, 0, 0, 0] },
    //                 TileLength: Entry { type_: SHORT, count: 1, offset: [0, 2, 0, 0, 0, 0, 0, 0] },
    //                 ImageWidth: Entry { type_: SHORT, count: 1, offset: [25, 6, 0, 0, 0, 0, 0, 0] },
    //                 GeoAsciiParamsTag: Entry { type_: ASCII, count: 24, offset: [26, 2, 0, 0, 0, 0, 0, 0] },
    //                 TileWidth: Entry { type_: SHORT, count: 1, offset: [0, 2, 0, 0, 0, 0, 0, 0] },
    //                 Compression: Entry { type_: SHORT, count: 1, offset: [1, 0, 0, 0, 0, 0, 0, 0] },
    //                 PlanarConfiguration: Entry { type_: SHORT, count: 1, offset: [1, 0, 0, 0, 0, 0, 0, 0] },
    //                 SamplesPerPixel: Entry { type_: SHORT, count: 1, offset: [3, 0, 0, 0, 0, 0, 0, 0] },
    //                 ImageLength: Entry { type_: SHORT, count: 1, offset: [63, 3, 0, 0, 0, 0, 0, 0] },
    //                 BitsPerSample: Entry { type_: SHORT, count: 3, offset: [134, 1, 0, 0, 0, 0, 0, 0] },
    //                 PhotometricInterpretation: Entry { type_: SHORT, count: 1, offset: [2, 0, 0, 0, 0, 0, 0, 0] },
    //             },
    //         ),
    //         width: 1561,
    //         height: 831,
    //         bits_per_sample: [
    //             8,
    //             8,
    //             8,
    //         ],
    //         samples: 3,
    //         sample_format: [
    //             Uint,
    //             Uint,
    //             Uint,
    //         ],
    //         photometric_interpretation: RGB,
    //         compression_method: None,
    //         predictor: None,
    //         jpeg_tables: None,
    //         chunk_type: Tile,
    //         strip_decoder: None,
    //         tile_attributes: Some(
    //             TileAttributes {
    //                 image_width: 1561,
    //                 image_height: 831,
    //                 tile_width: 512,
    //                 tile_length: 512,
    //             },
    //         ),
    //         chunk_offsets: [
    //             2360314,
    //             3146754,
    //             3933194,
    //             4719634,
    //             5506074,
    //             6292514,
    //             7078954,
    //             7865394,
    //         ],
    //         chunk_bytes: [
    //             786432,
    //             786432,
    //             786432,
    //             786432,
    //             786432,
    //             786432,
    //             786432,
    //             786432,
    //         ],
    //     }
    // }

    assert_eq!(tiff.colortype(), Some(tiff::ColorType::RGB(8)));

    tiff.read_cog();
}
