use georaster::geotiff::{GeoTiffReader, RasterValue};
use std::env;
use std::fs::File;
use std::io::{BufReader, Write};
use std::str::FromStr;

fn main() -> std::io::Result<()> {
    let src_fn = env::args().nth(1).expect("Input file required");
    let img_file = BufReader::new(File::open(src_fn).expect("Open input file"));
    let mut tiff = GeoTiffReader::open(img_file).expect("Open Tiff");

    // Read last image by default, which is smallest overview in a COG
    let img_no = env::args()
        .nth(2)
        .as_ref()
        .map(|s| usize::from_str(s).expect("Invalid number"))
        .unwrap_or(tiff.images().len() - 1);
    let (width, height) = tiff
        .images()
        .get(img_no)
        .expect("Get image dimensions")
        .dimensions
        .expect("Tiff dimensions");
    tiff.seek_to_image(img_no).unwrap();

    let mut ascii_img = vec![b' '; (width * height) as usize];
    for (x, y, pixel) in tiff.pixels(0, 0, width, height) {
        let ascii = match pixel {
            RasterValue::U8(v) => grey2ascii(v as f32 / u8::MAX as f32),
            RasterValue::U16(v) => grey2ascii(v as f32 / u16::MAX as f32),
            RasterValue::U32(v) => grey2ascii(v as f32 / u32::MAX as f32),
            RasterValue::U64(v) => grey2ascii(v as f32 / u64::MAX as f32),
            RasterValue::F32(v) => grey2ascii(v / f32::MAX),
            RasterValue::F64(v) => grey2ascii((v / f64::MAX) as f32),
            RasterValue::I8(v) => grey2ascii(v as f32 / i8::MAX as f32),
            RasterValue::I16(v) => grey2ascii(v as f32 / i16::MAX as f32),
            RasterValue::I32(v) => grey2ascii(v as f32 / i32::MAX as f32),
            RasterValue::I64(v) => grey2ascii(v as f32 / i64::MAX as f32),
            RasterValue::Rgb8(r, g, b) => rgb2ascii(r as u16, g as u16, b as u16),
            RasterValue::Rgb16(r, g, b) => rgb2ascii(r, g, b),
            RasterValue::Rgba8(r, g, b, _a) => rgb2ascii(r as u16, g as u16, b as u16),
            RasterValue::Rgba16(r, g, b, _a) => rgb2ascii(r, g, b),
            _ => ' ',
        };
        ascii_img[(y * width + x) as usize] = ascii as u8;
    }

    let mut out = std::io::stdout();
    for line in ascii_img.chunks(width as usize) {
        out.write_all(line)?;
        out.write_all(b"\n")?;
    }
    out.flush()
}

fn grey2ascii(luminance: f32) -> char {
    assert!((0.0..=1.0).contains(&luminance));
    let ascii_scale = " .:-=+░▒▓▓";
    //let ascii_scale = " .:-=+*#%@";
    let char_index = ((ascii_scale.chars().count() - 1) as f32 * luminance).round();
    ascii_scale.chars().nth(char_index as usize).unwrap()
}

fn rgb2ascii(r: u16, g: u16, b: u16) -> char {
    let avg = 255_f32 - (r + g + b) as f32 / 3f32;
    grey2ascii(avg / 255f32)
}
