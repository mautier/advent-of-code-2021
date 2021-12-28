fn main() {
    let input_path = advent_of_code::env::get_puzzle_input_path("2021-12-07.txt");
    let mut input_line = advent_of_code::iter::line_iter_from_file(&input_path);
    let mut positions: Vec<_> = input_line
        .next()
        .unwrap()
        .split(',')
        .map(|p| p.parse::<u16>().unwrap())
        .collect();

    // Part 1:
    // We're trying to find `target` that minimizes:
    //      cost(target) = Sum_pos { |pos - target| }
    // Equivalently, we're trying to minimize the arithmetic mean of absolute deviations.
    // That is simply the median.

    assert!(!positions.is_empty());
    let median = if positions.len() % 2 == 0 {
        let median_left_idx = positions.len() / 2 - 1;
        let (_, median_left, right) = positions.select_nth_unstable(median_left_idx);
        let median_right = right.iter().copied().min().unwrap();
        let sum = *median_left + median_right;
        // The median in this case is in-between 2 input values; since an integer is expected, make
        // sure that the result is actually one.
        assert!(sum % 2 == 0);
        sum / 2
    } else {
        let median_idx = positions.len() / 2;
        let (_, median, _) = positions.select_nth_unstable(median_idx);
        *median
    };

    // Compute the cost for this target.
    let total_cost: i32 = positions
        .iter()
        .copied()
        .map(|p| i32::abs(p as i32 - median as i32))
        .sum();
    println!(
        "Part 1: best position {}, total cost: {}",
        median, total_cost
    );

    // Part 2:
    // Now the unary cost becomes:
    //      cost(pos, target) = dist * (dist + 1) / 2
    //      where dist = |pos - target|.
    //
    // It's no longer obvious what the best position is, so we'll just try all of them. This will
    // cost O(n^2).
    fn compute_cost(target: u16, positions: &[u16]) -> u32 {
        positions
            .iter()
            .copied()
            .map(|p| {
                let dist = i32::abs(p as i32 - target as i32) as u32;
                dist * (dist + 1) / 2
            })
            .sum()
    }
    let min_target = positions.iter().copied().min().unwrap();
    let max_target = positions.iter().copied().max().unwrap();
    let min_cost = (min_target..=max_target)
        .map(|target| compute_cost(target, &positions[..]))
        .min()
        .unwrap();
    println!("Part 2: min cost {}", min_cost);
}
