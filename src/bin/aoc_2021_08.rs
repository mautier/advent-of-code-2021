/// A wire pattern is a set of active (on) wires, as a bitset with 7 bits (1 per wire).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct WirePattern(u8);

impl WirePattern {
    fn num_on(&self) -> usize {
        self.0.count_ones() as usize
    }
    fn intersect(&self, other: &Self) -> Self {
        Self(self.0 & other.0)
    }
    fn symmetric_diff(&self, other: &Self) -> Self {
        Self(self.0 ^ other.0)
    }
}

/// A single line from the input: 10 unique wire patterns (1 per digit), and 4 patterns that we
/// actually want to decode.
struct NoteEntry {
    unique_patterns: [WirePattern; 10],
    output_digits: [WirePattern; 4],
}

/// An array of {index i => wire pattern for digit i}
struct DecodedWirePatterns {
    digit_to_pattern: [WirePattern; 10],
}

impl DecodedWirePatterns {
    fn decode(&self, pat: WirePattern) -> Option<u8> {
        self.digit_to_pattern
            .iter()
            .copied()
            .enumerate()
            .find(|&(_digit, digit_pat)| pat == digit_pat)
            .map(|(digit, _digit_pat)| digit as u8)
    }
}

fn decode_wire_to_segment_mapping(unique_patterns: &[WirePattern; 10]) -> DecodedWirePatterns {
    // Helper for looking for a single pattern matching a predicate.
    // Panics when the predicate actually matches 0 or 2+ patterns.
    let find_one_pattern = |pred: &dyn Fn(&WirePattern) -> bool| -> WirePattern {
        let mut it = unique_patterns.iter().copied().filter(pred);
        let result = it.next().unwrap();
        assert_eq!(it.next(), None);
        result
    };

    // Look for the digits 1, 4, 7, 8, we can identify them from the number of on wires.
    let digit_1 = find_one_pattern(&|pat: &WirePattern| pat.num_on() == 2);
    let digit_4 = find_one_pattern(&|pat: &WirePattern| pat.num_on() == 4);
    let digit_7 = find_one_pattern(&|pat: &WirePattern| pat.num_on() == 3);
    let digit_8 = find_one_pattern(&|pat: &WirePattern| pat.num_on() == 7);

    // Digits 0, 6, 9 have 6 wires. Among them however, only 6 has an intersection with digit 1 of
    // size 1 wire.
    let digit_6 =
        find_one_pattern(&|pat| pat.num_on() == 6 && digit_1.intersect(pat).num_on() == 1);
    // Digit 5 is the only digit with 5 wires (among 2,3,5) that has 1 wire of difference with 6.
    let digit_5 =
        find_one_pattern(&|pat| pat.num_on() == 5 && digit_6.symmetric_diff(pat).num_on() == 1);

    // Looking at digits with 6 wires: 0,6,9
    // 0 has 3 of difference with 5.
    // 9 has 1 of difference with 5.
    let digit_0 =
        find_one_pattern(&|pat| pat.num_on() == 6 && digit_5.symmetric_diff(pat).num_on() == 3);
    let digit_9 = find_one_pattern(&|pat| {
        pat.num_on() == 6 && digit_5.symmetric_diff(pat).num_on() == 1 && *pat != digit_6
    });

    // Digit 3 has 1 of difference with 9
    let digit_3 = find_one_pattern(&|pat| {
        pat.num_on() == 5 && digit_9.symmetric_diff(pat).num_on() == 1 && *pat != digit_5
    });
    let digit_2 =
        find_one_pattern(&|pat| pat.num_on() == 5 && digit_9.symmetric_diff(pat).num_on() == 3);

    DecodedWirePatterns {
        digit_to_pattern: [
            digit_0, digit_1, digit_2, digit_3, digit_4, digit_5, digit_6, digit_7, digit_8,
            digit_9,
        ],
    }
}

fn parse_note_entry(line: &str) -> NoteEntry {
    let mut parts = line.split(' ');

    let mut unique_patterns = [WirePattern(0); 10];
    for upat in &mut unique_patterns {
        let txt = parts.next().unwrap();
        assert_ne!(txt, "|");
        *upat = txt
            .bytes()
            .map(|b| 1 << (b - b'a'))
            .fold(WirePattern(0), |acc, val: u8| WirePattern(acc.0 | val));
    }

    assert_eq!(parts.next().unwrap(), "|");

    let mut output_digits = [WirePattern(0); 4];
    for od in &mut output_digits {
        let txt = parts.next().unwrap();
        *od = txt
            .bytes()
            .map(|b| 1 << (b - b'a'))
            .fold(WirePattern(0), |acc, val: u8| WirePattern(acc.0 | val));
    }

    NoteEntry {
        unique_patterns,
        output_digits,
    }
}

fn main() {
    let input_path = advent_of_code::env::get_puzzle_input_path("2021-12-08.txt");
    let lines = advent_of_code::iter::line_iter_from_file(&input_path);

    let mut count_of_1478 = 0;
    let mut sum_of_decoded_numbers = 0;
    for (line_idx, line) in lines.enumerate() {
        let entry = parse_note_entry(&line);

        for od in entry.output_digits {
            let num_wires = od.0.count_ones();
            if [2, 3, 4, 7].contains(&num_wires) {
                count_of_1478 += 1;
            }
        }

        let decoded_wire_pats = decode_wire_to_segment_mapping(&entry.unique_patterns);
        let mut num: u32 = 0;
        for od in entry.output_digits {
            num = 10 * num + decoded_wire_pats.decode(od).unwrap() as u32;
        }
        sum_of_decoded_numbers += num;
        println!("Line {}, decoded number: {}", line_idx, num);
    }

    println!("Part 1: num of 1/4/7/8 in output digits: {}", count_of_1478);
    println!("Part 2: sum of decoded numbers: {}", sum_of_decoded_numbers);
}
