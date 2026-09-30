pub fn decode(text: &str) -> Option<Vec<u8>> {
    let sextets = text
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' => Some(u32::from(byte - b'A')),
            b'a'..=b'z' => Some(u32::from(byte - b'a') + 26),
            b'0'..=b'9' => Some(u32::from(byte - b'0') + 52),
            b'-' => Some(62),
            b'_' => Some(63),
            _ => None,
        })
        .collect::<Option<Vec<u32>>>()?;
    (sextets.len() % 4 != 1).then(|| {
        sextets
            .chunks(4)
            .flat_map(|chunk| {
                let bits = chunk.iter().fold(0, |bits, sextet| bits << 6 | sextet)
                    << (6 * (4 - chunk.len()));
                bits.to_be_bytes().into_iter().skip(1).take(chunk.len() - 1)
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn when_decode_with_test_vectors_of_rfc4648_then_returns_the_bytes() {
        let cases: [(&str, &[u8]); 8] = [
            ("", b""),
            ("Zg", b"f"),
            ("Zm8", b"fo"),
            ("Zm9v", b"foo"),
            ("Zm9vYg", b"foob"),
            ("Zm9vYmE", b"fooba"),
            ("Zm9vYmFy", b"foobar"),
            ("-_8", &[0xfb, 0xff]),
        ];

        cases.iter().for_each(|(text, bytes)| {
            assert_eq!(super::decode(text).as_deref(), Some(*bytes), "{text}");
        });
    }

    #[test]
    fn when_decode_with_text_outside_base64url_then_returns_none() {
        let text = random_string::generate_random_string(
            8,
            &[
                random_string::CharacterType::Lowercase,
                random_string::CharacterType::Uppercase,
                random_string::CharacterType::Numeric,
            ],
            "-_",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let cases = [
            ("standard alphabet +", text.clone() + "+A"),
            ("standard alphabet /", text.clone() + "/A"),
            ("padding", text.clone() + "AA=="),
            ("length leaving a single character", text.clone() + "A"),
        ];

        cases.iter().for_each(|(condition, text)| {
            assert_eq!(super::decode(text), None, "{condition}");
        });
    }
}
