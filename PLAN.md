# clipocr Implementation Plan

A cross-platform (macOS + Linux) CLI written in **Rust** that reads an image
from the system clipboard, runs local OCR, prints the recognized text, and
copies it back to the clipboard.

- Binary: `clipocr`
- Crate: `clipocr`
- License: MIT
- Repo: `github.com/rory/clipocr` (scaffolded at `~/Git/clipocr`)
- Platforms: macOS 10.15+ (Catalina), Linux (X11 + Wayland)

---

## 1. Goals & Non-Goals

### Goals
- One command: `clipocr`. No prompts, no flags needed for the happy path.
- Reads clipboard → OCRs → prints text to stdout → copies text back to clipboard.
- Fully local. No network.
- macOS: **Apple Vision** (`VNRecognizeTextRequest`) — no extra install.
- Linux: **Tesseract** (linked via `tesseract` crate) — X11 and Wayland.
- Single static-ish binary shippable via `cargo install`, Homebrew, or GitHub releases.
- < 3s latency on a typical UI screenshot.

### Non-Goals (v1)
- Windows.
- Languages other than English.
- GUI, daemon/watch mode, region-select screenshot.
- PDF / multi-page.
- Translation, LLM post-processing.

---

## 2. Tech Stack

| Concern              | Choice                                                | Notes                                          |
| -------------------- | ----------------------------------------------------- | ---------------------------------------------- |
| Language             | Rust (edition 2021, MSRV 1.75)                        |                                                |
| CLI framework        | `clap` (derive)                                       | Standard                                       |
| macOS OCR            | `objc2` + `objc2-vision` + `objc2-foundation`         | `VNRecognizeTextRequest` directly, no FFI shim |
| macOS clipboard img  | `objc2-app-kit` → `NSPasteboard.dataForType:NSPasteboardTypePNG` / `TIFF` | Raw bytes → decode via `image` crate |
| Linux OCR            | `tesseract` crate (libtesseract bindings)             | Links `libtesseract` + `libleptonica`          |
| Linux clipboard      | `wl-clipboard` (`wl-paste`) on Wayland; `xclip` on X11 | Shell out — most reliable across distros       |
| Image decode         | `image` crate                                         | PNG/TIFF → `DynamicImage`                      |
| Clipboard write-back | `arboard`                                             | Cross-platform text-to-clipboard, pure Rust    |
| Pretty output        | `owo-colors` (ANSI) + `supports-color`                | For the "copied ✓" banner on stderr            |
| Errors               | `anyhow` at the boundary, `thiserror` for typed errors |                                               |
| Logging              | `tracing` + `tracing-subscriber` (behind `-v`)        |                                                |
| Tests                | built-in + `assert_cmd` + `insta` for snapshot output |                                                |
| Lint / format        | `clippy -D warnings`, `rustfmt`                       |                                                |
| CI                   | GitHub Actions: macOS + Ubuntu (X11 + Wayland jobs)   |                                                |

### Why shell out to `wl-paste`/`xclip` on Linux but use native APIs on macOS?
Apple Vision is a dramatic accuracy win worth the `objc2` complexity. On Linux,
`arboard`'s image support is flaky across compositors; `wl-paste` / `xclip`
Just Work and are installed by default on most desktops. We shell out once, get
PNG bytes on stdout, decode with the `image` crate. Zero compositor-specific code.

---

## 3. Repository Layout

```
clipocr/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── PLAN.md
├── LICENSE                       # MIT
├── rust-toolchain.toml
├── .github/workflows/ci.yml
├── src/
│   ├── main.rs                   # thin: parse args → call run()
│   ├── lib.rs                    # pub fn run(Args) -> Result<Output>
│   ├── cli.rs                    # clap derive struct
│   ├── clipboard/
│   │   ├── mod.rs                # trait ClipboardImage + cfg-gated dispatch
│   │   ├── macos.rs              # NSPasteboard → Vec<u8>
│   │   ├── linux.rs              # detect Wayland vs X11 → wl-paste or xclip
│   │   └── error.rs
│   ├── ocr/
│   │   ├── mod.rs                # trait OcrEngine + dispatch
│   │   ├── vision.rs             # macOS, cfg(target_os="macos")
│   │   └── tesseract.rs          # Linux, cfg(target_os="linux")
│   ├── output.rs                 # fancy display (box, copied ✓)
│   └── errors.rs
├── tests/
│   ├── fixtures/
│   │   ├── hello.png
│   │   └── ui_screenshot.png
│   ├── cli.rs                    # assert_cmd smoke tests
│   └── snapshots/                # insta
└── examples/
    └── ocr_file.rs               # debug helper: OCR a file path
```

