fn main() {
    for test_file in ["2021-12-21.sample.txt", "2021-12-21.txt"] {
        println!("------------------ {} ------------------", test_file);
        let test_path = advent_of_code::env::get_puzzle_input_path(test_file);
        let (p1, p2) = parse_puzzle_input(&std::fs::read_to_string(&test_path).unwrap());
        let dd = DeterministicDie {
            sides: 100,
            next_side_idx: 0,
        };

        let out = play(p1, p2, dd);
        let losing_player_score = out.p1_score.min(out.p2_score);
        println!(
            "Part 1: losing score = {}, num rolls = {}, product = {}",
            losing_player_score,
            out.num_rolls,
            losing_player_score * out.num_rolls
        );

        let quantum_out = quantum_play(p1, p2);
        println!(
            "Part 2: p1 wins in {} universes, p2 wins in {} universes. Most wins: {}",
            quantum_out.p1_wins,
            quantum_out.p2_wins,
            quantum_out.p1_wins.max(quantum_out.p2_wins)
        );
    }
}

/// Over-engineered you say? Never heard of her.
trait Die {
    fn roll(&mut self) -> u32;
}

struct DeterministicDie {
    sides: u32,
    next_side_idx: u32,
}

impl Die for DeterministicDie {
    fn roll(&mut self) -> u32 {
        let value = self.next_side_idx + 1; // Turn the side index into a side value.
        self.next_side_idx = (self.next_side_idx + 1) % self.sides;
        value
    }
}

struct Outcome {
    num_rolls: u32,
    p1_score: u32,
    p2_score: u32,
}

fn play(mut p1: u32, mut p2: u32, mut die: impl Die) -> Outcome {
    let mut num_rolls = 0;
    let mut p1_score = 0;
    let mut p2_score = 0;

    loop {
        // P1 plays.
        let roll = die.roll() + die.roll() + die.roll();
        num_rolls += 3;
        p1 = (p1 + roll) % 10;
        p1_score += p1 + 1;
        if p1_score >= 1000 {
            break;
        }

        // P2 plays.
        let roll = die.roll() + die.roll() + die.roll();
        num_rolls += 3;
        p2 = (p2 + roll) % 10;
        p2_score += p2 + 1;
        if p2_score >= 1000 {
            break;
        }
    }

    Outcome {
        num_rolls,
        p1_score,
        p2_score,
    }
}

struct QuantumOutcome {
    p1_wins: u64,
    p2_wins: u64,
}

fn quantum_play(p1_start: u32, p2_start: u32) -> QuantumOutcome {
    #[derive(Clone, Copy, Eq, Hash, PartialEq)]
    struct State {
        /// Position of player 1, 0-indexed.
        p1: u8,
        /// Score of player 1, < 21.
        s1: u8,
        /// Position of player 2, 0-indexed.
        p2: u8,
        /// Score of player 2, < 21.
        s2: u8,
    }
    let mut next_player = 1u8;
    // 44 200 = number of possible states.
    let mut states = std::collections::HashMap::with_capacity(44200);
    states.insert(
        State {
            p1: p1_start as u8,
            s1: 0,
            p2: p2_start as u8,
            s2: 0,
        },
        1u64,
    );
    let mut next_states = std::collections::HashMap::with_capacity(44200);

    // When rolling 3 times, what are the possible outcomes and how many times do they occur?
    let triple_roll_results: Vec<(usize, u64)> = {
        let mut outcome = [0u64; 10];
        for r1 in [1, 2, 3] {
            for r2 in [1, 2, 3] {
                for r3 in [1, 2, 3] {
                    outcome[r1 + r2 + r3] += 1;
                }
            }
        }
        outcome
            .iter()
            .copied()
            .enumerate()
            .filter(|(_idx, count)| *count > 0)
            .collect()
    };

    let mut outcome = QuantumOutcome {
        p1_wins: 0,
        p2_wins: 0,
    };
    while !states.is_empty() {
        next_states.clear();

        if next_player == 1 {
            for (state, count) in &states {
                for (roll_sum, num_rolls) in &triple_roll_results {
                    let new_pos = (state.p1 + *roll_sum as u8) % 10;
                    let new_score = state.s1 + new_pos + 1;
                    if new_score >= 21 {
                        outcome.p1_wins += count * num_rolls;
                    } else {
                        let entry = next_states.entry(State {
                            p1: new_pos,
                            s1: new_score,
                            ..*state
                        });
                        *entry.or_insert(0) += count * num_rolls;
                    }
                }
            }
        } else {
            for (state, count) in &states {
                for (roll_sum, num_rolls) in &triple_roll_results {
                    let new_pos = (state.p2 + *roll_sum as u8) % 10;
                    let new_score = state.s2 + new_pos + 1;
                    if new_score >= 21 {
                        outcome.p2_wins += count * num_rolls;
                    } else {
                        let entry = next_states.entry(State {
                            p2: new_pos,
                            s2: new_score,
                            ..*state
                        });
                        *entry.or_insert(0) += count * num_rolls;
                    }
                }
            }
        }

        std::mem::swap(&mut states, &mut next_states);
        next_player = if next_player == 1 { 2 } else { 1 };
    }

    outcome
}

fn parse_puzzle_input(text: &str) -> (u32, u32) {
    let mut lines = text.lines();
    let p1 = lines
        .next()
        .unwrap()
        .strip_prefix("Player 1 starting position: ")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let p2 = lines
        .next()
        .unwrap()
        .strip_prefix("Player 2 starting position: ")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    assert_eq!(lines.next(), None);
    // We subtract 1 because we use 0-indexed positions.
    (p1 - 1, p2 - 1)
}
