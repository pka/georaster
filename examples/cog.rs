use georaster::geotiff::GeoTiffReader;
use std::fs::File;
use std::io::BufReader;

fn main() {
    // https://gdal.org/drivers/raster/cog.html
    let img_file =
        BufReader::new(File::open("data/tiff/N265E425.tif").expect("Cannot find test image!"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Cannot create decoder");

    for (idx, img) in tiff.images().iter().enumerate() {
        println!("Image #{idx}: {img:?}");
    }
    tiff.read_cog();
}
