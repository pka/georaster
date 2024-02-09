# georaster

[![crates.io version](https://img.shields.io/crates/v/georaster.svg)](https://crates.io/crates/georaster)
[![docs.rs docs](https://docs.rs/georaster/badge.svg)](https://docs.rs/georaster)

Rust library for accessing geospatial raster images.


## Usage examples

Read pixel value from GeoTIFF:
```rust
let img_file = BufReader::new(File::open("N265E425.tif").unwrap());
let mut tiff = GeoTiffReader::open(img_file).unwrap();
match tiff.read_pixel(x, y) {
    RasterValue::U16(v) => println!("Height: {v}"),
    _ => println!("Unexpected pixel type"),
};
```

Extract part of GeoTIFF into a PNG:
```rust
let img_file = BufReader::new(File::open("N265E425.tif").unwrap());
let mut tiff = GeoTiffReader::open(img_file).unwrap();
let (x0, y0, w, h) = (2500, 3000, 100, 100);
let mut img = ImageBuffer::new(w, h);
for (x, y, pixel) in tiff.pixels(x0, y0, w, h) {
    if let RasterValue::U16(v) = pixel {
        img.put_pixel(x - x0, y - y0, image::Luma([v]));
    }
}
img.save("dtm.png").unwrap();
```

## Running the examples

Download test data:

```
cd data
make
```

```
cargo run --example info data/tiff/N265E425.tif

cargo run --example pixel data/tiff/N265E425.tif 2550 3050

cargo run --example crop data/tiff/N265E425.tif 100x100+2500+3000 dtm.png

cargo run --example img2ascii data/tiff/sat.tif

cargo run --example http_dtm
```
