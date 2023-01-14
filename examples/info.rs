use georaster::geotiff::GeoTiffReader;
use std::env;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let src_fn = env::args().nth(1).expect("Input file required");
    let img_file = BufReader::new(File::open(src_fn).expect("Open input file"));
    let tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    println!("Origin: {:?}", tiff.origin());
    println!("Pixel size: {:?}", tiff.pixel_size());
    println!("SRS: {:?}", tiff.geo_params);
    for (idx, img) in tiff.images().iter().enumerate() {
        println!("Image #{idx}:");
        println!("  Dimensions: {:?}", img.dimensions);
        println!("  Color: {:?}", img.colortype);
        println!(
            "  Photometric interpretation: {:?}",
            img.photometric_interpretation
        );
        println!("  Planar config: {:?}", img.planar_config);
    }
}
