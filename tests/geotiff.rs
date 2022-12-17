use georaster::geotiff::{GeoTiffReader, RasterValue};
use std::fs::File;
use std::io::BufReader;
use tiff::tags::PhotometricInterpretation;

#[test]
fn single_band() {
    let img_file =
        BufReader::new(File::open("data/tiff/f32nan_data.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    assert_eq!(tiff.dimensions(), Some((128, 128)));
    assert_eq!(tiff.colortype(), Some(tiff::ColorType::Gray(32)));
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

    assert_eq!(tiff.dimensions(), Some((20, 20)));
    assert_eq!(tiff.colortype(), Some(tiff::ColorType::Gray(8)));
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

    assert_eq!(tiff.dimensions(), Some((20, 20)));
    assert_eq!(tiff.colortype(), Some(tiff::ColorType::Gray(32)));
    assert_eq!(tiff.origin(), Some([440720.0, 3751320.0]));
    assert_eq!(tiff.pixel_size(), Some([60.0, -60.0]));
    assert_eq!(tiff.geo_params, Some("NAD27 / UTM zone 11N|".to_string()));

    assert_eq!(tiff.read_pixel(0, 0), RasterValue::F32(107.0));
}

#[test]
fn int16() {
    let img_file = BufReader::new(File::open("data/tiff/int16.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    assert_eq!(tiff.dimensions(), Some((20, 20)));
    assert_eq!(tiff.colortype(), Some(tiff::ColorType::Gray(16)));
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

    assert_eq!(tiff.dimensions(), Some((20, 20)));
    assert_eq!(tiff.colortype(), Some(tiff::ColorType::Gray(32)));
    assert_eq!(tiff.origin(), Some([440720.0, 3751320.0]));
    assert_eq!(tiff.pixel_size(), Some([60.0, -60.0]));
    assert_eq!(tiff.geo_params, Some("NAD27 / UTM zone 11N|".to_string()));

    assert_eq!(tiff.read_pixel(0, 0), RasterValue::I32(107));
}

#[test]
fn rgbsmall() {
    let img_file = BufReader::new(File::open("data/tiff/rgbsmall.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    assert_eq!(tiff.dimensions(), Some((50, 50)));
    assert_eq!(tiff.colortype(), Some(tiff::ColorType::RGB(8)));
    assert_eq!(tiff.origin(), Some([-44.84032, -22.932584]));
    assert_eq!(tiff.pixel_size(), Some([0.003432, -0.003432]));
    assert_eq!(tiff.geo_params, Some("WGS 84|".to_string()));

    // convert -quiet data/tiff/rgbsmall.tif[0] -crop 1x1+25+25 txt:
    // assert_eq!(tiff.read_pixel(25, 25), RasterValue::U8(107));IoError(Error { kind: UnexpectedEof, message: "failed to fill whole buffer" })
}

#[test]
fn small_world() {
    let img_file =
        BufReader::new(File::open("data/tiff/small_world.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    assert_eq!(tiff.dimensions(), Some((400, 200)));
    assert_eq!(tiff.colortype(), Some(tiff::ColorType::RGB(8)));
    assert_eq!(tiff.origin(), Some([-180.0, 90.0]));
    assert_eq!(tiff.pixel_size(), Some([0.9, -0.9]));
    assert_eq!(tiff.geo_params, Some("WGS 84|".to_string()));

    // convert -quiet data/tiff/small_world.tif[0] -crop 1x1+0+0 txt:
    // 0,0: (11,10,50)  #0B0A32  srgb(11,10,50)
    assert_eq!(tiff.read_pixel(0, 0), RasterValue::Rgb8(11, 11, 11)); //FIXME

    // convert -quiet data/tiff/small_world.tif[0] -crop 1x1+25+25 txt:
    //0,0: (54,62,21)  #363E15  srgb(54,62,21)
    assert_eq!(tiff.read_pixel(25, 25), RasterValue::Rgb8(48, 59, 49)); //FIXME
}

#[test]
fn small_world_pct() {
    let img_file =
        BufReader::new(File::open("data/tiff/small_world_pct.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    assert_eq!(tiff.dimensions(), Some((400, 200)));
    assert_eq!(tiff.colortype(), None);
    assert_eq!(tiff.origin(), Some([-180.0, 90.0]));
    assert_eq!(tiff.pixel_size(), Some([0.9, -0.9]));
    assert_eq!(tiff.geo_params, Some("WGS 84|".to_string()));

    // assert_eq!(tiff.read_pixel(0, 0), RasterValue::U8(107)); // UnsupportedError(InterpretationWithBits(RGBPalette, [8]))
}

#[test]
fn utm() {
    let img_file = BufReader::new(File::open("data/tiff/utm.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    assert_eq!(tiff.dimensions(), Some((512, 512)));
    assert_eq!(tiff.colortype(), Some(tiff::ColorType::Gray(8)));
    assert_eq!(tiff.origin(), Some([440720.0, 3751320.0]));
    assert_eq!(tiff.pixel_size(), Some([60.0, -60.0]));
    assert_eq!(tiff.geo_params, Some("UTM    11 S E000|".to_string()));

    assert_eq!(tiff.read_pixel(0, 0), RasterValue::U8(107));
}

#[test]
fn rgb() {
    let img_file = BufReader::new(File::open("data/tiff/sat.tif").expect("Open image file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    assert_eq!(tiff.dimensions(), Some((200, 200)));
    assert_eq!(tiff.colortype(), Some(tiff::ColorType::RGB(8)));
    assert_eq!(tiff.origin(), Some([2747994.2968, 1205137.2435]));
    assert_eq!(
        tiff.pixel_size(),
        Some([1.8898895579756552, -1.8898895306859578])
    );
    assert_eq!(tiff.geo_params, Some("CH1903+ / LV95|CH1903+|".to_string()));
    assert_eq!(
        tiff.photometric_interpretation,
        Some(PhotometricInterpretation::RGB)
    );
    assert_eq!(tiff.chunk_dimensions(), (512, 512));

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
    let tiff = GeoTiffReader::open(img_file);
    assert!(tiff.is_err()); // FormatError(InconsistentSizesEncountered)
}
