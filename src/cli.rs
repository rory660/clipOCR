use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "clipocr",
    about = "OCR the image on your clipboard and copy the text back.",
    version
)]
pub struct Args {
    /// Do not copy the recognized text back to the clipboard.
    #[arg(long)]
    pub no_copy: bool,

    /// Disable fancy output; just print text to stdout.
    #[arg(long)]
    pub plain: bool,

    /// Use ASCII box-drawing chars in the banner (auto when non-UTF-8 locale).
    #[arg(long)]
    pub ascii: bool,

    /// Write recognized text to a file instead of stdout.
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Log timings and backend info to stderr.
    #[arg(short, long)]
    pub verbose: bool,
}
