// Debug helper: OCR a file path directly, bypassing the clipboard.
//
//   cargo run --example ocr_file -- path/to/image.png

use clipocr::clipboard::{ClipboardImage, ImageFormatHint};
use clipocr::ocr::default_engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("usage: ocr_file <path>")?;
    let bytes = std::fs::read(&path)?;
    let fmt = match std::path::Path::new(&path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => ImageFormatHint::Png,
        "tif" | "tiff" => ImageFormatHint::Tiff,
        "bmp" => ImageFormatHint::Bmp,
        "jpg" | "jpeg" => ImageFormatHint::Jpeg,
        _ => ImageFormatHint::Unknown,
    };
    let img = ClipboardImage { bytes, format: fmt };
    let result = default_engine().recognize(&img)?;
    println!("engine={} ms={}", result.engine, result.elapsed.as_millis());
    println!("{}", result.text);
    Ok(())
}
