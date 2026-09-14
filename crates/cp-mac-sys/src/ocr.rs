//! Leer el texto que hay dentro de una imagen.
//!
//! Es lo que convierte una captura de pantalla en algo que se puede
//! encontrar. La 2.x guarda las imágenes como un bloque opaco: si copiaste
//! una captura con un número de pedido, la única forma de recuperarla es
//! recordar cuándo fue y bajar por la lista.

use objc2::AllocAnyThread;
use objc2_foundation::{NSArray, NSData, NSDictionary};
use objc2_vision::{
    VNImageRequestHandler, VNRecognizeTextRequest, VNRequest, VNRequestTextRecognitionLevel,
};

/// Rápido frente a preciso. Para buscar en el historial interesa lo primero:
/// el usuario escribe un trozo de palabra, no espera una transcripción.
const LEVEL: VNRequestTextRecognitionLevel = VNRequestTextRecognitionLevel::Fast;

/// El texto reconocido en la imagen, línea a línea.
///
/// Devuelve `None` si Vision no puede con el formato, y una lista vacía si
/// la imagen simplemente no tiene texto. Son cosas distintas y quien llame
/// puede querer distinguirlas.
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
    // La corrección lingüística estorba aquí: convierte identificadores,
    // rutas y códigos en palabras del diccionario, que es justo lo que el
    // usuario quiere encontrar tal cual lo copió.
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

/// Todo el texto de la imagen en una sola cadena, listo para el índice.
pub fn searchable_text(bytes: &[u8]) -> Option<String> {
    let lines = text_in_image(bytes)?;
    (!lines.is_empty()).then(|| lines.join(" "))
}
