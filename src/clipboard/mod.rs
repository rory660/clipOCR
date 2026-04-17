use crate::errors::ClipocrError;

#[derive(Debug, Clone, Copy)]
pub enum ImageFormatHint {
    Png,
    Tiff,
    Bmp,
    Jpeg,
    Unknown,
}

pub struct ClipboardImage {
    pub bytes: Vec<u8>,
    pub format: ImageFormatHint,
}

pub trait ClipboardSource {
    fn read_image(&self) -> Result<ClipboardImage, ClipocrError>;
}

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;

pub fn default_source() -> Box<dyn ClipboardSource> {
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacClipboard)
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxClipboard::detect())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        compile_error!("clipocr supports macOS and Linux only");
    }
}
