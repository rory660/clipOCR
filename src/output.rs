use crate::ocr::OcrResult;
use owo_colors::OwoColorize;
use std::io::Write;

#[derive(Debug, Clone, Copy)]
pub struct BannerOpts {
    pub ascii: bool,
    pub color: bool,
    pub copied: bool,
}

pub fn should_use_ascii(explicit: bool) -> bool {
    if explicit {
        return true;
    }
    let lang = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default()
        .to_ascii_uppercase();
    !lang.contains("UTF-8") && !lang.contains("UTF8")
}

pub fn should_use_color() -> bool {
    supports_color::on(supports_color::Stream::Stderr).is_some()
}

pub fn render_banner<W: Write>(w: &mut W, r: &OcrResult, opts: BannerOpts) -> std::io::Result<()> {
    let (tl, tr, bl, br, h, v) = if opts.ascii {
        ('+', '+', '+', '+', '-', '|')
    } else {
        ('┌', '┐', '└', '┘', '─', '│')
    };

    let lines = [
        format!("Engine:    {}", r.engine),
        format!("Time:      {} ms", r.elapsed.as_millis()),
        format!("Chars:     {}", r.text.chars().count()),
        format!(
            "Clipboard: {}",
            if opts.copied {
                "copied ✓"
            } else {
                "not copied"
            }
        ),
    ];

    let title = " clipocr ";
    let inner_w = lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(20)
        .max(40)
        + 2;

    // Full content width between corners is inner_w + 1 (matching bottom
    // border). The top border reserves 1 leading dash then the title, then
    // fills the remainder with dashes.
    let total_mid = inner_w + 1;
    let after_title = total_mid.saturating_sub(1 + title.chars().count());
    let top = format!(
        "{tl}{h}{title}{dashes}{tr}",
        dashes = std::iter::repeat(h).take(after_title).collect::<String>()
    );

    if opts.color {
        writeln!(w, "{}", top.dimmed())?;
    } else {
        writeln!(w, "{top}")?;
    }

    for l in &lines {
        let pad = inner_w.saturating_sub(l.chars().count() + 1);
        let line = format!("{v}  {l}{spaces}{v}", spaces = " ".repeat(pad));
        if opts.color {
            writeln!(w, "{}", line.dimmed())?;
        } else {
            writeln!(w, "{line}")?;
        }
    }

    let bottom = format!(
        "{bl}{dashes}{br}",
        dashes = std::iter::repeat(h).take(inner_w + 1).collect::<String>()
    );
    if opts.color {
        writeln!(w, "{}", bottom.dimmed())?;
    } else {
        writeln!(w, "{bottom}")?;
    }
    Ok(())
}
