use georaster::geotiff::{GeoTiffReader, RasterValue};
use image::ImageBuffer;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;

fn main() {
    let src_fn = env::args().nth(1).expect("Input file required");
    let window = env::args().nth(2).expect("Window size required");
    let dst_fn = env::args().nth(3).expect("Output file required");

    // window size example: 100x100+2500+3000
    let parts: Vec<_> = window
        .split(&['x', '+'])
        .map(|s| u32::from_str(s).expect("Invalid number"))
        .collect();
    let w = *parts.first().expect("witdh missing");
    let h = *parts.get(1).expect("height missing");
    let x0 = *parts.get(2).unwrap_or(&0);
    let y0 = *parts.get(3).unwrap_or(&0);

    let img_file = BufReader::new(File::open(src_fn).expect("Open input file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");
    let mut img = ImageBuffer::new(w, h);
    for (x, y, pixel) in tiff.pixels(x0, y0, w, h) {
        if let RasterValue::U16(v) = pixel {
            img.put_pixel(x - x0, y - y0, image::Luma([v]));
        }
    }
    img.save(dst_fn).unwrap();
}
