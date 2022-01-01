fn main() {
    for test_file in ["2021-12-17.sample.txt", "2021-12-17.txt"] {
        println!("-------------------- {} --------------------", test_file);
        let path = advent_of_code::env::get_puzzle_input_path(&test_file);
        let text = std::fs::read_to_string(&path).unwrap();
        let target = parse_puzzle_input(&text);
        let res = search_for_highest_reaching_launch(&target);
        println!("Part 1: highest y-coord: {}", res.y_peak);
        println!("Part 2: num feasible shots: {}", res.num_feasible_shots);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Target {
    x_range: std::ops::Range<i64>,
    y_range: std::ops::Range<i64>,
}

struct LaunchResult {
    y_peak: i64,
    num_feasible_shots: usize,
}

/// Returns the highest y-coordinate that can be achieved while reaching the target area.
fn search_for_highest_reaching_launch(target: &Target) -> LaunchResult {
    assert!(target.x_range.start > 0);
    assert!(target.y_range.end < 0);

    // Let's compute the probe position as a function of initial velocity (vx, vy) and step s.
    //
    // For y:
    //      y(vy, s) = vy + vy-1 + ... + vy-(s-1)
    //               = s * vy - (1 + 2 + ... + s-1)
    //               = s * vy - (s-1) * s / 2
    //
    // We'll also want the peak y value:
    //      y_peak(vy) = vy + vy-1 + ... + 1
    //                 = vy * (vy + 1) / 2
    fn y_peak(vy: i64) -> i64 {
        vy * (vy + 1) / 2
    }
    // For x, motion is capped since the velocity will eventually reach 0 and remain there.
    // The farthest position reachable is:
    //     x_end(vx) = vx + vx-1 + ... + 1
    //               = vx * (vx+1) / 2
    //
    // And from there:
    //                 / x_end(vx) if s >= vx
    //      x(vx, s) = |
    //                 \ s * vx - (s-1) * s / 2
    fn x_end(vx: i64) -> i64 {
        vx * (vx + 1) / 2
    }
    fn x(vx: i64, s: i64) -> i64 {
        let s = s as i64;
        if s >= vx {
            x_end(vx)
        } else {
            s * vx - (s - 1) * s / 2
        }
    }

    // We can compute a bounded range of feasible vx values:
    // - The probe must reach the target area (along the x-axis), ie:
    //          x_end(vx) >= target.xmin
    //      <=> vx^2 + vx - 2 * target.xmin >= 0
    //      <=> vx >= (-1 + sqrt(1 + 8 * xmin)) / 2
    // - The probe must not overshoot the target area immediately on step 1, ie:
    //          vx < target.xmax
    let vx_min =
        f32::ceil((-1.0 + f32::sqrt(1.0 + 8.0 * target.x_range.start as f32)) / 2.0) as i64;
    let vx_max = target.x_range.end;

    // When we find a (s, vx) pair for which x(vx, s) is in the target range, we'll want to look
    // for a vy value that also works:
    //          ymin <= y(vy, s) < ymax
    //      <=> ymin <= s * vy - (s-1)*s/2 < ymax
    //      <=> (s-1)/2 + ymin/s <= vy < (s-1)/2 + ymax/s
    fn vy_min(s: i64, ymin: i64) -> i64 {
        let s = s as f32;
        f32::ceil((s - 1.0) / 2.0 + ymin as f32 / s) as i64
    }
    fn vy_max(s: i64, ymax: i64) -> i64 {
        let s = s as f32;
        f32::ceil((s - 1.0) / 2.0 + ymax as f32 / s) as i64
    }

    let mut max_y_peak = 0;
    // When a single (vx, vy) yields multiple positions in the target zone, we will find all those
    // positions. Using a hash set here deduplicates the results.
    let mut feasible_shots = std::collections::BTreeSet::new();

    for vx in vx_min..vx_max {
        // Look for steps s where the probe is still in motion along x, and x(vx, s) is in the
        // target area.
        for s in 1..vx {
            if !target.x_range.contains(&x(vx, s)) {
                continue;
            }
            // (s, vx) is a candidate. Let's see if there exists vy such that the probe is in the
            // right spot.
            for vy in vy_min(s, target.y_range.start)..vy_max(s, target.y_range.end) {
                max_y_peak = max_y_peak.max(y_peak(vy));
                feasible_shots.insert((vx, vy));
            }
        }

        // Now, if x_end(vx) is in the target area, consider the range [s, infinity) of steps where
        // x is in the target area.
        if !target.x_range.contains(&x_end(vx)) {
            continue;
        }
        // The first step for which x == x_end is:
        let s_min = vx;

        // Going back to the range of suitable vy values:
        //      (s-1)/2 + ymin/s <= vy < (s-1)/2 + ymax/s
        //
        // Given that ymin / ymax are negative:
        // - s is odd => (s-1)/2 is an integer k, and the range is of the form [k - a, k - b)
        //   for some positive a and b.
        //   When a < 1, there are no integers in the range.
        // - s is even => (s-1)/2 is of the form k.5, and the range is [k.5 - a, k.5 - b)
        //   When a < 0.5, there are no integers in the range.
        //
        // Therefore, we can compute an upper bound on the values of s that may yield feasible
        // values of vy: |ymin|/s < 0.5, ie s > 2 * |ymin|
        //
        // This gives us an upper bound on the interesting values of s:
        assert!(target.y_range.start < 0);
        // Note: the trailing +1 is because s_max should be excluded (the inegality above is
        // strict).
        let s_max = - 2 * target.y_range.start + 1;

        for s in s_min..s_max {
            for vy in vy_min(s, target.y_range.start)..vy_max(s, target.y_range.end) {
                max_y_peak = max_y_peak.max(y_peak(vy));
                feasible_shots.insert((vx, vy));
            }
        }
    }

    assert!(feasible_shots.len() > 0);
    LaunchResult {
        y_peak: max_y_peak,
        num_feasible_shots: feasible_shots.len(),
    }
}

fn parse_puzzle_input(text: &str) -> Target {
    // Parses a number at the begining of the string, and returns the number and what's left of the
    // string.
    fn parse_number(s: &mut &str) -> i64 {
        let mut num_chars = 0;
        let mut num = 0;
        let sign = if *s.as_bytes().first().unwrap() == b'-' {
            *s = &(*s)[1..];
            -1
        } else {
            1
        };
        for c in s.chars() {
            if c.is_ascii_digit() {
                num_chars += 1;
                num = num * 10 + (c as u32 - '0' as u32) as i64;
            } else {
                break;
            }
        }
        *s = &(*s)[num_chars..];
        sign * num
    }

    let mut text = text.strip_prefix("target area: x=").unwrap();

    let xmin = parse_number(&mut text);
    text = text.strip_prefix("..").unwrap();
    let xmax = parse_number(&mut text) + 1;

    text = text.strip_prefix(", y=").unwrap();

    let ymin = parse_number(&mut text);
    text = text.strip_prefix("..").unwrap();
    let ymax = parse_number(&mut text) + 1;

    text = text.trim_end();
    assert!(text.is_empty());

    Target {
        x_range: xmin..xmax,
        y_range: ymin..ymax,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_puzzle_inpu() {
        let input_string = "target area: x=20..30, y=-10..-5\n";
        let target = super::parse_puzzle_input(&input_string);
        assert_eq!(
            target,
            super::Target {
                x_range: 20..31,
                y_range: -10..-4
            }
        );
    }
}
