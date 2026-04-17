use super::{ClipboardImage, ClipboardSource, ImageFormatHint};
use crate::errors::ClipocrError;
use std::process::{Command, Stdio};

pub enum LinuxClipboard {
    Wayland,
    X11,
}

impl LinuxClipboard {
    pub fn detect() -> Self {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            Self::Wayland
        } else {
            Self::X11
        }
    }
}

impl ClipboardSource for LinuxClipboard {
    fn read_image(&self) -> Result<ClipboardImage, ClipocrError> {
        match self {
            Self::Wayland => read_wayland(),
            Self::X11 => read_x11(),
        }
    }
}

fn which(bin: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {bin}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_bytes(mut cmd: Command) -> Result<Vec<u8>, ClipocrError> {
    let out = cmd
        .output()
        .map_err(|e| ClipocrError::Other(format!("failed to spawn clipboard backend: {e}")))?;
    if !out.status.success() {
        return Err(ClipocrError::NoImage);
    }
    Ok(out.stdout)
}

fn read_wayland() -> Result<ClipboardImage, ClipocrError> {
    if !which("wl-paste") {
        return Err(ClipocrError::BackendMissing {
            binary: "wl-paste",
            install_hint: "sudo apt install wl-clipboard",
        });
    }
    let types = {
        let mut c = Command::new("wl-paste");
        c.arg("--list-types");
        String::from_utf8_lossy(&run_bytes(c)?).to_string()
    };
    for (mime, fmt) in [
        ("image/png", ImageFormatHint::Png),
        ("image/tiff", ImageFormatHint::Tiff),
        ("image/bmp", ImageFormatHint::Bmp),
        ("image/jpeg", ImageFormatHint::Jpeg),
    ] {
        if types.lines().any(|l| l.trim() == mime) {
            let mut c = Command::new("wl-paste");
            c.arg("-t").arg(mime);
            let bytes = run_bytes(c)?;
            if !bytes.is_empty() {
                return Ok(ClipboardImage { bytes, format: fmt });
            }
        }
    }
    Err(ClipocrError::NoImage)
}

fn read_x11() -> Result<ClipboardImage, ClipocrError> {
    if !which("xclip") {
        return Err(ClipocrError::BackendMissing {
            binary: "xclip",
            install_hint: "sudo apt install xclip",
        });
    }
    let targets = {
        let mut c = Command::new("xclip");
        c.args(["-selection", "clipboard", "-t", "TARGETS", "-o"]);
        String::from_utf8_lossy(&run_bytes(c)?).to_string()
    };
    for (mime, fmt) in [
        ("image/png", ImageFormatHint::Png),
        ("image/tiff", ImageFormatHint::Tiff),
        ("image/bmp", ImageFormatHint::Bmp),
        ("image/jpeg", ImageFormatHint::Jpeg),
    ] {
        if targets.lines().any(|l| l.trim() == mime) {
            let mut c = Command::new("xclip");
            c.args(["-selection", "clipboard", "-t", mime, "-o"]);
            let bytes = run_bytes(c)?;
            if !bytes.is_empty() {
                return Ok(ClipboardImage { bytes, format: fmt });
            }
        }
    }
    Err(ClipocrError::NoImage)
}
