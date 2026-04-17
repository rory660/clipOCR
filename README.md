# clipocr

OCR the image on your clipboard. Print the text. Copy it back.

One command, no flags, fully local. macOS uses Apple Vision; Linux uses Tesseract.

```console
$ # screenshot something with Cmd+Shift+Ctrl+4
$ clipocr
┌─ clipocr ─────────────────────────────────┐
│  Engine:    Apple Vision                  │
│  Time:      405 ms                        │
│  Chars:     13                            │
│  Clipboard: copied ✓                      │
└───────────────────────────────────────────┘
Hello, world.
```

The recognized text goes to **stdout**; the banner goes to **stderr**, so
`clipocr > out.txt` and `clipocr --plain | pbcopy` both Just Work.

## Requirements

- **macOS 10.15+** (Catalina) — Vision text recognition requires 10.15.
- **Linux** — X11 or Wayland desktop, plus:
  - `tesseract` + English language data: `sudo apt install libtesseract-dev libleptonica-dev tesseract-ocr-eng`
  - `wl-clipboard` (Wayland): `sudo apt install wl-clipboard`
  - `xclip` (X11): `sudo apt install xclip`

## Install

For now, build from source:

```bash
git clone https://github.com/rory/clipocr
cd clipocr
cargo install --path .
```

This drops `clipocr` into `~/.cargo/bin/`.

### Distribution (planned)

- `cargo install clipocr` (once published to crates.io).
- Homebrew tap: `brew install rory/tap/clipocr`.
- Prebuilt binaries on GitHub Releases for `aarch64-apple-darwin`,
  `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`.

## Usage

```
clipocr [OPTIONS]

OPTIONS:
      --no-copy         Do not copy the recognized text back to the clipboard
      --plain           Disable the banner; print only the text
      --ascii           Use ASCII box-drawing chars (auto on non-UTF-8 locales)
  -o, --output <PATH>   Write the recognized text to a file instead of stdout
  -v, --verbose         Log timings and backend info to stderr
      --version
  -h, --help
```

### Exit codes

| Code | Meaning                                                              |
| ---- | -------------------------------------------------------------------- |
| 0    | Success                                                              |
| 1    | Unexpected internal error                                            |
| 2    | Clipboard does not contain an image                                  |
| 3    | Linux clipboard backend (`wl-paste` / `xclip`) missing               |
| 4    | OCR engine failure (e.g. missing `tesseract-ocr-eng`)                |
| 5    | OCR returned no text                                                 |

## Troubleshooting

**"clipboard does not contain an image"**
Copy a screenshot (Cmd+Shift+Ctrl+4 on macOS, GNOME Screenshot / Flameshot on
Linux), then run `clipocr` again.

**"wl-paste not found" / "xclip not found"**
Install via `sudo apt install wl-clipboard` or `sudo apt install xclip`.

**"Tesseract could not find English language data"**
Install via `sudo apt install tesseract-ocr-eng`.

## How it works

- macOS: reads `NSPasteboard` (PNG or TIFF), hands raw bytes to
  `VNRecognizeTextRequest` at `.accurate` level.
- Linux: shells out to `wl-paste` on Wayland or `xclip` on X11 to get
  `image/png` bytes, decodes with `image`, runs Tesseract via the `tesseract`
  crate.
- Text write-back goes through `arboard` on both platforms.

## License

MIT
