fn main() {
    let input_path = {
        let mut p = advent_of_code::env::get_data_dir();
        p.push("2021-12-01.txt");
        p
    };
    let file =
        std::io::BufReader::new(std::fs::File::open(input_path).expect("Failed to open input"));

    use std::io::BufRead;
    let depths = file.lines().filter_map(|l: std::io::Result<String>| {
        let l = l.expect("Failed to read line");
        if l.is_empty() {
            None
        } else {
            Some(l.parse::<i32>().expect("Failed to parse i32"))
        }
    }).collect::<Vec<_>>();


    // Part 1: each depth on its own.
    {
        let mut iter = depths.iter().copied();
        let mut prev_depth = iter.next().expect("Empty input");
        let mut num_increases = 0;
        for d in iter {
            if d > prev_depth {
                num_increases += 1;
            }
            prev_depth = d;
        }

        println!("Part 1: {} increases.", num_increases);
    }

    // Part 2: a sliding window.
    {
        let mut iter = advent_of_code::iter::WindowIterator::<_, 3>::new(depths.iter().copied());
        let mut prev_sum: i32 = iter.next().expect("Not enough inputs").iter().sum();
        let mut num_increases = 0;
        for win in iter {
            let s = win.iter().sum();
            if s > prev_sum {
                num_increases += 1;
            }
            prev_sum = s;
        }

        println!("Part 2: {} increases.", num_increases);
    }
}
