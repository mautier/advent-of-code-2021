fn main() {
    let input_path = advent_of_code::env::get_puzzle_input_path("2021-12-06.txt");

    let input_line = advent_of_code::iter::line_iter_from_file(&input_path)
        .next()
        .unwrap();
    let initial_fish_days = input_line.split(',').map(|d| d.parse::<u8>().unwrap());

    // A histogram of how many fishes have d days left.
    // Min number of days is 0, max number is 8.
    let mut fish_day_hist = [0usize; 9];
    for fish in initial_fish_days {
        fish_day_hist[fish as usize] += 1;
    }

    for day in 1..=256 {
        // All fish with 0 days will spawn new ones.
        let fish_with_0_days = fish_day_hist[0];

        // Reduce the number of days by one for all existing fish.
        for d in 0..=7 {
            fish_day_hist[d] = fish_day_hist[d + 1];
        }

        // Reset the counter for the 0-day fish.
        fish_day_hist[6] += fish_with_0_days;

        // Spawn new fish.
        fish_day_hist[8] = fish_with_0_days;

        if day == 80 || day == 256 {
            let count: usize = fish_day_hist.iter().copied().sum();
            println!("After {} days, there are {} fish.", day, count);
        }
    }
}
