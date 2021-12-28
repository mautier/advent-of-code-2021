struct Bingo {
    /// The sequence of numbers that will be selected, with duplicates removed (just in case,
    /// unclear if actually needed).
    deduped_numbers: Vec<u8>,
    /// The bingo boards.
    boards: Vec<Board>,
}

/// A single cell on a board. Initially, all cells are Unmarked.
#[derive(Clone, Copy, Debug)]
enum BoardCell {
    Unmarked(u8),
    Marked,
}

struct Board {
    /// Row-major matrix of numbers.
    matrix: [[BoardCell; 5]; 5],
    /// For each row, the total number of `Marked` cells. Used for quickly checking for fully
    /// marked rows.
    num_marked_per_row: [u8; 5],
    /// Similar to `num_marked_per_row`, but for columns.
    num_marked_per_col: [u8; 5],
}

/// Outcome of marking a number on a bingo board.
enum MarkOutcome {
    /// Nothing special happened.
    None,
    /// The board is complete, and here's its score.
    Bingo(u32),
}

impl Board {
    fn mark(&mut self, n: u8) -> MarkOutcome {
        for (row_idx, row) in self.matrix.iter_mut().enumerate() {
            for (col_idx, col) in row.iter_mut().enumerate() {
                match col {
                    BoardCell::Unmarked(x) if *x == n => {
                        *col = BoardCell::Marked;
                        self.num_marked_per_row[row_idx] += 1;
                        self.num_marked_per_col[col_idx] += 1;
                    }
                    _ => {}
                }
            }
        }

        if self.num_marked_per_row.contains(&5) || self.num_marked_per_col.contains(&5) {
            // Bingo! Compute the score.
            let sum_unmarked: u32 = self
                .matrix
                .iter()
                .flat_map(|row| row.iter().copied())
                .map(|cell| match cell {
                    BoardCell::Unmarked(x) => x as u32,
                    BoardCell::Marked => 0u32,
                })
                .sum();

            let score = n as u32 * sum_unmarked;
            MarkOutcome::Bingo(score)
        } else {
            MarkOutcome::None
        }
    }
}

/// Parses the input lines, **assuming the empty lines have been removed**.
fn parse_puzzle_input(mut lines: impl Iterator<Item = String>) -> Bingo {
    // First line is the drawn numbers, comma-separated.
    let mut numbers: Vec<u8> = lines
        .next()
        .expect("Missing 1st line")
        .split(',')
        .map(|num_str| num_str.parse::<u8>().expect("Failed to parse u8"))
        .collect();

    let deduped_numbers = {
        let mut seen = std::collections::HashSet::with_capacity(numbers.len());
        numbers.retain(|&n| seen.insert(n));
        numbers
    };

    // Now parse the boards: 5 lines of numbers per board.
    let mut boards = Vec::new();
    'board_loop: loop {
        let mut matrix = [[BoardCell::Marked; 5]; 5];
        for matrix_row in &mut matrix {
            let row = if let Some(r) = lines.next() {
                r
            } else {
                break 'board_loop;
            };
            let row_nums = row.split(' ').filter_map(|maybe_num| {
                if maybe_num.is_empty() {
                    None
                } else {
                    Some(maybe_num.parse::<u8>().unwrap())
                }
            });

            for (col_idx, n) in row_nums.enumerate() {
                matrix_row[col_idx] = BoardCell::Unmarked(n);
            }
        }

        boards.push(Board {
            matrix,
            num_marked_per_row: [0; 5],
            num_marked_per_col: [0; 5],
        });
    }

    Bingo {
        deduped_numbers,
        boards,
    }
}

fn main() {
    let input_path = advent_of_code::env::get_puzzle_input_path("2021-12-04.txt");
    let lines = advent_of_code::iter::line_iter_from_file(&input_path);

    let mut bingo = parse_puzzle_input(lines);
    let mut board_has_won = vec![false; bingo.boards.len()];

    let mut first_win_score = None;
    let mut last_win_score = None;

    for n in bingo.deduped_numbers {
        for (board, has_won) in bingo.boards.iter_mut().zip(&mut board_has_won) {
            if *has_won {
                continue;
            }
            match board.mark(n) {
                MarkOutcome::None => {}
                MarkOutcome::Bingo(score) => {
                    if first_win_score.is_none() {
                        first_win_score = Some(score);
                    } else {
                        last_win_score = Some(score);
                    }
                    *has_won = true;
                }
            }
        }
    }

    println!("First winning board score: {}", first_win_score.unwrap());
    println!("Last winning board score: {}", last_win_score.unwrap());
}
