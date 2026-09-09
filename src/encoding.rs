//! Percent-encoding for the few places the crate builds URLs and cookie
//! values by hand.

/// Encodes everything outside RFC 3986's unreserved set, so `;`, `,`,
/// `"`, `@` and spaces cannot break a cookie or a query string;
/// `decodeURIComponent` reverses it.
pub fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::percent_encode;

    #[test]
    fn reserved_characters_are_escaped() {
        assert_eq!(percent_encode("ada@example.com"), "ada%40example.com");
        assert_eq!(percent_encode("a b;c\"d\""), "a%20b%3Bc%22d%22");
        assert_eq!(percent_encode("safe-._~09Az"), "safe-._~09Az");
    }
}
