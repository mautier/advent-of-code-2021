use std::collections::HashSet;

fn main() {
    for test_file in ["2021-12-13.sample.txt", "2021-12-13.txt"] {
        println!("---------------- {} ----------------", test_file);
        let input_path = advent_of_code::env::get_puzzle_input_path(test_file);

        let (dots, instructions) = parse_puzzle_input(&input_path);
        let dots_one_fold = fold_paper(&dots, instructions[0]);
        println!("Part 1: {} dots after first fold.", dots_one_fold.len());

        let mut final_dots = dots_one_fold;
        for fold in &instructions[1..] {
            final_dots = fold_paper(&final_dots, *fold);
        }
        println!("Part 2 result:");
        print_dots(&final_dots);
    }
}

/// A single dot.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
struct Point {
    x: u16,
    y: u16,
}

/// A fold instruction.
#[derive(Copy, Clone, Debug)]
enum Fold {
    AlongX(u16),
    AlongY(u16),
}

fn fold_paper(dots: &HashSet<Point>, fold: Fold) -> HashSet<Point> {
    let mut result = HashSet::with_capacity(dots.len());

    match fold {
        Fold::AlongX(fold) => {
            for pt in dots {
                if pt.x < fold {
                    result.insert(*pt);
                } else {
                    result.insert(Point {
                        x: fold - (pt.x - fold),
                        y: pt.y,
                    });
                }
            }
        }
        Fold::AlongY(fold) => {
            for pt in dots {
                if pt.y < fold {
                    result.insert(*pt);
                } else {
                    result.insert(Point {
                        x: pt.x,
                        y: fold - (pt.y - fold),
                    });
                }
            }
        }
    }

    result
}

fn print_dots(dots: &HashSet<Point>) {
    let mut dots: Vec<Point> = dots.iter().copied().collect();
    // Sort by row, then column.
    dots.sort_by_key(|pt| (pt.y, pt.x));

    // Next character we'll be printing.
    let mut cursor = Point { x: 0, y: 0 };
    for pt in dots {
        // Print blank lines if this point is further than our cursor.
        while cursor.y < pt.y {
            println!();
            cursor.y += 1;
            cursor.x = 0;
        }

        // Print spaces till we reach `pt`.
        while cursor.x < pt.x {
            print!(" ");
            cursor.x += 1;
        }

        print!("#");
        cursor.x += 1;
    }

    println!();
}

/// Parses the initial pattern of dots, and the list of fold instructions.
fn parse_puzzle_input(path: &std::path::Path) -> (HashSet<Point>, Vec<Fold>) {
    use std::io::BufRead;
    let mut lines =
        std::io::BufReader::new(std::fs::File::open(path).expect("Failed to open file")).lines();

    let mut dots = HashSet::new();
    for line in &mut lines {
        let line = line.expect("Failed to read line");
        if line.is_empty() {
            break;
        }
        let mut parts = line.split(',');
        dots.insert(Point {
            x: parts.next().unwrap().parse::<u16>().unwrap(),
            y: parts.next().unwrap().parse::<u16>().unwrap(),
        });
    }

    let mut instructions = Vec::new();
    for line in lines {
        let line = line.expect("Failed to read line");

        const PREFIX: &str = "fold along ";
        assert!(line.starts_with(PREFIX));
        let rest = &line[PREFIX.len()..];

        match (&rest[0..2], &rest[2..]) {
            ("x=", val) => instructions.push(Fold::AlongX(val.parse::<u16>().unwrap())),
            ("y=", val) => instructions.push(Fold::AlongY(val.parse::<u16>().unwrap())),
            _ => panic!("Invalid line: {}", rest),
        }
    }

    (dots, instructions)
}
