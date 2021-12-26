fn strip_prefix_and_parse_int(txt: &str, prefix: &str) -> Option<i32> {
    if !txt.starts_with(prefix) {
        return None;
    }
    Some(
        txt[prefix.len()..]
            .parse::<i32>()
            .expect("Failed to parse i32"),
    )
}

fn main() {
    let input_path = {
        let mut p = advent_of_code::env::get_data_dir();
        p.push("2021-12-02.txt");
        p
    };
    let lines = advent_of_code::iter::line_iter_from_file(&input_path);

    const FORWARD: &str = "forward ";
    const DOWN: &str = "down ";
    const UP: &str = "up ";

    let mut part1_pos = 0i32;
    let mut part1_depth = 0i32;

    let mut part2_pos = 0i32;
    let mut part2_aim = 0i32;
    let mut part2_depth = 0i32;
    for l in lines {
        if let Some(x) = strip_prefix_and_parse_int(&l, FORWARD) {
            part1_pos += x;
            part2_pos += x;
            part2_depth += part2_aim * x;
            continue;
        }
        if let Some(x) = strip_prefix_and_parse_int(&l, DOWN) {
            part1_depth += x;
            part2_aim += x;
            continue;
        }
        if let Some(x) = strip_prefix_and_parse_int(&l, UP) {
            part1_depth -= x;
            part2_aim -= x;
            continue;
        }
    }

    println!(
        "Part1: pos = {}, depth = {}, product = {}",
        part1_pos,
        part1_depth,
        part1_pos * part1_depth
    );
    println!(
        "Part2: pos = {}, depth = {}, product = {}",
        part2_pos,
        part2_depth,
        part2_pos * part2_depth
    );
}
