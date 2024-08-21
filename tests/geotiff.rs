use georaster::{
    geotiff::{GeoTiffReader, RasterValue},
    Coordinate,
};
use std::fs::File;
use std::io::BufReader;
use tiff::tags::PhotometricInterpretation;

#[test]
fn single_band() {
    let img_file =
        BufReader::new(File::open("data/tiff/f32nan_data.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    let img = tiff.images().get(0).expect("Image info");
    assert_eq!(img.dimensions, Some((128, 128)));
    assert_eq!(img.colortype, Some(tiff::ColorType::Gray(32)));
    assert_eq!(tiff.origin(), Some([0.0, 0.0]));
    assert_eq!(tiff.pixel_size(), Some([1.0, 1.0]));
    assert_eq!(tiff.geo_params, None);

    // convert -quiet data/tiff/f32nan_data.tif[0] -crop 1x1+124+9 txt:
    assert_eq!(tiff.read_pixel(124, 9), RasterValue::F32(0.0050608707));

    // NaN - comparison is always false
    if let RasterValue::F32(val) = tiff.read_pixel(0, 0) {
        assert!(val.is_nan());
    } else {
        assert!(false, "RasterValue::F32(_)")
    }

    // x > width
    assert_eq!(tiff.read_pixel(128, 64), RasterValue::NoData);

    // y > height
    assert_eq!(tiff.read_pixel(64, 128), RasterValue::NoData);

    // convert -quiet data/tiff/f32nan_data.tif[0] -crop 2x2+102+15 txt:
    // 0,0: gray(2.35752%)
    // 1,0: gray(1.28939%)
    // 0,1: gray(30.4997%)
    // 1,1: gray(69.7505%)
    let pixels: Vec<_> = tiff.pixels(102, 15, 2, 2).collect();
    assert_eq!(
        pixels,
        vec![
            (102, 15, RasterValue::F32(0.023571081)),
            (103, 15, RasterValue::F32(0.012901229)),
            (102, 16, RasterValue::F32(0.305)),
            (103, 16, RasterValue::F32(0.6975))
        ]
    );

    // Test 0
    let mut pixels = tiff.pixels(0, 0, 0, 0);
    assert_eq!(pixels.next().map(|(x, y, _nan)| (x, y)), Some((0, 0)));
    assert_eq!(pixels.next(), None);
}

#[test]
fn byte() {
    let img_file = BufReader::new(File::open("data/tiff/byte.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    let img = tiff.images().get(0).expect("Image info");
    assert_eq!(img.dimensions, Some((20, 20)));
    assert_eq!(img.colortype, Some(tiff::ColorType::Gray(8)));
    assert_eq!(tiff.origin(), Some([440720.0, 3751320.0]));
    assert_eq!(tiff.pixel_size(), Some([60.0, -60.0]));
    assert_eq!(tiff.geo_params, Some("NAD27 / UTM zone 11N|".to_string()));

    // convert -quiet data/tiff/byte.tif[0] -crop 1x1+0+0 txt:
    assert_eq!(tiff.read_pixel(0, 0), RasterValue::U8(107));
}

#[test]
fn float32() {
    let img_file = BufReader::new(File::open("data/tiff/float32.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    let img = tiff.images().get(0).expect("Image info");
    assert_eq!(img.dimensions, Some((20, 20)));
    assert_eq!(img.colortype, Some(tiff::ColorType::Gray(32)));
    assert_eq!(tiff.origin(), Some([440720.0, 3751320.0]));
    assert_eq!(tiff.pixel_size(), Some([60.0, -60.0]));
    assert_eq!(tiff.geo_params, Some("NAD27 / UTM zone 11N|".to_string()));

    assert_eq!(tiff.read_pixel(0, 0), RasterValue::F32(107.0));
}

#[test]
fn int16() {
    let img_file = BufReader::new(File::open("data/tiff/int16.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    let img = tiff.images().get(0).expect("Image info");
    assert_eq!(img.dimensions, Some((20, 20)));
    assert_eq!(img.colortype, Some(tiff::ColorType::Gray(16)));
    assert_eq!(tiff.origin(), Some([440720.0, 3751320.0]));
    assert_eq!(tiff.pixel_size(), Some([60.0, -60.0]));
    assert_eq!(tiff.geo_params, Some("NAD27 / UTM zone 11N|".to_string()));

    // convert -quiet data/tiff/int16.tif[0] -crop 1x1+0+0 txt:
    assert_eq!(tiff.read_pixel(0, 0), RasterValue::I16(107));
}

#[test]
fn int32() {
    let img_file = BufReader::new(File::open("data/tiff/int32.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    let img = tiff.images().get(0).expect("Image info");
    assert_eq!(img.dimensions, Some((20, 20)));
    assert_eq!(img.colortype, Some(tiff::ColorType::Gray(32)));
    assert_eq!(tiff.origin(), Some([440720.0, 3751320.0]));
    assert_eq!(tiff.pixel_size(), Some([60.0, -60.0]));
    assert_eq!(tiff.geo_params, Some("NAD27 / UTM zone 11N|".to_string()));

    assert_eq!(tiff.read_pixel(0, 0), RasterValue::I32(107));
}

#[test]
fn rgbsmall() {
    let img_file = BufReader::new(File::open("data/tiff/rgbsmall.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");
    let img = tiff.images().get(0).expect("Image info");
    assert_eq!(img.dimensions, Some((50, 50)));
    assert_eq!(img.colortype, Some(tiff::ColorType::RGB(8)));
    assert_eq!(tiff.origin(), Some([-44.84032, -22.932584]));
    assert_eq!(tiff.pixel_size(), Some([0.003432, -0.003432]));
    assert_eq!(tiff.geo_params, Some("WGS 84|".to_string()));
    assert_eq!(
        img.planar_config,
        Some(tiff::tags::PlanarConfiguration::Planar)
    );

    // convert -quiet data/tiff/rgbsmall.tif[0] -crop 1x1+25+25 txt:
    // 0,0: (89,123,37)  #597B25  srgb(89,123,37)
    assert_eq!(tiff.read_pixel(25, 25), RasterValue::U8(89));
}

#[test]
fn small_world() {
    let img_file =
        BufReader::new(File::open("data/tiff/small_world.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    // Decoder {
    //     reader: SmartReader {
    //         reader: BufReader {
    //             reader: File {
    //                 fd: 3,
    //                 path: "./data/tiff/small_world.tif",
    //                 read: true,
    //                 write: false,
    //             },
    //             buffer: 56/8192,
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
    //     next_ifd: None,
    //     ifd_offsets: [
    //         240008,
    //     ],
    //     seen_ifds: {
    //         240008,
    //     },
    //     image: Image {
    //         ifd: Some(
    //             {
    //                 ModelPixelScaleTag: Entry { type_: DOUBLE, count: 3, offset: [62, 171, 3, 0, 0, 0, 0, 0] },
    //                 GeoKeyDirectoryTag: Entry { type_: SHORT, count: 24, offset: [134, 171, 3, 0, 0, 0, 0, 0] },
    //                 ImageLength: Entry { type_: SHORT, count: 1, offset: [200, 0, 0, 0, 0, 0, 0, 0] },
    //                 SampleFormat: Entry { type_: SHORT, count: 3, offset: [56, 171, 3, 0, 0, 0, 0, 0] },
    //                 ImageWidth: Entry { type_: SHORT, count: 1, offset: [144, 1, 0, 0, 0, 0, 0, 0] },
    //                 Compression: Entry { type_: SHORT, count: 1, offset: [1, 0, 0, 0, 0, 0, 0, 0] },
    //                 SamplesPerPixel: Entry { type_: SHORT, count: 1, offset: [3, 0, 0, 0, 0, 0, 0, 0] },
    //                 ModelTiepointTag: Entry { type_: DOUBLE, count: 6, offset: [86, 171, 3, 0, 0, 0, 0, 0] },
    //                 GeoAsciiParamsTag: Entry { type_: ASCII, count: 8, offset: [182, 171, 3, 0, 0, 0, 0, 0] },
    //                 PlanarConfiguration: Entry { type_: SHORT, count: 1, offset: [2, 0, 0, 0, 0, 0, 0, 0] },
    //                 RowsPerStrip: Entry { type_: SHORT, count: 1, offset: [20, 0, 0, 0, 0, 0, 0, 0] },
    //                 StripByteCounts: Entry { type_: LONG, count: 30, offset: [192, 170, 3, 0, 0, 0, 0, 0] },
    //                 BitsPerSample: Entry { type_: SHORT, count: 3, offset: [66, 170, 3, 0, 0, 0, 0, 0] },
    //                 StripOffsets: Entry { type_: LONG, count: 30, offset: [72, 170, 3, 0, 0, 0, 0, 0] },
    //                 PhotometricInterpretation: Entry { type_: SHORT, count: 1, offset: [2, 0, 0, 0, 0, 0, 0, 0] },
    //             },
    //         ),
    //         width: 400,
    //         height: 200,
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
    //         chunk_type: Strip,
    //         strip_decoder: Some(
    //             StripDecodeState {
    //                 rows_per_strip: 20,
    //             },
    //         ),
    //         tile_attributes: None,
    //         chunk_offsets: [
    //         ...

    let img = tiff.images().get(0).expect("Image info");
    assert_eq!(img.dimensions, Some((400, 200)));
    assert_eq!(img.colortype, Some(tiff::ColorType::RGB(8)));
    assert_eq!(tiff.origin(), Some([-180.0, 90.0]));
    assert_eq!(tiff.pixel_size(), Some([0.9, -0.9]));
    assert_eq!(tiff.geo_params, Some("WGS 84|".to_string()));
    assert_eq!(
        img.planar_config,
        Some(tiff::tags::PlanarConfiguration::Planar)
    );

    // convert -quiet data/tiff/small_world.tif[0] -crop 1x1+0+0 txt:
    // 0,0: (11,10,50)  #0B0A32  srgb(11,10,50)
    assert_eq!(tiff.read_pixel(0, 0), RasterValue::U8(11));
    tiff.select_raster_band(2).unwrap();
    assert_eq!(tiff.read_pixel(0, 0), RasterValue::U8(10));
    tiff.select_raster_band(3).unwrap();
    assert_eq!(tiff.read_pixel(0, 0), RasterValue::U8(50));

    // convert -quiet data/tiff/small_world.tif[0] -crop 1x1+399+199 txt:
    // 0,0: (214,204,194)  #D6CCC2  srgb(214,204,194)
    tiff.select_raster_band(1).unwrap();
    assert_eq!(tiff.read_pixel(399, 199), RasterValue::U8(214));
    tiff.select_raster_band(2).unwrap();
    assert_eq!(tiff.read_pixel(399, 199), RasterValue::U8(204));
    tiff.select_raster_band(3).unwrap();
    assert_eq!(tiff.read_pixel(399, 199), RasterValue::U8(194));

    // convert -quiet data/tiff/small_world.tif[0] -crop 2x2+30+30 txt:
    // 0,0: (76,83,52)  #4C5334  srgb(76,83,52)
    // 1,0: (132,140,116)  #848C74  srgb(132,140,116)
    // 0,1: (149,148,128)  #959480  srgb(149,148,128)
    // 1,1: (46,69,13)  #2E450D  srgb(46,69,13)
    tiff.select_raster_band(1).unwrap();
    let pixels: Vec<_> = tiff.pixels(30, 30, 2, 2).map(|(_x, _y, px)| px).collect();
    assert_eq!(
        pixels,
        vec!(
            RasterValue::U8(76),
            RasterValue::U8(132),
            RasterValue::U8(149),
            RasterValue::U8(46)
        )
    );
    tiff.select_raster_band(2).unwrap();
    let pixels: Vec<_> = tiff.pixels(30, 30, 2, 2).map(|(_x, _y, px)| px).collect();
    assert_eq!(
        pixels,
        vec!(
            RasterValue::U8(83),
            RasterValue::U8(140),
            RasterValue::U8(148),
            RasterValue::U8(69)
        )
    );
    tiff.select_raster_band(3).unwrap();
    let pixels: Vec<_> = tiff.pixels(30, 30, 2, 2).map(|(_x, _y, px)| px).collect();
    assert_eq!(
        pixels,
        vec!(
            RasterValue::U8(52),
            RasterValue::U8(116),
            RasterValue::U8(128),
            RasterValue::U8(13)
        )
    );
}

#[test]
fn small_world_pct() {
    let img_file =
        BufReader::new(File::open("data/tiff/small_world_pct.tif").expect("Open image file"));
    let tiff = GeoTiffReader::open(img_file).expect("Open Tiff");
    let img = tiff.images().get(0).expect("Image info");
    assert_eq!(img.dimensions, Some((400, 200)));
    assert_eq!(img.colortype, None);
    assert_eq!(tiff.origin(), Some([-180.0, 90.0]));
    assert_eq!(tiff.pixel_size(), Some([0.9, -0.9]));
    assert_eq!(tiff.geo_params, Some("WGS 84|".to_string()));
    assert_eq!(
        img.photometric_interpretation,
        Some(PhotometricInterpretation::RGBPalette)
    );
    // Palette is not supported
    // assert_eq!(tiff.read_pixel(0, 0), RasterValue::U8(107)); // UnsupportedError(InterpretationWithBits(RGBPalette, [8]))
}

#[test]
fn utm() {
    let img_file = BufReader::new(File::open("data/tiff/utm.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");
    let img = tiff.images().get(0).expect("Image info");
    assert_eq!(img.dimensions, Some((512, 512)));
    assert_eq!(img.colortype, Some(tiff::ColorType::Gray(8)));
    assert_eq!(tiff.origin(), Some([440720.0, 3751320.0]));
    assert_eq!(tiff.pixel_size(), Some([60.0, -60.0]));
    assert_eq!(tiff.geo_params, Some("UTM    11 S E000|".to_string()));
    assert_eq!(
        img.photometric_interpretation,
        Some(PhotometricInterpretation::BlackIsZero)
    );

    assert_eq!(tiff.read_pixel(0, 0), RasterValue::U8(107));
}

#[test]
fn rgb() {
    let img_file = BufReader::new(File::open("data/tiff/sat.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    // Decoder {
    //     reader: SmartReader {
    //         reader: BufReader {
    //             reader: File {
    //                 fd: 7,
    //                 path: "./data/tiff/sat.tif",
    //                 read: true,
    //                 write: false,
    //             },
    //             buffer: 8144/8192,
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
    //     next_ifd: None,
    //     ifd_offsets: [
    //         192,
    //     ],
    //     seen_ifds: {
    //         192,
    //     },
    //     image: Image {
    //         ifd: Some(
    //             {
    //                 Predictor: Entry { type_: SHORT, count: 1, offset: [1, 0, 0, 0, 0, 0, 0, 0] },
    //                 TileLength: Entry { type_: SHORT, count: 1, offset: [0, 2, 0, 0, 0, 0, 0, 0] },
    //                 TileWidth: Entry { type_: SHORT, count: 1, offset: [0, 2, 0, 0, 0, 0, 0, 0] },
    //                 ImageWidth: Entry { type_: SHORT, count: 1, offset: [200, 0, 0, 0, 0, 0, 0, 0] },
    //                 PhotometricInterpretation: Entry { type_: SHORT, count: 1, offset: [2, 0, 0, 0, 0, 0, 0, 0] },
    //                 BitsPerSample: Entry { type_: SHORT, count: 3, offset: [146, 1, 0, 0, 0, 0, 0, 0] },
    //                 TileByteCounts: Entry { type_: LONG, count: 1, offset: [23, 147, 1, 0, 0, 0, 0, 0] },
    //                 SamplesPerPixel: Entry { type_: SHORT, count: 1, offset: [3, 0, 0, 0, 0, 0, 0, 0] },
    //                 GeoAsciiParamsTag: Entry { type_: ASCII, count: 24, offset: [38, 2, 0, 0, 0, 0, 0, 0] },
    //                 GeoKeyDirectoryTag: Entry { type_: SHORT, count: 32, offset: [230, 1, 0, 0, 0, 0, 0, 0] },
    //                 ImageLength: Entry { type_: SHORT, count: 1, offset: [200, 0, 0, 0, 0, 0, 0, 0] },
    //                 PlanarConfiguration: Entry { type_: SHORT, count: 1, offset: [1, 0, 0, 0, 0, 0, 0, 0] },
    //                 SampleFormat: Entry { type_: SHORT, count: 3, offset: [152, 1, 0, 0, 0, 0, 0, 0] },
    //                 TileOffsets: Entry { type_: LONG, count: 1, offset: [66, 2, 0, 0, 0, 0, 0, 0] },
    //                 Compression: Entry { type_: SHORT, count: 1, offset: [8, 0, 0, 0, 0, 0, 0, 0] },
    //                 ModelTiepointTag: Entry { type_: DOUBLE, count: 6, offset: [182, 1, 0, 0, 0, 0, 0, 0] },
    //                 ModelPixelScaleTag: Entry { type_: DOUBLE, count: 3, offset: [158, 1, 0, 0, 0, 0, 0, 0] },
    //             },
    //         ),
    //         width: 200,
    //         height: 200,
    //         bits_per_sample: [
    //             8,
    //             8,
    //             8,
    //         ],
    //         samples: 3,
    //         sample_format: [
    //             Uint,
    //             Uint,
    //         ],
    //         photometric_interpretation: RGB,
    //         compression_method: Deflate,
    //         predictor: None,
    //         jpeg_tables: None,
    //         chunk_type: Tile,
    //         strip_decoder: None,
    //         tile_attributes: Some(
    //             TileAttributes {
    //                 image_width: 200,
    //                 image_height: 200,
    //                 tile_width: 512,
    //                 tile_length: 512,
    //             },
    //         ),
    //         chunk_offsets: [
    //             578,
    //         ],
    //         chunk_bytes: [
    //             103191,
    //         ],
    //     },
    // }
    let img = tiff.images().get(0).expect("Image info");
    assert_eq!(img.dimensions, Some((200, 200)));
    assert_eq!(img.colortype, Some(tiff::ColorType::RGB(8)));
    assert_eq!(tiff.origin(), Some([2747994.2968, 1205137.2435]));
    assert_eq!(
        tiff.pixel_size(),
        Some([1.8898895579756552, -1.8898895306859578])
    );
    assert_eq!(tiff.geo_params, Some("CH1903+ / LV95|CH1903+|".to_string()));
    assert_eq!(
        img.photometric_interpretation,
        Some(PhotometricInterpretation::RGB)
    );

    // convert -quiet data/tiff/sat.tif[0] -crop 2x2+0+0 txt:
    // 0,0: (59,65,27)  #3B411B  srgb(59,65,27)
    // 1,0: (63,69,31)  #3F451F  srgb(63,69,31)
    // 0,1: (53,64,22)  #354016  srgb(53,64,22)
    // 1,1: (59,70,30)  #3B461E  srgb(59,70,30)
    assert_eq!(tiff.read_pixel(0, 0), RasterValue::Rgb8(59, 65, 27));
    assert_eq!(tiff.read_pixel(1, 0), RasterValue::Rgb8(63, 69, 31));
    assert_eq!(tiff.read_pixel(0, 1), RasterValue::Rgb8(53, 64, 22));
    assert_eq!(tiff.read_pixel(1, 1), RasterValue::Rgb8(59, 70, 30));

    let pixels: Vec<_> = tiff.pixels(0, 0, 2, 2).map(|(_x, _y, px)| px).collect();
    assert_eq!(
        pixels,
        vec!(
            RasterValue::Rgb8(59, 65, 27),
            RasterValue::Rgb8(63, 69, 31),
            RasterValue::Rgb8(53, 64, 22),
            RasterValue::Rgb8(59, 70, 30)
        )
    );
    // convert -quiet data/tiff/sat.tif[0] -crop 2x2+198+198 txt:
    // 0,0: (27,21,7)   #1B1507  srgb(27,21,7)
    // 1,0: (13,8,0)    #0D0800  srgb(13,8,0)
    // 0,1: (21,12,7)   #150C07  srgb(21,12,7)
    // 1,1: (25,15,13)  #190F0D  srgb(25,15,13)
    let pixels: Vec<_> = tiff.pixels(198, 198, 2, 2).map(|(_x, _y, px)| px).collect();
    assert_eq!(
        pixels,
        vec!(
            RasterValue::Rgb8(27, 21, 7),
            RasterValue::Rgb8(13, 8, 0),
            RasterValue::Rgb8(21, 12, 7),
            RasterValue::Rgb8(25, 15, 13)
        )
    );
}

#[test]
fn rgb_bands() {
    let img_file =
        BufReader::new(File::open("data/tiff/sat_multiband.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");
    let img = tiff.images().get(0).expect("Image info");
    assert_eq!(img.dimensions, Some((200, 200)));
    assert_eq!(img.colortype, Some(tiff::ColorType::RGB(8)));
    assert_eq!(tiff.origin(), Some([2747994.2968, 1205137.2435]));
    assert_eq!(
        tiff.pixel_size(),
        Some([1.8898895579756552, -1.8898895306859578])
    );
    assert_eq!(tiff.geo_params, Some("CH1903+ / LV95|CH1903+|".to_string()));

    // convert -quiet data/tiff/sat_multiband.tif[0] -crop 1x1+124+9 txt:
    assert_eq!(tiff.read_pixel(124, 9), RasterValue::U8(18));
}

#[test]
fn read_coord() {
    let img_file =
        BufReader::new(File::open("data/tiff/small_world.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    let location = Coordinate { x: -90.0, y: 45.0 };
    let (pixel_x, pixel_y) = tiff.coord_to_pixel(&location).unwrap();

    let value = tiff.read_pixel_at_location(&location);

    assert_eq!(value, tiff.read_pixel(pixel_x, pixel_y));

    assert_eq!(value, RasterValue::U8(60));
}

#[test]
fn convert_pixel_coordinates() {
    let img_file =
        BufReader::new(File::open("data/tiff/small_world.tif").expect("Open image file"));
    let tiff = GeoTiffReader::open(img_file).expect("Open Tiff");
    let location = Coordinate { x: 0.0, y: 0.0 };

    let (x, y) = tiff.coord_to_pixel(&location).unwrap();
    assert_eq!(x, 200);
    assert_eq!(y, 100);
    let rev_location = tiff.pixel_to_coord(x, y).unwrap();
    assert_eq!(location, rev_location);

    let location = Coordinate { x: -90.0, y: 45.0 };

    let (x, y) = tiff.coord_to_pixel(&location).unwrap();
    assert_eq!(x, 100);
    assert_eq!(y, 50);
    let rev_location = tiff.pixel_to_coord(x, y).unwrap();
    assert_eq!(location, rev_location);
}
