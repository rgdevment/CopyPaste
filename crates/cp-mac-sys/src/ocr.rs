use objc2::AllocAnyThread;
use objc2_foundation::{NSArray, NSData, NSDictionary};
use objc2_vision::{
    VNImageRequestHandler, VNRecognizeTextRequest, VNRequest, VNRequestTextRecognitionLevel,
};

const LEVEL: VNRequestTextRecognitionLevel = VNRequestTextRecognitionLevel::Fast;

pub fn text_in_image(bytes: &[u8]) -> Option<Vec<String>> {
    let data = NSData::with_bytes(bytes);
    let options = NSDictionary::new();
    let handler = VNImageRequestHandler::initWithData_options(
        VNImageRequestHandler::alloc(),
        &data,
        &options,
    );

    let request = VNRecognizeTextRequest::new();
    request.setRecognitionLevel(LEVEL);
    request.setUsesLanguageCorrection(false);

    let requests: Vec<&VNRequest> = vec![&request];
    let array = NSArray::from_slice(&requests);
    let performed = handler.performRequests_error(&array);
    if performed.is_err() {
        return None;
    }

    let results = request.results()?;
    let mut lines = Vec::new();
    for observation in results.iter() {
        let candidates = observation.topCandidates(1);
        if let Some(best) = candidates.iter().next() {
            let text = best.string().to_string();
            if !text.trim().is_empty() {
                lines.push(text);
            }
        }
    }
    Some(lines)
}

pub fn searchable_text(bytes: &[u8]) -> Option<String> {
    let lines = text_in_image(bytes)?;
    (!lines.is_empty()).then(|| lines.join(" "))
}
