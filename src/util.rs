//! General utility functions.

/// Escape Pango/XML/HTML special characters in a string.
/// Converts &, <, > to their entity equivalents.
pub fn escape_pango(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Create a Waybar-compatible JSON error line.
pub fn error_json(text: &str, tooltip: &str) -> String {
    serde_json::json!({ "text": text, "tooltip": tooltip }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_pango() {
        assert_eq!(escape_pango("Hello & World"), "Hello &amp; World");
        assert_eq!(escape_pango("3 < 4 > 2"), "3 &lt; 4 &gt; 2");
        assert_eq!(escape_pango("a & b < c > d"), "a &amp; b &lt; c &gt; d");
        assert_eq!(escape_pango("no special"), "no special");
    }

    #[test]
    fn test_error_json() {
        let out = error_json("⛔️", "oops");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["text"], "⛔️");
        assert_eq!(v["tooltip"], "oops");
    }
}
