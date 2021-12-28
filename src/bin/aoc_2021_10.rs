fn main() {
    let input_path = advent_of_code::env::get_puzzle_input_path("2021-12-10.txt");
    let lines = advent_of_code::iter::line_iter_from_file(&input_path);

    let mut total_syntax_error_score = 0u32;
    let mut completion_scores = Vec::new();
    for line in lines {
        match parse_line(&line) {
            ParseResult::Corrupted(c) => {
                total_syntax_error_score += error_score_from_closer(c);
            }
            ParseResult::Incomplete(openers) => {
                completion_scores.push(compute_autocomplete_score(&openers));
            }
            _ => {}
        }
    }

    println!(
        "Part 1: total syntax error score: {}",
        total_syntax_error_score
    );

    assert!(completion_scores.len() % 2 == 1);
    let middle_idx = completion_scores.len() / 2;
    completion_scores.select_nth_unstable(middle_idx);
    println!("Part 2: auto-complete scores: {:?}", completion_scores);
    println!("Part 2: middle score: {}", completion_scores[middle_idx]);
}

fn parse_line(line: &str) -> ParseResult {
    // A stack of opening delimiters.
    let mut opening_chars = Vec::new();

    for c in line.chars() {
        match c {
            '(' | '[' | '{' | '<' => opening_chars.push(c),
            ')' | ']' | '}' | '>' => {
                if let Some(opener) = opening_chars.pop() {
                    if opener != opener_from_closer(c) {
                        return ParseResult::Corrupted(c);
                    }
                } else {
                    // We read a closing delimiter, but there was no opener.
                    return ParseResult::Corrupted(c);
                }
            }
            _ => panic!("Invalid char: {}", c),
        }
    }

    if opening_chars.is_empty() {
        ParseResult::Ok
    } else {
        ParseResult::Incomplete(opening_chars)
    }
}

enum ParseResult {
    Ok,
    /// Corrupted line. The char is the first unexpected closing character.
    Corrupted(char),
    /// Incomplete line. Vec contains the sequence of opening chars that need closing.
    Incomplete(Vec<char>),
}

fn opener_from_closer(closer: char) -> char {
    match closer {
        ')' => '(',
        ']' => '[',
        '}' => '{',
        '>' => '<',
        _ => panic!("Invalid char: {}", closer),
    }
}

fn error_score_from_closer(closer: char) -> u32 {
    match closer {
        ')' => 3,
        ']' => 57,
        '}' => 1197,
        '>' => 25137,
        _ => panic!("Invalid char: {}", closer),
    }
}

fn compute_autocomplete_score(openers: &[char]) -> u64 {
    let mut score = 0;
    for c in openers.iter().rev() {
        score *= 5;
        score += match c {
            '(' => 1,
            '[' => 2,
            '{' => 3,
            '<' => 4,
            _ => panic!("Invalid char: {}", c),
        };
    }
    score
}
