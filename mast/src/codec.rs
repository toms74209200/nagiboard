const LENGTH_BASES: [usize; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA_BITS: [u32; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCE_BASES: [usize; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA_BITS: [u32; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

struct Bits<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl Bits<'_> {
    fn read(&mut self, count: u32) -> Option<usize> {
        (0..count).try_fold(0, |value, i| {
            let byte = self.bytes.get(self.position / 8)?;
            let bit = usize::from((byte >> (self.position % 8)) & 1);
            self.position += 1;
            Some(value | bit << i)
        })
    }
}

struct Huffman {
    counts: [usize; 16],
    symbols: Vec<usize>,
}

impl Huffman {
    fn new(lengths: &[usize]) -> Option<Huffman> {
        let counts = lengths.iter().fold([0; 16], |mut counts, &length| {
            counts[length] += 1;
            counts
        });
        (1..16)
            .try_fold(1isize, |left, length| {
                let left = (left << 1) - counts[length] as isize;
                (left >= 0).then_some(left)
            })
            .map(|_| Huffman {
                counts,
                symbols: (1..16)
                    .flat_map(|length| {
                        (0..lengths.len()).filter(move |&symbol| lengths[symbol] == length)
                    })
                    .collect(),
            })
    }

    fn decode(&self, bits: &mut Bits) -> Option<usize> {
        (1..16)
            .try_fold((0, 0, 0), |(code, first, index), length| {
                let code = code | bits.read(1).ok_or(None)?;
                let count = self.counts[length];
                if code < first + count {
                    Err(self.symbols.get(index + code - first).copied())
                } else {
                    Ok(((code << 1), (first + count) << 1, index + count))
                }
            })
            .err()
            .flatten()
    }
}

pub fn decode(data: &str, max_len: usize) -> Option<String> {
    let sextets = data
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
    if sextets.len() % 4 == 1 {
        return None;
    }
    let bytes: Vec<u8> = sextets
        .chunks(4)
        .flat_map(|chunk| {
            let bits =
                chunk.iter().fold(0, |bits, sextet| bits << 6 | sextet) << (6 * (4 - chunk.len()));
            bits.to_be_bytes().into_iter().skip(1).take(chunk.len() - 1)
        })
        .collect();
    let mut bits = Bits {
        bytes: &bytes,
        position: 0,
    };
    let mut output = Vec::new();
    loop {
        let last = bits.read(1)? == 1;
        let tables = match bits.read(2)? {
            0 => {
                bits.position = bits.position.div_ceil(8) * 8;
                let length = bits.read(16)?;
                let complement = bits.read(16)?;
                let start = bits.position / 8;
                let block = bytes.get(start..start + length)?;
                if length != !complement & 0xffff || output.len() + length > max_len {
                    return None;
                }
                output.extend_from_slice(block);
                bits.position += length * 8;
                None
            }
            1 => Some((
                Huffman {
                    counts: [0, 0, 0, 0, 0, 0, 0, 24, 152, 112, 0, 0, 0, 0, 0, 0],
                    symbols: (256..280)
                        .chain(0..144)
                        .chain(280..288)
                        .chain(144..256)
                        .collect(),
                },
                Huffman {
                    counts: [0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    symbols: (0..32).collect(),
                },
            )),
            2 => {
                let literal_count = bits.read(5)? + 257;
                let distance_count = bits.read(5)? + 1;
                let code_length_count = bits.read(4)? + 4;
                if literal_count > 286 || distance_count > 30 {
                    return None;
                }
                let code_lengths = CODE_LENGTH_ORDER[..code_length_count].iter().try_fold(
                    [0; 19],
                    |mut code_lengths, &symbol| {
                        code_lengths[symbol] = bits.read(3)?;
                        Some(code_lengths)
                    },
                )?;
                let code_length = Huffman::new(&code_lengths)?;
                let total = literal_count + distance_count;
                let mut lengths: Vec<usize> = Vec::with_capacity(total);
                while lengths.len() < total {
                    let (length, repeat) = match code_length.decode(&mut bits)? {
                        length @ 0..=15 => (length, 1),
                        16 => (*lengths.last()?, 3 + bits.read(2)?),
                        17 => (0, 3 + bits.read(3)?),
                        _ => (0, 11 + bits.read(7)?),
                    };
                    lengths.extend(std::iter::repeat_n(length, repeat));
                }
                if lengths.len() != total || lengths[256] == 0 {
                    return None;
                }
                Some((
                    Huffman::new(&lengths[..literal_count])?,
                    Huffman::new(&lengths[literal_count..])?,
                ))
            }
            _ => return None,
        };
        if let Some((literal, distance)) = tables {
            loop {
                match literal.decode(&mut bits)? {
                    byte @ 0..=255 => {
                        if output.len() >= max_len {
                            return None;
                        }
                        output.push(byte as u8);
                    }
                    256 => break,
                    symbol @ 257..=285 => {
                        let index = symbol - 257;
                        let length = LENGTH_BASES[index] + bits.read(LENGTH_EXTRA_BITS[index])?;
                        let code = distance.decode(&mut bits)?;
                        let back =
                            DISTANCE_BASES.get(code)? + bits.read(DISTANCE_EXTRA_BITS[code])?;
                        if back > output.len() || output.len() + length > max_len {
                            return None;
                        }
                        let start = output.len() - back;
                        (start..start + length).for_each(|i| output.push(output[i]));
                    }
                    _ => return None,
                }
            }
        }
        if last {
            return String::from_utf8(output).ok();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn when_decode_with_data_of_dsl_then_returns_the_dsl() {
        assert_eq!(
            super::decode(
                "HYxLCoQwEAX3fYqHbnWRVsSVCHOA8Qoh9oiQD2Sic_3p-BZvU0W1kFti-ZaUwxkP3IbKWbzgnXfJ-Pj0I7JOMeqs0Wtel_pBcoMV49zxwORSCDbuSt2jbN46QaqRarGZOuZRUwb9og79AQ",
                usize::MAX
            )
            .as_deref(),
            Some(
                "# eventstorming v1\ntitle Order flow\n\nactor     a1   \"Customer\" @ 48,232\ncommand   c1   \"Place order\" @ 216,224\n\na1 -> c1\n"
            )
        );
    }

    #[test]
    fn when_decode_with_data_of_each_block_type_then_returns_the_text() {
        let cases = [
            ("stored", "AQwA8_9vcmRlciBwbGFjZWQ", "order placed"),
            ("fixed huffman", "S0xKToQhAA", "abcabcabcabc"),
            ("blocks before the last", "SkxKBgAAAP__S0lNAwA", "abcdef"),
        ];

        cases.iter().for_each(|(condition, data, text)| {
            assert_eq!(
                super::decode(data, usize::MAX).as_deref(),
                Some(*text),
                "{condition}"
            );
        });
    }

    #[test]
    fn when_decode_with_text_of_max_len_then_returns_the_text() {
        assert_eq!(
            super::decode("AQwA8_9vcmRlciBwbGFjZWQ", 12).as_deref(),
            Some("order placed")
        );
    }

    #[test]
    fn when_decode_with_text_over_max_len_then_returns_none() {
        let cases = [
            ("stored", "AQwA8_9vcmRlciBwbGFjZWQ", 11),
            ("fixed huffman literal", "S0xKBgA", 2),
            ("fixed huffman back-reference", "S0xKToQhAA", 11),
        ];

        cases.iter().for_each(|(condition, data, max_len)| {
            assert_eq!(super::decode(data, *max_len), None, "{condition}");
        });
    }

    #[test]
    fn when_decode_with_data_outside_base64url_then_returns_none() {
        let data = "HYxLCoQwEAX3fYqHbnWRVsSVCHOA8Qoh9oiQD2Sic_3p-BZvU0W1kFti-ZaUwxkP3IbKWbzgnXfJ-Pj0I7JOMeqs0Wtel_pBcoMV49zxwORSCDbuSt2jbN46QaqRarGZOuZRUwb9og79AQ";
        let cases = [
            (
                "standard alphabet",
                data.replace('-', "+").replace('_', "/"),
            ),
            ("padding", data.to_string() + "=="),
            (
                "length leaving a single character",
                data.to_string() + "AAA",
            ),
        ];

        cases.iter().for_each(|(condition, data)| {
            assert_eq!(super::decode(data, usize::MAX), None, "{condition}");
        });
    }

    #[test]
    fn when_decode_with_text_not_utf8_then_returns_none() {
        assert_eq!(super::decode("AQEA_v__", usize::MAX), None);
    }

    #[test]
    fn when_decode_with_malformed_stream_then_returns_none() {
        let cases = [
            ("empty", ""),
            ("reserved block type", "Bw"),
            (
                "stored length not matching its complement",
                "AQwAAABvcmRlciBwbGFjZWQ",
            ),
            (
                "stored block shorter than its length",
                "AQ0A8v9vcmRlciBwbGFjZWQ",
            ),
            ("distance beyond output", "AwIA"),
            ("reserved literal/length symbol", "GwM"),
            ("reserved distance code", "SwQ-"),
            ("dynamic header cut short", "BeAB"),
            ("literal/length count over 286", "9QCA5B8"),
            ("distance count over 30", "BR6A5B8"),
            ("over-subscribed code", "BQCSAA"),
            ("copy without previous length", "BQACJA"),
            ("code lengths overrunning the counts", "BQCA5P8f"),
            ("no end-of-block code", "BQCA5H8b"),
        ];

        cases.iter().for_each(|(condition, data)| {
            assert_eq!(super::decode(data, usize::MAX), None, "{condition}");
        });
    }
}
