use crate::clipboard::ClipboardImage;
use crate::errors::ClipocrError;
use crate::ocr::{OcrEngine, OcrResult};
use std::time::Instant;

pub struct VisionEngine;

impl OcrEngine for VisionEngine {
    fn recognize(&self, img: &ClipboardImage) -> Result<OcrResult, ClipocrError> {
        let start = Instant::now();
        let text = run_vision(&img.bytes)?;
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            return Err(ClipocrError::Empty);
        }
        Ok(OcrResult {
            text: trimmed,
            engine: "Apple Vision",
            elapsed: start.elapsed(),
        })
    }
}

fn run_vision(bytes: &[u8]) -> Result<String, ClipocrError> {
    use objc2::rc::autoreleasepool;
    use objc2::AnyThread;
    use objc2_foundation::{NSArray, NSData, NSDictionary, NSString};
    use objc2_vision::{
        VNImageRequestHandler, VNRecognizeTextRequest, VNRequest, VNRequestTextRecognitionLevel,
    };

    autoreleasepool(|_| {
        let data = NSData::with_bytes(bytes);
        let empty: objc2::rc::Retained<NSDictionary<NSString>> = NSDictionary::new();
        let handler = VNImageRequestHandler::initWithData_options(
            VNImageRequestHandler::alloc(),
            &data,
            &empty,
        );

        let request = VNRecognizeTextRequest::new();
        request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
        request.setUsesLanguageCorrection(true);
        let lang = NSString::from_str("en-US");
        let langs = NSArray::from_slice(&[&*lang]);
        request.setRecognitionLanguages(&langs);

        let req_ref: &VNRequest = request.as_ref();
        let reqs = NSArray::from_slice(&[req_ref]);
        handler
            .performRequests_error(&reqs)
            .map_err(|e| ClipocrError::Ocr(format!("Vision performRequests failed: {e}")))?;

        let Some(results) = request.results() else {
            return Ok(String::new());
        };

        let mut lines: Vec<String> = Vec::new();
        for obs in results.iter() {
            let candidates = obs.topCandidates(1);
            if candidates.count() == 0 {
                continue;
            }
            let top = candidates.objectAtIndex(0);
            let s = top.string();
            lines.push(s.to_string());
        }
        Ok(lines.join("\n"))
    })
}
