use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClipocrError {
    #[error("clipboard does not contain an image.\n  Copy a screenshot (Cmd+Shift+Ctrl+4 on macOS, GNOME Screenshot on Linux), then run `clipocr` again.")]
    NoImage,

    #[error("{binary} not found.\n  Install it with: {install_hint}")]
    BackendMissing {
        binary: &'static str,
        install_hint: &'static str,
    },

    #[error("OCR engine failure: {0}")]
    Ocr(String),

    #[error("OCR returned no text")]
    Empty,

    #[error("{0}")]
    Other(String),
}

impl ClipocrError {
    pub fn exit_code(&self) -> i32 {
        match self {
            ClipocrError::NoImage => 2,
            ClipocrError::BackendMissing { .. } => 3,
            ClipocrError::Ocr(_) => 4,
            ClipocrError::Empty => 5,
            ClipocrError::Other(_) => 1,
        }
    }
}
