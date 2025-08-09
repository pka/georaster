use georaster::{geotiff::GeoTiffReader, RasterValue};
use http_range_client::HttpReader;

fn main() {
    env_logger::init();
    let mut img_reader = HttpReader::new("https://data.sourcepole.com/srtm_1km_3857.tif");
    // File size: 244M
    // gdalinfo:
    // Driver: GTiff/GeoTIFF
    // Files: srtm_1km_3857.tif
    // Size is 41999, 17610
    // Coordinate System is:
    //     ID["EPSG",3857]]
    // Data axis to CRS axis mapping: 1,2
    // Origin = (-20037506.487966008484364,8401593.447238374501467)
    // Pixel Size = (954.174162299386694,-954.196881813704067)
    // Metadata:
    //   AREA_OR_POINT=Area
    // Image Structure Metadata:
    //   COMPRESSION=DEFLATE
    //   INTERLEAVE=BAND
    // Corner Coordinates:
    // Upper Left  (-20037506.488, 8401593.447) (179d59'59.94"W, 60d 0'30.00"N)
    // Lower Left  (-20037506.488,-8401813.642) (179d59'59.94"W, 60d 0'33.56"S)
    // Upper Right (20036854.154, 8401593.447) (179d59'38.84"E, 60d 0'30.00"N)
    // Lower Right (20036854.154,-8401813.642) (179d59'38.84"E, 60d 0'33.56"S)
    // Center      (    -326.167,    -110.097) (  0d 0'10.55"W,  0d 0' 3.56"S)
    // Band 1 Block=256x256 Type=Int32, ColorInterp=Gray
    //   NoData Value=-9999
    //   Overviews: 5250x2202, 657x276, 83x35

    // img_reader.set_min_req_size(1_048_576); // 1MB 6 requests, 6'291'456 B
    // img_reader.set_min_req_size(524288); // 512KB 7 requests, 3'670'016 B
    img_reader.set_min_req_size(262144); // 256KB 7 requests, 1'835'008 B

    let mut tiff = GeoTiffReader::open(img_reader).expect("Cannot create decoder");

    let img = tiff.images().first().expect("Image info");
    assert_eq!(img.dimensions, Some((41999, 17610)));
    assert_eq!(img.colortype, Some(tiff::ColorType::Gray(32)));
    assert_eq!(tiff.origin(), Some([-20037506.48796601, 8401593.447238375]));
    assert_eq!(
        tiff.pixel_size(),
        Some([954.1741622993867, -954.1968818137041])
    );
    assert_eq!(
        tiff.geo_params,
        Some("WGS 84 / Pseudo-Mercator|WGS 84|".to_string())
    );

    tiff.seek_to_image(0).unwrap();
    assert_eq!(tiff.read_pixel(20000, 8000), RasterValue::I32(372));

    // Read medium overview
    tiff.seek_to_image(1).unwrap();
    let max_height = tiff
        .pixels(0, 0, 657, 276)
        .map(|(_x, _y, h)| if let RasterValue::I32(v) = h { v } else { 0 })
        .max();
    assert_eq!(max_height, Some(3405));
}
