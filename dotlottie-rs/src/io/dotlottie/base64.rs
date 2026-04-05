const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(super) fn encode_into(input: &[u8], out: &mut String) {
    let output_len = input.len().div_ceil(3) * 4;
    out.reserve(output_len);

    let mut i = 0;
    while i + 2 < input.len() {
        let b0 = input[i] as u32;
        let b1 = input[i + 1] as u32;
        let b2 = input[i + 2] as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;

        out.push(BASE64_CHARS[((n >> 18) & 63) as usize] as char);
        out.push(BASE64_CHARS[((n >> 12) & 63) as usize] as char);
        out.push(BASE64_CHARS[((n >> 6) & 63) as usize] as char);
        out.push(BASE64_CHARS[(n & 63) as usize] as char);
        i += 3;
    }

    if i < input.len() {
        let b0 = input[i] as u32;
        let b1 = input.get(i + 1).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8);

        out.push(BASE64_CHARS[((n >> 18) & 63) as usize] as char);
        out.push(BASE64_CHARS[((n >> 12) & 63) as usize] as char);
        out.push(if i + 1 < input.len() {
            BASE64_CHARS[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push('=');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(input: &[u8]) -> String {
        let mut out = String::new();
        encode_into(input, &mut out);
        out
    }

    #[test]
    fn test_base64_encoding() {
        assert_eq!(encode(b"Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
        assert_eq!(encode(b""), "");
    }
}
