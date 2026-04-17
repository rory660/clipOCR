use crate::clipboard::ClipboardImage;
use crate::errors::ClipocrError;
use std::time::Duration;

pub struct OcrResult {
    pub text: String,
    pub engine: &'static str,
    pub elapsed: Duration,
}

pub trait OcrEngine {
    fn recognize(&self, img: &ClipboardImage) -> Result<OcrResult, ClipocrError>;
}

#[cfg(target_os = "linux")]
pub mod tesseract;
#[cfg(target_os = "macos")]
pub mod vision;

pub fn default_engine() -> Box<dyn OcrEngine> {
    #[cfg(target_os = "macos")]
    {
        Box::new(vision::VisionEngine)
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(tesseract::TesseractEngine)
    }
}
