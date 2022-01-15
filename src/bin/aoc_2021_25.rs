use advent_of_code::image::Image;

fn main() {
    for test_file in ["2021-12-25.sample.txt", "2021-12-25.txt"] {
        println!(
            "----------------------- {} -----------------------",
            test_file
        );
        let input_path = advent_of_code::env::get_puzzle_input_path(test_file);

        let mut image = parse_puzzle_input(&std::fs::read_to_string(&input_path).unwrap());
        let mut next_image = Image::new_with_same_shape(&image, Spot::Empty);

        let mut num_steps = 0;
        loop {
            num_steps += 1;
            let moved_east = step::<EAST>(&image, &mut next_image);
            let moved_south = step::<SOUTH>(&next_image, &mut image);

            if !moved_east && !moved_south {
                break;
            }
        }

        println!("Part 1: no more movement after {} steps.", num_steps);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Spot {
    /// The spot is empty (.)
    Empty,
    /// The spot contains a sea cucumber, traveling in a certain direction.
    SeaCuc(Direction),
}

type Direction = u8;
const EAST: Direction = 0;
const SOUTH: Direction = 1;

/// Advances all the sea cucumbers by one step, storing the result in `next`.
///
/// `next` is passed in to avoid dynamic memory allocations.
///
/// Returns `true` if something moved, `false` otherwise.
fn step<const DIR: Direction>(current: &Image<Spot>, next: &mut Image<Spot>) -> bool {
    assert_eq!(current.size_hw(), next.size_hw());
    assert!(DIR == EAST || DIR == SOUTH);

    // The next image starts in the same configuration as the current.
    next.data.copy_from_slice(&current.data);

    let mut any_movement = false;

    for (row, col, spot) in current.enumerate_pixels() {
        if *spot != Spot::SeaCuc(DIR) {
            continue;
        }

        let (next_row, next_col) = if DIR == EAST {
            (row, (col + 1) % current.width)
        } else {
            ((row + 1) % current.height, col)
        };

        if *current.pixel(next_row, next_col) == Spot::Empty {
            any_movement = true;
            *next.pixel_mut(row, col) = Spot::Empty;
            *next.pixel_mut(next_row, next_col) = Spot::SeaCuc(DIR);
        }
    }

    any_movement
}

fn parse_puzzle_input(text: &str) -> Image<Spot> {
    let mut height = 0;
    let mut width = 0;
    let mut data = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        height += 1;

        let line_len = line.as_bytes().len() as u16;
        if width == 0 {
            width = line_len;
        }
        assert_eq!(width, line_len);

        data.extend(line.bytes().map(|b| match b {
            b'.' => Spot::Empty,
            b'>' => Spot::SeaCuc(EAST),
            b'v' => Spot::SeaCuc(SOUTH),
            _ => panic!("Invalid image pixel character: {}", b),
        }));
    }

    Image {
        height,
        width,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_puzzle_input() {
        let e = Spot::Empty;
        let r = Spot::SeaCuc(EAST);
        let d = Spot::SeaCuc(SOUTH);
        let input_str = "\
v...>
.vv>>";
        let img = parse_puzzle_input(input_str);
        assert_eq!(
            img,
            Image {
                height: 2,
                width: 5,
                data: vec![d, e, e, e, r, e, d, d, r, r],
            }
        );
    }

    #[test]
    fn test_step_on_sample() {
        let initial_img = parse_puzzle_input(
            "\
v...>>.vv>
.vv>>.vv..
>>.>v>...v
>>v>>.>.v.
v>v.vv.v..
>.>>..v...
.vv..>.>v.
v.v..>>v.v
....v..v.>
",
        );

        let mut img = initial_img.clone();
        let mut next = Image {
            height: 9,
            width: 10,
            data: vec![Spot::Empty; 9 * 10],
        };

        // (number of steps, expected image)
        let expected_outputs: std::collections::HashMap<usize, Image<Spot>> = [
            (
                1,
                parse_puzzle_input(
                    "\
....>.>v.>
v.v>.>v.v.
>v>>..>v..
>>v>v>.>.v
.>v.v...v.
v>>.>vvv..
..v...>>..
vv...>>vv.
>.v.v..v.v
",
                ),
            ),
            (
                2,
                parse_puzzle_input(
                    "\
>.v.v>>..v
v.v.>>vv..
>v>.>.>.v.
>>v>v.>v>.
.>..v....v
.>v>>.v.v.
v....v>v>.
.vv..>>v..
v>.....vv.
",
                ),
            ),
            (
                3,
                parse_puzzle_input(
                    "\
v>v.v>.>v.
v...>>.v.v
>vv>.>v>..
>>v>v.>.v>
..>....v..
.>.>v>v..v
..v..v>vv>
v.v..>>v..
.v>....v..
",
                ),
            ),
            (
                4,
                parse_puzzle_input(
                    "\
v>..v.>>..
v.v.>.>.v.
>vv.>>.v>v
>>.>..v>.>
..v>v...v.
..>>.>vv..
>.v.vv>v.v
.....>>vv.
vvv>...v..",
                ),
            ),
            (
                5,
                parse_puzzle_input(
                    "\
vv>...>v>.
v.v.v>.>v.
>.v.>.>.>v
>v>.>..v>>
..v>v.v...
..>.>>vvv.
.>...v>v..
..v.v>>v.v
v.v.>...v.",
                ),
            ),
            (
                10,
                parse_puzzle_input(
                    "\
..>..>>vv.
v.....>>.v
..v.v>>>v>
v>.>v.>>>.
..v>v.vv.v
.v.>>>.v..
v.v..>v>..
..v...>v.>
.vv..v>vv.",
                ),
            ),
            (
                20,
                parse_puzzle_input(
                    "\
v>.....>>.
>vv>.....v
.>v>v.vv>>
v>>>v.>v.>
....vv>v..
.v.>>>vvv.
..v..>>vv.
v.v...>>.v
..v.....v>",
                ),
            ),
            (
                30,
                parse_puzzle_input(
                    "\
.vv.v..>>>
v>...v...>
>.v>.>vv.>
>v>.>.>v.>
.>..v.vv..
..v>..>>v.
....v>..>v
v.v...>vv>
v.v...>vvv",
                ),
            ),
            (
                40,
                parse_puzzle_input(
                    "\
>>v>v..v..
..>>v..vv.
..>>>v.>.v
..>>>>vvv>
v.....>...
v.v...>v>>
>vv.....v>
.>v...v.>v
vvv.v..v.>",
                ),
            ),
            (
                50,
                parse_puzzle_input(
                    "\
..>>v>vv.v
..v.>>vv..
v.>>v>>v..
..>>>>>vv.
vvv....>vv
..v....>>>
v>.......>
.vv>....v>
.>v.vv.v..",
                ),
            ),
            (
                55,
                parse_puzzle_input(
                    "\
..>>v>vv..
..v.>>vv..
..>>v>>vv.
..>>>>>vv.
v......>vv
v>v....>>v
vvv...>..>
>vv.....>.
.>v.vv.v..",
                ),
            ),
            (
                56,
                parse_puzzle_input(
                    "\
..>>v>vv..
..v.>>vv..
..>>v>>vv.
..>>>>>vv.
v......>vv
v>v....>>v
vvv....>.>
>vv......>
.>v.vv.v..",
                ),
            ),
            (
                57,
                parse_puzzle_input(
                    "\
..>>v>vv..
..v.>>vv..
..>>v>>vv.
..>>>>>vv.
v......>vv
v>v....>>v
vvv.....>>
>vv......>
.>v.vv.v..",
                ),
            ),
        ]
        .into_iter()
        .collect();

        for i in 1..=57 {
            let moved_east = step::<EAST>(&img, &mut next);
            let moved_south = step::<SOUTH>(&next, &mut img);
            assert!(moved_east || moved_south);

            if let Some(expected) = expected_outputs.get(&i) {
                assert_eq!(expected, &img);
            }
        }

        // 58-th step: no movement.
        assert!(!step::<EAST>(&img, &mut next));
        assert!(!step::<SOUTH>(&next, &mut img));
    }
}
