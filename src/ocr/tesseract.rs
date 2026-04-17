use crate::clipboard::ClipboardImage;
use crate::errors::ClipocrError;
use crate::ocr::{OcrEngine, OcrResult};
use std::time::Instant;

pub struct TesseractEngine;

impl OcrEngine for TesseractEngine {
    fn recognize(&self, img: &ClipboardImage) -> Result<OcrResult, ClipocrError> {
        let start = Instant::now();

        // Decode to RGBA and preprocess (grayscale + contrast).
        let dynimg = image::load_from_memory(&img.bytes)
            .map_err(|e| ClipocrError::Ocr(format!("failed to decode clipboard image: {e}")))?;
        let gray = dynimg.grayscale();

        // Write to a temp PNG (most reliable path for the tesseract crate).
        let tmp = std::env::temp_dir().join(format!("clipocr-{}.png", std::process::id()));
        gray.save(&tmp)
            .map_err(|e| ClipocrError::Ocr(format!("failed to write temp image: {e}")))?;

        let tmp_str = tmp
            .to_str()
            .ok_or_else(|| ClipocrError::Ocr("temp path is not valid UTF-8".into()))?;

        let text = (|| -> Result<String, String> {
            let t = tesseract::Tesseract::new(None, Some("eng"))
                .map_err(|e| format!("Tesseract init failed (install tesseract-ocr-eng?): {e}"))?;
            let t = t
                .set_image(tmp_str)
                .map_err(|e| format!("set_image failed: {e}"))?;
            let mut t = t;
            t.get_text().map_err(|e| format!("get_text failed: {e}"))
        })()
        .map_err(ClipocrError::Ocr);

        let _ = std::fs::remove_file(&tmp);

        let text = text?;
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            return Err(ClipocrError::Empty);
        }
        Ok(OcrResult {
            text: trimmed,
            engine: "Tesseract",
            elapsed: start.elapsed(),
        })
    }
}