---

## 4. CLI Surface

```
clipocr [OPTIONS]

OPTIONS:
  --no-copy         Do not copy the recognized text back to the clipboard
  --plain           Disable fancy output (just print text to stdout)
      --ascii       Use ASCII box-drawing chars in the banner (auto when non-UTF-8 locale)
  -o, --output <PATH>   Write text to file instead of stdout
  -v, --verbose     Log timings and backend info to stderr
      --version
  -h, --help
```

### Default flow (no flags)
1. `clipocr`
2. Read clipboard image → fail with clear error if not an image.
3. Run OCR (Vision on macOS, Tesseract on Linux).
4. Print fancy block to **stderr**:
   ```
   ┌─ clipocr ────────────────────────────────────────┐
   │  Engine:   Apple Vision                           │
   │  Time:     412 ms                                 │
   │  Chars:    287                                    │
   │  Clipboard: copied ✓                              │
   └───────────────────────────────────────────────────┘
   ```
5. Print raw recognized text to **stdout**.
6. Copy recognized text back to clipboard (overwrites the image).
7. Exit 0.

Separating fancy UI (stderr) from text (stdout) keeps `clipocr | pbcopy` and
`clipocr > out.txt` working cleanly, and `--plain` suppresses the banner for
scripts that don't want ANSI on stderr either.

### Exit codes
- `0` success
- `2` clipboard has no image (`clipocr: error: clipboard does not contain an image. Copy a screenshot or image, then run clipocr again.`)
- `3` clipboard backend unavailable on Linux (missing `wl-paste`/`xclip`; include `apt install wl-clipboard` hint)
- `4` OCR engine failure (Vision threw / Tesseract not linked / language data missing)
- `5` OCR returned empty text
- `1` unexpected / internal error

---

## 5. Component Design

### 5.1 Clipboard trait (`src/clipboard/mod.rs`)

```rust
pub struct ClipboardImage {
    pub bytes: Vec<u8>,      // raw PNG or TIFF
    pub format: ImageFormat, // hint for the decoder
}

pub trait ClipboardSource {
    fn read_image(&self) -> Result<ClipboardImage, ClipboardError>;
}

pub fn default_source() -> Box<dyn ClipboardSource> {
    #[cfg(target_os = "macos")] { Box::new(macos::MacClipboard) }
    #[cfg(target_os = "linux")] { Box::new(linux::LinuxClipboard::detect()) }
}
```

#### macOS (`clipboard/macos.rs`)
- Use `objc2-app-kit::NSPasteboard::generalPasteboard()`.
- Try `NSPasteboardTypePNG` first, fall back to `NSPasteboardTypeTIFF`.
- Return raw bytes; decoding happens later (or pass NSImage directly to Vision — see §5.3).
- **Optimization**: on macOS we can skip decode entirely and hand the raw
  `NSData` / `CGImage` to Vision. Keep the `Vec<u8>` path for tests but prefer
  zero-copy `CGImage` in the hot path.

#### Linux (`clipboard/linux.rs`)
- Detect: `std::env::var("WAYLAND_DISPLAY").is_ok()` → Wayland else X11.
- Wayland:
  - `wl-paste --list-types` → must contain `image/png` (or `image/*`).
  - `wl-paste -t image/png` → stdout bytes.
- X11:
  - `xclip -selection clipboard -t TARGETS -o` → must contain `image/png`.
  - `xclip -selection clipboard -t image/png -o` → stdout bytes.
- Fall back: if `image/png` not offered but `image/tiff` / `image/bmp` is, try those.
- If the binary is missing: return `ClipboardError::BackendMissing { binary, install_hint }`.

### 5.2 OCR trait (`src/ocr/mod.rs`)

