use std::collections::HashMap;

fn main() {
    for test_file in ["2021-12-14.sample.txt", "2021-12-14.txt"] {
        println!("--------------- {} ---------------", test_file);
        let input_path = advent_of_code::env::get_puzzle_input_path(test_file);
        let (template, rules) =
            parse_puzzle_input(advent_of_code::iter::line_iter_from_file(&input_path));

        let mut result = template.clone();
        let mut input = Vec::new();
        for _step in 0..10 {
            std::mem::swap(&mut input, &mut result);
            result.clear();
            apply_rules(&input, &rules, &mut result);
        }

        let (min_count, max_count) = find_min_max_element_counts(&result);
        println!(
            "Part 1: min count = {}, max count = {}, difference: {}",
            min_count,
            max_count,
            max_count - min_count
        );

        let mut result = count_digrams(&template);
        for _step in 0..40 {
            result = apply_rules_to_digrams(&result, &rules);
        }

        let (min_count, max_count) = find_min_max_element_counts_in_digrams(&template, &result);
        println!(
            "Part 2: min count = {}, max count = {}, difference: {}",
            min_count,
            max_count,
            max_count - min_count
        );
    }
}

type Element = u8;
type Pair = [Element; 2];

fn apply_rules(input: &[Element], rules: &HashMap<Pair, Element>, result: &mut Vec<Element>) {
    result.clear();
    for pair in input.windows(2) {
        // Always push the first from the pair.
        result.push(pair[0]);
        // Then maybe push the newly inserted element.
        let pair: Pair = pair.try_into().unwrap();
        if let Some(to_insert) = rules.get(&pair) {
            result.push(*to_insert);
        }
    }
    // Push the final element from the input.
    result.push(*input.last().unwrap());
}

fn find_min_max_element_counts(input: &[Element]) -> (usize, usize) {
    let mut histogram = [0usize; 256];
    for elem in input {
        histogram[*elem as usize] += 1;
    }

    let mut min = input.len();
    let mut max = 0;
    for count in histogram {
        if count == 0 {
            continue;
        }
        min = min.min(count);
        max = max.max(count);
    }

    (min, max)
}

/// Count the number of occurrences of all the digrams appearing in a sequence.
fn count_digrams(sequence: &[Element]) -> HashMap<Pair, usize> {
    let mut digram_counts = HashMap::new();
    for pair in sequence.windows(2) {
        let pair: Pair = pair.try_into().unwrap();
        *digram_counts.entry(pair).or_insert(0) += 1;
    }
    digram_counts
}

/// Apply the insertion rules to a sequence represented by its digrams.
fn apply_rules_to_digrams(digrams: &HashMap<Pair, usize>, rules: &HashMap<Pair, Element>) -> HashMap<Pair, usize> {
    let mut new_digrams = HashMap::new();
    for (digram, count) in digrams {
        if let Some(to_insert) = rules.get(digram) {
            let first = [digram[0], *to_insert];
            *new_digrams.entry(first).or_insert(0) += count;

            let second = [*to_insert, digram[1]];
            *new_digrams.entry(second).or_insert(0) += count;
        } else {
            *new_digrams.entry(*digram).or_insert(0) += count;
        }
    }

    new_digrams
}

fn find_min_max_element_counts_in_digrams(original_template: &[Element], digrams: &HashMap<Pair, usize>) -> (usize, usize) {
    let mut histogram = [0usize; 256];

    for (digram, count) in digrams {
        // To avoid double-counting elements, we only count the first in the digram.
        histogram[digram[0] as usize] += count;
    }
    // We're only missing 1 element: the very last one in the sequence, which never appears in
    // first position of any digram. But that last one also happens to have remained the same
    // element as the last element of the original template string!
    histogram[*original_template.last().unwrap() as usize] += 1;

    let mut min = std::usize::MAX;
    let mut max = 0;
    for count in histogram {
        if count == 0 {
            continue;
        }
        min = min.min(count);
        max = max.max(count);
    }

    (min, max)
}

fn parse_puzzle_input(
    mut lines: impl Iterator<Item = String>,
) -> (Vec<Element>, HashMap<Pair, Element>) {
    let template: Vec<Element> = lines.next().unwrap().as_bytes().into();

    let mut rules = HashMap::new();
    for line in lines {
        let bytes = line.as_bytes();
        assert_eq!(bytes.len(), 7);
        assert_eq!(&bytes[2..6], b" -> ");

        rules.insert([bytes[0], bytes[1]], bytes[6]);
    }

    (template, rules)
}
