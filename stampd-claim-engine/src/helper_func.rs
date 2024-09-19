use mime_classifier::{ApacheBugFlag, LoadContext, MimeClassifier, NoSniffFlag};
use actix_multipart::form::tempfile::TempFile;

pub async fn mime_classifier(file_path: string) {
    // Create a classifier using default configuration
    let classifier = MimeClassifier::new();
    // Select the context, this is used to help the classifier based on where
    // the resource is loaded from. `Browsing` corresponds to simply typing
    // the URL in the address bar.
    let context = LoadContext::Browsing;
    // Flag indicating that sniffing should be avoided. This usually corresponds
    // to the server sending the header value `X-Content-Type-Options = "nosniff"`
    // but may also be applied automatically by the browser (e.g. `fetch` API)
    let no_sniff_flag = NoSniffFlag::Off;
    // Enable workaround for an Apache bug when server incorrectly sends a
    // `text/plain` or similar `Content-Type`.
    // See <https://mimesniff.spec.whatwg.org/#ref-for-check-for-apache-bug-flag>
    let apache_bug_flag = ApacheBugFlag::Off;
    // `Content-Type` set by the server, if any
    let supplied_type: Option<mime::Mime> = None;
    // Response body to classify
    let body: &[u8] = include_bytes!(file_path);

    let computed_type = classifier.classify(context, no_sniff_flag, apache_bug_flag, &supplied_type, body);
    assert_eq!(computed_type, mime::IMAGE_PNG);
}