```rust
pub struct OcrResult { pub text: String, pub engine: &'static str, pub elapsed: Duration }

pub trait OcrEngine {
    fn recognize(&self, img: &ClipboardImage) -> Result<OcrResult, OcrError>;
}
```

### 5.3 macOS Vision (`ocr/vision.rs`)

Using `objc2-vision`:
1. Build a `CGImage` from the `NSData` (PNG/TIFF) via `CGImageSourceCreateWithData`.
2. Create `VNImageRequestHandler` with that `CGImage`.
3. Create `VNRecognizeTextRequest`:
   - `recognitionLevel = .accurate`
   - `usesLanguageCorrection = true`
   - `recognitionLanguages = ["en-US"]`
4. `perform([request])` synchronously (we're already off the main thread — CLI is single-threaded).
5. Collect `results` → each `VNRecognizedTextObservation` → `.topCandidates(1).first.string`.
6. Join observations with `\n`. Vision returns them roughly in reading order; for v1 trust it.

Edge cases:
- Vision can throw on unsupported image formats → decode via `image` crate to
  RGBA8 and build a `CGImage` from the raw buffer as a fallback.
- Empty results → `OcrError::Empty`.

### 5.4 Linux Tesseract (`ocr/tesseract.rs`)

Using the `tesseract` crate (v0.15+):
1. Decode `ClipboardImage.bytes` via `image::load_from_memory`.
2. Optional preprocess (grayscale + autocontrast) — always on for Linux since
   Tesseract benefits more than Vision does.
3. Write to a temp `.png` (or pass bytes via Leptonica's `pixReadMem` depending
   on crate API) — the `tesseract` crate accepts file paths most reliably.
4. `Tesseract::new(None, Some("eng"))?.set_image(path)?.get_text()?`.
5. Return trimmed text.

Build-time requirement: `libtesseract-dev` + `libleptonica-dev` + English
traineddata (`tesseract-ocr-eng`). Document in README.

---

## 6. Output Layer (`src/output.rs`)

- Default: print banner to stderr (box-drawing chars, colored with `owo-colors`
  if `supports-color::on(Stream::Stderr)`), then print `result.text` to stdout.
- `--plain`: stdout text only, no stderr banner, no colors.
- `--output PATH`: write text to file; banner still goes to stderr unless `--plain`.
- Clipboard write-back via `arboard::Clipboard::new()?.set_text(&text)?`.
  - If clipboard write fails, log a warning on stderr but still exit 0 — the
    user has the text on stdout.

---

## 7. Error Messages (examples)

```
clipocr: error: clipboard does not contain an image.
  Copy a screenshot (Cmd+Shift+Ctrl+4 on macOS, GNOME Screenshot on Linux),
  then run `clipocr` again.

clipocr: error: wl-paste not found.
  Install it with: sudo apt install wl-clipboard

clipocr: error: Tesseract could not find English language data.
  Install it with: sudo apt install tesseract-ocr-eng
```

All errors to stderr, prefixed `clipocr: error:`, with an install/recovery hint
when we can give one.

---

## 8. Testing Strategy

### Unit
- `clipboard::linux`: mock `Command` via a trait; assert we invoke the right
  args and handle missing-binary / non-image cases.
- `ocr::tesseract`: OCR `fixtures/hello.png` ("Hello, world."), strip-equal assert.
  Skip if `TESSDATA_PREFIX` unset / lib not linked.
- `ocr::vision`: macOS-only `#[cfg]` test, OCR the same fixture.
- `output`: snapshot the banner with `insta` in color-off mode.

### Integration (GitHub Actions matrix)
- **macOS runner**: `cargo test` + a smoke test that loads `fixtures/hello.png`
  directly into Vision (bypassing clipboard) and asserts text.
- **Ubuntu X11** (`xvfb-run`): `xclip -selection clipboard -t image/png -i fixtures/hello.png`,
  then `cargo run -- --plain`, assert stdout.
- **Ubuntu Wayland** (`sway` headless or `cage` + `wl-clipboard`):
  `wl-copy < fixtures/hello.png --type image/png`, then `clipocr --plain`, assert stdout.

### Manual QA
- [ ] Cmd+Shift+Ctrl+4 on macOS → `clipocr` → text printed, copied back.
- [ ] GNOME screenshot (Wayland) → `clipocr` → text.
- [ ] Flameshot on X11 → `clipocr` → text.
- [ ] Copy **text** (not image) → exit 2 with helpful message.
- [ ] Empty clipboard → exit 2.
- [ ] `wl-clipboard` uninstalled → exit 3 with install hint.
- [ ] Piping: `clipocr --plain | wc -w` works (no banner leaks into stdout).

Coverage target: 80%+ on non-FFI code. `objc2-vision` call sites stay thin.

---

## 9. Build & Distribution

### For now (per user ask)
- `cargo build --release` produces `./target/release/clipocr`.
- README documents manual install: `cargo install --path .` or copy the binary to `~/.local/bin/`.
- README has a **Distribution (planned)** section listing the three future channels below.

### Planned (post-v1)
1. **crates.io**: `cargo install clipocr` — works once the crate is published.
2. **Homebrew tap** (`rory/tap/clipocr`): formula declaring `depends_on "tesseract"`
   on Linux-only; macOS pulls from a prebuilt bottle.
3. **GitHub Releases**: prebuilt binaries for
   - `aarch64-apple-darwin`
   - `x86_64-apple-darwin`
   - `x86_64-unknown-linux-gnu`
   - `aarch64-unknown-linux-gnu`
   built by a release workflow, attached to a tag.

Cross-compilation notes for later: macOS targets build on macOS runners; Linux
targets need `libtesseract` available for the linker — simplest is to build on
the matching Linux runner rather than cross-compile.

---

## 10. Milestones

| M  | Deliverable                                                         | Est.  |
| -- | ------------------------------------------------------------------- | ----- |
| 1  | Cargo scaffold, `clap` CLI, `--version`, CI skeleton                | 0.5d  |
| 2  | macOS clipboard → bytes (`NSPasteboard`)                            | 0.5d  |
| 3  | macOS Vision OCR happy path on `fixtures/hello.png`                 | 1d    |
| 4  | End-to-end macOS: copy screenshot → `clipocr` → text on stdout      | 0.5d  |
| 5  | Linux clipboard (Wayland + X11 via `wl-paste`/`xclip`)              | 1d    |
| 6  | Linux Tesseract OCR via `tesseract` crate                           | 1d    |
| 7  | Output layer: fancy banner, clipboard write-back, `--plain`         | 0.5d  |
| 8  | Error UX, exit codes, install hints                                 | 0.5d  |
| 9  | Test matrix in CI (macOS + Ubuntu X11 + Ubuntu Wayland)             | 1d    |
| 10 | README (install, usage, troubleshooting, distribution roadmap)       | 0.5d  |

**Total: ~7 engineer-days for a shippable v1.**

First locally-runnable binary: end of M4 (~2.5d in).

---

## 11. Stretch Goals (v2+)

- `clipocr watch` — daemon that fires on every new clipboard image.
- `clipocr snip` — invoke `screencapture -i -c` (macOS) / `grim -g "$(slurp)"`
  (Wayland) / `maim -s` (X11), then OCR.
- `--json` with bounding boxes (`VNRecognizedTextObservation.boundingBox` on
  macOS, `tesseract::image_to_data` on Linux).
- `--lang` flag + language pack detection.
- Windows support via `Windows.Media.Ocr` (`windows` crate).
- Apple Vision on iOS/visionOS — out of scope but the OCR crate is portable.

---

## 12. Resolved Decisions

1. **Repo/crate rename**: Done — directory is now `~/Git/clipocr`. Crate and
   binary are `clipocr`.
2. **macOS minimum version**: Hard-require macOS 10.15+ (Vision text
   recognition requirement). Surface in:
   - `Cargo.toml` metadata (`[package.metadata]` note + `README` badge).
   - README "Requirements" section.
   - Runtime: if Vision init fails on older macOS, emit a clear error pointing
     at the version requirement.
3. **Banner charset**: Add `--ascii` flag to `§4 CLI Surface`. Default to
   Unicode box-drawing chars; `--ascii` swaps to `+`, `-`, `|`. Also auto-fall
   back to ASCII when `LANG`/`LC_ALL` don't indicate UTF-8.
