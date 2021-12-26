fn compute_most_frequent_bit(lines: &[String], bit_index: usize) -> u32 {
    let mut num_ones = 0usize;
    for l in lines.iter() {
        if l.as_bytes()[bit_index] == b'1' {
            num_ones += 1;
        }
    }

    let num_zeroes = lines.len() - num_ones;
    if num_ones >= num_zeroes {
        1
    } else {
        0
    }
}

fn retain_with_bit(mut lines: Vec<String>, bit_index: usize, bit_value: u32) -> Vec<String> {
    let byte_value = if bit_value == 1 { b'1' } else { b'0' };

    lines.retain(|l| l.as_bytes()[bit_index] == byte_value);
    lines
}

fn u32_from_line(l: &str) -> u32 {
    assert!(l.len() == 12);
    let mut res = 0;
    for x in l.bytes() {
        res <<= 1;
        if x == b'1' {
            res += 1;
        }
    }
    res
}

fn main() {
    let input_path = {
        let mut p = advent_of_code::env::get_data_dir();
        p.push("2021-12-03.txt");
        p
    };
    let lines: Vec<_> = advent_of_code::iter::line_iter_from_file(&input_path).collect();

    {
        let mut one_counts: [usize; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        for l in &lines {
            assert_eq!(l.len(), 12);
            for (i, b) in l.as_bytes().iter().enumerate() {
                if *b == b'1' {
                    one_counts[i] += 1;
                }
            }
        }

        let mut gamma = 0u32;
        for (i, num_ones) in one_counts.iter().enumerate() {
            let num_zeroes = lines.len() - num_ones;
            if *num_ones > num_zeroes {
                gamma += 1 << (11 - i);
            }
        }

        let epsilon = (!gamma) & 0xfff;
        println!(
            "gamma: {}, epsilon: {}, product: {}",
            gamma,
            epsilon,
            gamma * epsilon
        );
    }

    {
        let mut oxygen_generator_lines = lines.clone();
        let oxygen_rating = {
            let mut i = 0;
            loop {
                assert!(!oxygen_generator_lines.is_empty());

                let most_frequent_value = compute_most_frequent_bit(&oxygen_generator_lines[..], i);
                oxygen_generator_lines =
                    retain_with_bit(oxygen_generator_lines, i, most_frequent_value);

                if oxygen_generator_lines.len() == 1 {
                    break u32_from_line(oxygen_generator_lines.first().unwrap());
                }
                i += 1;
            }
        };

        let mut co2_scrubber_lines = lines;
        let co2_rating = {
            let mut i = 0;
            loop {
                assert!(!co2_scrubber_lines.is_empty());

                let most_frequent_value = compute_most_frequent_bit(&co2_scrubber_lines[..], i);
                let least_frequent_value = 1 - most_frequent_value;
                co2_scrubber_lines = retain_with_bit(co2_scrubber_lines, i, least_frequent_value);

                if co2_scrubber_lines.len() == 1 {
                    break u32_from_line(co2_scrubber_lines.first().unwrap());
                }
                i += 1;
            }
        };
        println!(
            "Oxygen: {}, CO2: {}, product: {}",
            oxygen_rating,
            co2_rating,
            oxygen_rating * co2_rating
        );
    }
}
