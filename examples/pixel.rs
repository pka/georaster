use georaster::geotiff::GeoTiffReader;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;

fn main() {
    let src_fn = env::args().nth(1).expect("Input file required");
    let x = env::args()
        .nth(2)
        .map(|s| u32::from_str(&s).expect("Invalid number"))
        .expect("X required");
    let y = env::args()
        .nth(3)
        .map(|s| u32::from_str(&s).expect("Invalid number"))
        .expect("Y required");

    let img_file = BufReader::new(File::open(src_fn).expect("Open input file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");
    let pixel = tiff.read_pixel(x, y).unwrap();
    println!("{pixel}");
}
