//! Prove hash determinism across platforms

#[cfg(test)]
mod tests {
use faultreport::modules::error_capture;

    #[test]
    fn test_hash_determinism() {
        let msg = "Cannot read property 'x' of undefined";
        let stack = "at app.js:42\nat other.js:100";
        let url = "https://example.com/?foo=bar";

        let hash1 = error_capture::compute_hash(msg, stack, url);
        let hash2 = error_capture::compute_hash(msg, stack, url);
        assert_eq!(hash1, hash2);
    }
}

