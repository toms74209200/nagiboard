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
        let counts = lengths.iter().try_fold([0; 16], |mut counts, &length| {
            *counts.get_mut(length)? += 1;
            Some(counts)
        })?;
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

pub fn inflate(bytes: &[u8], max_len: usize) -> Option<Vec<u8>> {
    let mut bits = Bits { bytes, position: 0 };
    let mut output = Vec::new();
    loop {
        let last = bits.read(1)? == 1;
        match bits.read(2)? {
            0 => stored(&mut bits, &mut output, max_len)?,
            1 => codes(
                &mut bits,
                &mut output,
                max_len,
                &Huffman::new(
                    &(0..288)
                        .map(|symbol| match symbol {
                            0..=143 => 8,
                            144..=255 => 9,
                            256..=279 => 7,
                            _ => 8,
                        })
                        .collect::<Vec<usize>>(),
                )?,
                &Huffman::new(&[5; 30])?,
            )?,
            2 => {
                let (literal, distance) = dynamic_tables(&mut bits)?;
                codes(&mut bits, &mut output, max_len, &literal, &distance)?
            }
            _ => return None,
        }
        if last {
            return Some(output);
        }
    }
}

fn stored(bits: &mut Bits, output: &mut Vec<u8>, max_len: usize) -> Option<()> {
    bits.position = bits.position.div_ceil(8) * 8;
    let length = bits.read(16)?;
    let complement = bits.read(16)?;
    let start = bits.position / 8;
    let block = bits.bytes.get(start..start + length)?;
    if length != !complement & 0xffff || output.len() + length > max_len {
        return None;
    }
    output.extend_from_slice(block);
    bits.position += length * 8;
    Some(())
}

fn codes(
    bits: &mut Bits,
    output: &mut Vec<u8>,
    max_len: usize,
    literal: &Huffman,
    distance: &Huffman,
) -> Option<()> {
    loop {
        match literal.decode(bits)? {
            256 => return Some(()),
            symbol @ 0..=255 if output.len() < max_len => output.push(symbol as u8),
            symbol @ 257.. => {
                let index = symbol - 257;
                let length = LENGTH_BASES.get(index)? + bits.read(LENGTH_EXTRA_BITS[index])?;
                let code = distance.decode(bits)?;
                let back = DISTANCE_BASES.get(code)? + bits.read(DISTANCE_EXTRA_BITS[code])?;
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

fn dynamic_tables(bits: &mut Bits) -> Option<(Huffman, Huffman)> {
    let literal_count = bits.read(5)? + 257;
    let distance_count = bits.read(5)? + 1;
    let code_length_count = bits.read(4)? + 4;
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
        let (length, repeat) = match code_length.decode(bits)? {
            length @ 0..=15 => (length, 1),
            16 => (*lengths.last()?, 3 + bits.read(2)?),
            17 => (0, 3 + bits.read(3)?),
            18 => (0, 11 + bits.read(7)?),
            _ => return None,
        };
        lengths.extend(std::iter::repeat_n(length, repeat));
    }
    if lengths.len() != total || literal_count > 286 || distance_count > 30 || lengths[256] == 0 {
        return None;
    }
    Some((
        Huffman::new(&lengths[..literal_count])?,
        Huffman::new(&lengths[literal_count..])?,
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn when_inflate_with_stored_block_then_returns_its_content() {
        let content = random_string::generate_random_string(
            16,
            &[
                random_string::CharacterType::Lowercase,
                random_string::CharacterType::Uppercase,
                random_string::CharacterType::Numeric,
            ],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let block = [&[0x01, 0x10, 0x00, 0xef, 0xff][..], content.as_bytes()].concat();

        assert_eq!(
            super::inflate(&block, usize::MAX),
            Some(content.into_bytes())
        );
    }

    #[test]
    fn when_inflate_with_fixed_huffman_block_then_returns_the_decompressed_bytes() {
        assert_eq!(
            super::inflate(&[75, 76, 74, 78, 132, 33, 0], usize::MAX),
            Some(b"abcabcabcabc".to_vec())
        );
    }

    #[test]
    fn when_inflate_with_dynamic_huffman_block_then_returns_the_decompressed_bytes() {
        let compressed = [
            77, 142, 73, 14, 195, 32, 12, 0, 239, 126, 133, 149, 92, 91, 9, 104, 207, 85, 191, 130,
            140, 155, 34, 177, 137, 160, 244, 251, 97, 17, 106, 110, 22, 51, 120, 188, 34, 31, 28,
            202, 94, 98, 246, 54, 108, 120, 72, 40, 182, 56, 198, 152, 13, 103, 252, 184, 248, 3,
            232, 10, 178, 196, 101, 188, 38, 167, 137, 205, 130, 111, 20, 55, 49, 169, 154, 116,
            255, 218, 148, 6, 86, 226, 34, 60, 166, 64, 58, 16, 59, 55, 148, 103, 87, 40, 122, 175,
            131, 65, 170, 141, 190, 125, 244, 71, 66, 138, 139, 80, 51, 45, 240, 231, 173, 209, 12,
            168, 127, 239, 175, 122, 37, 84, 167, 13, 10, 78,
        ];

        assert_eq!(
            super::inflate(&compressed, usize::MAX),
            Some(b"# eventstorming v1\ntitle order flow\n\nevent e1 \"order placed\" @ 0,0\nevent e2 \"order shipped\" @ 200,0\nevent e3 \"order cancelled\" @ 400,0\ncommand c1 \"place order\" @ 0,100\ncommand c2 \"ship order\" @ 200,100\n\nc1 -> e1\nc2 -> e2\n".to_vec())
        );
    }

    #[test]
    fn when_inflate_with_output_of_max_len_then_returns_the_bytes() {
        let content = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let block = [&[0x01, 0x10, 0x00, 0xef, 0xff][..], content.as_bytes()].concat();

        assert_eq!(super::inflate(&block, 16), Some(content.into_bytes()));
    }

    #[test]
    fn when_inflate_with_output_over_max_len_then_returns_none() {
        let content = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let cases = [
            (
                "stored",
                [&[0x01, 0x10, 0x00, 0xef, 0xff][..], content.as_bytes()].concat(),
                15,
            ),
            ("fixed huffman", vec![75, 76, 74, 78, 132, 33, 0], 11),
        ];

        cases.iter().for_each(|(condition, block, max_len)| {
            assert_eq!(super::inflate(block, *max_len), None, "{condition}");
        });
    }

    #[test]
    fn when_inflate_with_malformed_stream_then_returns_none() {
        let content = random_string::generate_random_string(
            16,
            &[random_string::CharacterType::Lowercase],
            "",
            &mut std::fs::File::open("/dev/urandom").unwrap(),
        );
        let cases = [
            ("empty", vec![]),
            ("reserved block type", vec![0x07]),
            (
                "stored length not matching its complement",
                [&[0x01, 0x10, 0x00, 0x00, 0x00][..], content.as_bytes()].concat(),
            ),
            (
                "stored block shorter than its length",
                [&[0x01, 0x11, 0x00, 0xee, 0xff][..], content.as_bytes()].concat(),
            ),
            ("distance beyond output", vec![0x03, 0x02, 0x00]),
        ];

        cases.iter().for_each(|(condition, stream)| {
            assert_eq!(super::inflate(stream, usize::MAX), None, "{condition}");
        });
    }
}
