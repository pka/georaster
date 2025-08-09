# georaster

[![crates.io version](https://img.shields.io/crates/v/georaster.svg)](https://crates.io/crates/georaster)
[![docs.rs docs](https://docs.rs/georaster/badge.svg)](https://docs.rs/georaster)

Rust library for accessing geospatial raster images.


## Usage examples

Read height pixel value from GeoTIFF:
```rust
let img_file = BufReader::new(File::open("N265E425.tif").unwrap());
let mut tiff = GeoTiffReader::open(img_file).unwrap();
let height = tiff.read_pixel(x, y).height();
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

## Development

* This project is easier to develop with [just](https://github.com/casey/just#readme), a modern alternative to `make`.
  Install it with `cargo install just`.
* To get a list of available commands, run `just`.
* To run tests, use `just test`.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)
  at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
