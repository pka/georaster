use georaster::geotiff::{GeoTiffReader, RasterValue};
use image::ImageBuffer;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let img_file =
        BufReader::new(File::open("data/tiff/N265E425.tif").expect("Cannot find test image!"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Cannot create decoder");
    // convert data/tiff/N265E425.tif[0] -crop 100x100+2500+3000 dtm.png
    let (x0, y0, w, h) = (2500, 3000, 100, 100);
    let mut img = ImageBuffer::new(w, h);
    for (x, y, pixel) in tiff.pixels(x0, y0, w, h) {
        if let RasterValue::U16(v) = pixel {
            img.put_pixel(x - x0, y - y0, image::Luma([v]));
        }
    }
    img.save("dtm.png").unwrap();
}
