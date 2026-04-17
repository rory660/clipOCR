pub mod cli;
pub mod clipboard;
pub mod errors;
pub mod ocr;
pub mod output;

use crate::cli::Args;
use crate::errors::ClipocrError;
use crate::output::BannerOpts;
use std::io::Write;

pub fn run(args: Args) -> Result<i32, ClipocrError> {
    if args.verbose {
        let _ = tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .try_init();
    }

    let source = clipboard::default_source();
    let img = source.read_image()?;
    tracing::debug!(bytes = img.bytes.len(), "read clipboard image");

    let engine = ocr::default_engine();
    let result = engine.recognize(&img)?;
    tracing::debug!(
        chars = result.text.len(),
        elapsed_ms = result.elapsed.as_millis() as u64,
        "OCR done"
    );

    // Clipboard write-back.
    let mut copied = false;
    if !args.no_copy {
        match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(result.text.clone())) {
            Ok(()) => copied = true,
            Err(e) => {
                eprintln!("clipocr: warning: failed to write clipboard: {e}");
            }
        }
    }

    // Banner to stderr unless --plain.
    if !args.plain {
        let opts = BannerOpts {
            ascii: output::should_use_ascii(args.ascii),
            color: output::should_use_color(),
            copied,
        };
        let mut err = std::io::stderr().lock();
        let _ = output::render_banner(&mut err, &result, opts);
    }

    // Text output.
    if let Some(path) = &args.output {
        std::fs::write(path, &result.text)
            .map_err(|e| ClipocrError::Other(format!("failed to write {}: {e}", path.display())))?;
    } else {
        let mut out = std::io::stdout().lock();
        out.write_all(result.text.as_bytes())
            .map_err(|e| ClipocrError::Other(format!("failed to write stdout: {e}")))?;
        if !result.text.ends_with('\n') {
            let _ = out.write_all(b"\n");
        }
    }

    Ok(0)
}
