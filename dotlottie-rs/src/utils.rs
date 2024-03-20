pub fn base64_encode(plain: &[u8]) -> String {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    plain
        .chunks(3)
        .flat_map(|chunk| {
            let (b1, b2, b3) =
                match *chunk {
                    [b1, b2, b3] => (b1, b2, b3),
                    [b1, b2] => (b1, b2, 0),
                    [b1] => (b1, 0, 0),
                    _ => (0, 0, 0),
                };
            [
                BASE64_CHARS[(b1 >> 2) as usize],
                BASE64_CHARS[((b1 & 0x03) << 4 | (b2 >> 4)) as usize],
                if chunk.len() > 1 {
                    BASE64_CHARS[((b2 & 0x0f) << 2 | (b3 >> 6)) as usize]
                } else {
                    b'='
                },
                if chunk.len() == 3 {
                    BASE64_CHARS[(b3 & 0x3f) as usize]
                } else {
                    b'='
                },
            ]
        })
        .map(|b| b as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::base64_encode;

    #[test]
    fn test_basic_encoding() {
        let inputs_and_expected =
            vec![
                (b"hello", "aGVsbG8="),
                (b"world", "d29ybGQ="),
                (b"rusty", "cnVzdHk="),
            ];

        for (input, expected) in inputs_and_expected {
            assert_eq!(base64_encode(input), expected);
        }
    }

    #[test]
    fn test_edge_cases() {
        // Test empty input
        assert_eq!(base64_encode(b""), "");

        // Test input length is a multiple of 3
        assert_eq!(base64_encode(b"abc"), "YWJj");

        // Test input length is not a multiple of 3
        assert_eq!(base64_encode(b"ab"), "YWI=");
        assert_eq!(base64_encode(b"a"), "YQ==");
    }

    #[test]
    fn test_special_characters() {
        // Include numbers, plus, and slash which are part of the base64 character set
        assert_eq!(base64_encode(b"123+"), "MTIzKw==");
        assert_eq!(base64_encode(b"/456"), "LzQ1Ng==");
    }

    #[test]
    fn test_large_input() {
        let large_input = vec![b'a'; 1000]; // 1000 'a's
                                            // This is a simplistic approach; a real test might validate the length or specific patterns in the output
        assert!(!base64_encode(&large_input).is_empty());
    }

    #[test]
    fn test_known_values() {
        // Using known base64 encoded strings to validate against
        assert_eq!(base64_encode(b"OpenAI"), "T3BlbkFJ");
        assert_eq!(base64_encode(b"ChatGPT"), "Q2hhdEdQVA==");
    }
}
