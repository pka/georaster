use georaster::geotiff::GeoTiffReader;
use std::fs::File;
use std::io::BufReader;

fn main() {
    // https://gdal.org/drivers/raster/cog.html
    let img_file =
        BufReader::new(File::open("data/tiff/sat.tif").expect("Cannot find test image!"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Cannot create decoder");

    assert_eq!(tiff.colortype(), Some(tiff::ColorType::RGB(8)));

    tiff.read_cog();
}
