fn main() {
    let input_path = advent_of_code::env::get_puzzle_input_path("2021-12-11.txt");
    let lines = advent_of_code::iter::line_iter_from_file(&input_path);

    let img = parse_input_image(lines);

    let total_flashes = simulate_n_steps(img.clone(), 100);
    println!(
        "Part 1: after 100 steps, {} flashes have occurred.",
        total_flashes
    );

    let synchronized_flashing_step = find_synchronized_flashing_step(img);
    println!(
        "Part 2: after {} steps, all octopuses flash simultaneously.",
        synchronized_flashing_step
    );
}

#[derive(Copy, Clone)]
struct ChargeLevel(u8);

impl ChargeLevel {
    fn increment(&mut self) -> Effect {
        self.0 = self.0.saturating_add(1);
        if self.0 == 10 {
            Effect::Flash
        } else {
            Effect::None
        }
    }

    fn reset_if_flashed(&mut self) {
        if self.0 > 9 {
            self.0 = 0;
        }
    }
}

enum Effect {
    Flash,
    None,
}

/// Returns the number of flashes after n steps have occurred.
fn simulate_n_steps(mut img: Image<ChargeLevel>, n: usize) -> usize {
    let mut scratch_buffer = Vec::new();
    (0..n).map(|_| step(&mut img, &mut scratch_buffer)).sum()
}

/// Returns the number of steps before all octopuses flash simultaneously.
fn find_synchronized_flashing_step(mut img: Image<ChargeLevel>) -> usize {
    let mut scratch_buffer = Vec::new();
    for n in 1.. {
        let flashes = step(&mut img, &mut scratch_buffer);
        if flashes == (img.height * img.width) as usize {
            return n;
        }
    }
    unreachable!()
}

/// Simulates a single step, returning the number of flashes.
///
/// `scratch_buffer` is an implementation detail; passing the same Vec when calling step again and
/// again saves on allocations (the buffer is cleared internally).
fn step(img: &mut Image<ChargeLevel>, scratch_buffer: &mut Vec<(i32, i32)>) -> usize {
    let to_process = scratch_buffer;
    to_process.clear();

    // First pass: all octopuses charge by 1 on their own.
    for row in 0..img.height {
        for col in 0..img.width {
            match img.pixel_mut(row, col).increment() {
                Effect::None => {}
                Effect::Flash => to_process.push((row, col)),
            }
        }
    }

    // Now, propagate all flashing effects.
    let mut total_flashes = 0usize;
    while let Some((row, col)) = to_process.pop() {
        total_flashes += 1;

        #[rustfmt::skip]
        let neighbors = [
            (row - 1, col - 1), (row - 1, col), (row - 1, col + 1),
            (row    , col - 1),                 (row    , col + 1),
            (row + 1, col - 1), (row + 1, col), (row + 1, col + 1),
        ];

        for (nrow, ncol) in neighbors {
            if nrow < 0 || img.height <= nrow || ncol < 0 || img.width <= ncol {
                // Out-of-bounds neighbor.
                continue;
            }
            match img.pixel_mut(nrow, ncol).increment() {
                Effect::None => {}
                Effect::Flash => to_process.push((nrow, ncol)),
            }
        }
    }

    // Finally, reset all the octopuses that flashed.
    for charge_level in &mut img.data {
        charge_level.reset_if_flashed();
    }

    total_flashes
}

struct Image<T> {
    height: i32,
    width: i32,
    /// Row-major pixel data, of size height * width.
    data: Vec<T>,
}

impl<T> Image<T> {
    fn pixel_mut(&mut self, row: i32, col: i32) -> &mut T {
        &mut self.data[(row * self.width + col) as usize]
    }
}

impl<T: Clone> Clone for Image<T> {
    fn clone(&self) -> Self {
        Image {
            height: self.height,
            width: self.width,
            data: self.data.clone(),
        }
    }
}

fn parse_input_image(lines: impl Iterator<Item = String>) -> Image<ChargeLevel> {
    let mut height = 0;
    let mut width = 0;
    let mut data = Vec::new();
    for line in lines {
        height += 1;
        if width == 0 {
            width = line.len() as i32;
        }
        assert_eq!(width, line.len() as i32);

        data.extend(line.bytes().map(|b| ChargeLevel(b - b'0')));
    }

    Image {
        height,
        width,
        data,
    }
}
