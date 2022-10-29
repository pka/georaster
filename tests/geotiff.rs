use georaster::geotiff::{GeoTiffReader, RasterValue};
use std::fs::File;
use std::io::BufReader;

#[test]
fn single_band() {
    let img_file =
        BufReader::new(File::open("data/tiff/f32nan_data.tif").expect("Cannot find test image!"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Cannot create decoder");

    assert_eq!(tiff.dimensions(), (128, 128));
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
    let mut pixels = tiff.pixels(102, 15, 2, 2);
    assert_eq!(
        pixels.next(),
        Some((102, 15, RasterValue::F32(0.023571081)))
    );
    assert_eq!(
        pixels.next(),
        Some((103, 15, RasterValue::F32(0.012901229)))
    );
    assert_eq!(pixels.next(), Some((102, 16, RasterValue::F32(0.305))));
    assert_eq!(pixels.next(), Some((103, 16, RasterValue::F32(0.6975))));
    assert_eq!(pixels.next(), None);

    // Test 0
    let mut pixels = tiff.pixels(0, 0, 0, 0);
    assert_eq!(pixels.next().map(|(x, y, _nan)| (x, y)), Some((0, 0)));
    assert_eq!(pixels.next(), None);
}
