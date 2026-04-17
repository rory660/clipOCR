use super::{ClipboardImage, ClipboardSource, ImageFormatHint};
use crate::errors::ClipocrError;

pub struct MacClipboard;

impl ClipboardSource for MacClipboard {
    fn read_image(&self) -> Result<ClipboardImage, ClipocrError> {
        read_pasteboard_image()
    }
}

fn read_pasteboard_image() -> Result<ClipboardImage, ClipocrError> {
    use objc2::rc::autoreleasepool;
    use objc2_app_kit::NSPasteboard;
    use objc2_foundation::{NSArray, NSString};

    autoreleasepool(|_| {
        let pb = NSPasteboard::generalPasteboard();

        // Try PNG first, then TIFF.
        let png_type = NSString::from_str("public.png");
        let tiff_type = NSString::from_str("public.tiff");

        for (ns_type, fmt) in [
            (&*png_type, ImageFormatHint::Png),
            (&*tiff_type, ImageFormatHint::Tiff),
        ] {
            let types = NSArray::from_slice(&[ns_type]);
            if let Some(_best) = pb.availableTypeFromArray(&types) {
                if let Some(data) = pb.dataForType(ns_type) {
                    let bytes = data.to_vec();
                    if !bytes.is_empty() {
                        return Ok(ClipboardImage { bytes, format: fmt });
                    }
                }
            }
        }

        Err(ClipocrError::NoImage)
    })
}
