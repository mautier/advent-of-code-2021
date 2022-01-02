fn main() {
    for test_file in ["2021-12-19.sample.txt", "2021-12-19.txt"] {
        println!("------------------ {} ------------------", test_file);
        let test_path = advent_of_code::env::get_puzzle_input_path(test_file);
        let test_contents = std::fs::read_to_string(&test_path).unwrap();
        let mut reports = parse_puzzle_input(&test_contents);

        // The set of beacons, with coordinates in the reference frame (the frame of the first
        // scanner we'll process).
        let mut beacons_in_ref_frame: std::collections::HashSet<Position> = Default::default();

        // Scanner reports that have been transformed to the reference frame, but have yet to be
        // matched against `unmatched` reports.
        let mut scanner_positions = vec![Position::new(0, 0, 0)];
        let mut processed = vec![reports.pop().unwrap()];
        let mut unmatched = reports;

        while let Some(report) = processed.pop() {
            // Add the beacons from this report to the set.
            beacons_in_ref_frame.extend(report.beacons.iter().cloned());

            // Using `report` as reference, try to match other reports.
            unmatched.retain(|other| {
                if let Some((transform, transformed_other, count)) =
                    find_rotation_and_beacon_matches(&report, other)
                {
                    assert!(count >= 12);
                    scanner_positions.push(Position(transform.translation.0));
                    // Now that we've warped `other` into the canonical frame, use it in a future
                    // iteration to find more matches.
                    processed.push(transformed_other);
                    false
                } else {
                    // No matches, so keep this report here in `unmatched`.
                    true
                }
            });
        }

        // We should have succeeded in finding all coordinate frames and all beacons.
        assert!(unmatched.is_empty());
        println!(
            "Part 1: number of unique beacons: {}",
            beacons_in_ref_frame.len()
        );

        // Iterate over all pairs of scanner positions, and find the largest one.
        let mut largest_dist = 0;
        for pos_a in &scanner_positions {
            for pos_b in &scanner_positions {
                let dist = pos_a
                    .0
                    .iter()
                    .zip(pos_b.0.iter())
                    .map(|(u, v)| i32::abs(*u - *v))
                    .sum();

                largest_dist = largest_dist.max(dist);
            }
        }

        println!("Part 2: largest Manhattan distance: {}", largest_dist);
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Position([i32; 3]);

impl Position {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self([x, y, z])
    }

    fn x(&self) -> i32 {
        self.0[0]
    }

    fn y(&self) -> i32 {
        self.0[1]
    }

    fn z(&self) -> i32 {
        self.0[2]
    }
}

impl Position {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScannerReport {
    beacons: Vec<Position>,
}

impl ScannerReport {
    fn rotate(&self, rot: &Rotation) -> Self {
        Self {
            beacons: self.beacons.iter().map(|b| rot.rotate(b)).collect(),
        }
    }

    fn transform(&self, transform: &Isometry3d) -> Self {
        Self {
            beacons: self
                .beacons
                .iter()
                .map(|b| transform.transform(b))
                .collect(),
        }
    }
}

/// A rotation of the axes.
/// There are 24 such rotations: 3 axes * 2 signs for the first new x-axis, 2 axes * 2 signs for
/// the new y-axis, and 1 axis and 1 sign for the new z-axis (there is no mirroring, so the
/// determinant must be == 1, which fully determines the new z-axis).
#[derive(Clone, Debug)]
struct Rotation {
    /// `src_axes[0]` contains the index of the axis which will be mapped to the new x-axis.
    src_axes: [usize; 3],
    /// `signs[0]` contains the sign of the mapping. So x_new = signs[0] * coords[src_axes[0]].
    signs: [i32; 3],
}

impl Rotation {
    const fn new(src_axes: [usize; 3], signs: [i32; 3]) -> Self {
        Self { src_axes, signs }
    }
    fn all_rotations() -> &'static [Rotation; 24] {
        #[rustfmt::skip]
        const ROTS: [Rotation; 24] = [
            // x_new = x
            Rotation::new([0, 1, 2], [ 1,  1,  1]),
            Rotation::new([0, 1, 2], [ 1, -1, -1]),
            Rotation::new([0, 2, 1], [ 1,  1, -1]),
            Rotation::new([0, 2, 1], [ 1, -1,  1]),
            // x_new = -x
            Rotation::new([0, 1, 2], [-1, -1,  1]),
            Rotation::new([0, 1, 2], [-1,  1, -1]),
            Rotation::new([0, 2, 1], [-1,  1,  1]),
            Rotation::new([0, 2, 1], [-1, -1, -1]),
            // x_new = y
            Rotation::new([1, 0, 2], [ 1, -1,  1]),
            Rotation::new([1, 0, 2], [ 1,  1, -1]),
            Rotation::new([1, 2, 0], [ 1,  1,  1]),
            Rotation::new([1, 2, 0], [ 1, -1, -1]),
            // x_new = -y
            Rotation::new([1, 0, 2], [-1,  1,  1]),
            Rotation::new([1, 0, 2], [-1, -1, -1]),
            Rotation::new([1, 2, 0], [-1, -1,  1]),
            Rotation::new([1, 2, 0], [-1,  1, -1]),
            // x_new = z
            Rotation::new([2, 0, 1], [ 1,  1,  1]),
            Rotation::new([2, 0, 1], [ 1, -1, -1]),
            Rotation::new([2, 1, 0], [ 1, -1,  1]),
            Rotation::new([2, 1, 0], [ 1,  1, -1]),
            // x_new = -z
            Rotation::new([2, 0, 1], [-1, -1,  1]),
            Rotation::new([2, 0, 1], [-1,  1, -1]),
            Rotation::new([2, 1, 0], [-1,  1,  1]),
            Rotation::new([2, 1, 0], [-1, -1, -1]),
            ];

        &ROTS
    }

    fn rotate(&self, pos: &Position) -> Position {
        Position([
            self.signs[0] * pos.0[self.src_axes[0]],
            self.signs[1] * pos.0[self.src_axes[1]],
            self.signs[2] * pos.0[self.src_axes[2]],
        ])
    }
}

#[derive(Clone, Debug)]
struct Translation([i32; 3]);

impl Translation {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self([x, y, z])
    }

    fn translate(&self, pos: &Position) -> Position {
        Position([
            self.0[0] + pos.0[0],
            self.0[1] + pos.0[1],
            self.0[2] + pos.0[2],
        ])
    }
}

#[derive(Clone, Debug)]
struct Isometry3d {
    rotation: &'static Rotation,
    translation: Translation,
}

impl Isometry3d {
    fn transform(&self, pos: &Position) -> Position {
        self.translation.translate(&self.rotation.rotate(pos))
    }
}

/// Taking `ref_scanner` as canonical coordinate frame, looks for a transform reference_T_other
/// that maps beacons in `other`'s frame to `reference`'s frame. The transform that maximizes the
/// number of beacon matches is used to return (transformed_other, num_matches).
fn find_rotation_and_beacon_matches(
    reference: &ScannerReport,
    other: &ScannerReport,
) -> Option<(Isometry3d, ScannerReport, usize)> {
    let reference_beacons: std::collections::HashSet<Position> =
        reference.beacons.iter().cloned().collect();

    let mut best_transform = None;
    let mut best_count = 0;

    'rotation_loop: for rot in Rotation::all_rotations() {
        // Rotate the `other` frame. Beyond that, we'll assume the axes are aligned and oriented
        // the same.
        let other_rot = other.rotate(rot);

        // Try matching every beacon in reference to every beacon in other, and see which pairing
        // yields the most aligned beacons.
        for beacon_ref in &reference.beacons {
            for beacon_other in &other_rot.beacons {
                let translation = Translation::new(
                    beacon_ref.x() - beacon_other.x(),
                    beacon_ref.y() - beacon_other.y(),
                    beacon_ref.z() - beacon_other.z(),
                );

                let count: usize = other_rot
                    .beacons
                    .iter()
                    .filter(|b| reference_beacons.contains(&translation.translate(b)))
                    .count();

                let transform = Isometry3d {
                    rotation: rot,
                    translation,
                };

                // Per the instructions, a match of at least 12 beacons is good enough.
                // Note that as far as I can tell 12 has no particular significance.
                if count >= 12 {
                    best_count = count;
                    best_transform = Some(transform);
                    break 'rotation_loop;
                }
            }
        }
    }

    if let Some(transform) = best_transform {
        let other_transformed = other.transform(&transform);
        Some((transform, other_transformed, best_count))
    } else {
        None
    }
}

fn parse_puzzle_input(text: &str) -> Vec<ScannerReport> {
    let mut reports = Vec::new();

    for line in text.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("--- scanner ") {
            let scanner_idx = rest.strip_suffix(" ---").unwrap().parse::<usize>().unwrap();
            assert_eq!(scanner_idx, reports.len());
            reports.push(ScannerReport {
                beacons: Vec::new(),
            });
        } else {
            let mut parts = line.split(',');
            let relpos = Position([
                parts.next().unwrap().parse::<i32>().unwrap(),
                parts.next().unwrap().parse::<i32>().unwrap(),
                parts.next().unwrap().parse::<i32>().unwrap(),
            ]);
            assert_eq!(parts.next(), None);

            reports.last_mut().unwrap().beacons.push(relpos);
        }
    }

    reports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_puzzle_input() {
        let input = r"
            --- scanner 0 ---
            0,1,3
            -3,-2,-1

            --- scanner 1 ---
            77,888,9999";

        assert_eq!(
            vec![
                ScannerReport {
                    beacons: vec![Position::new(0, 1, 3), Position::new(-3, -2, -1)],
                },
                ScannerReport {
                    beacons: vec![Position::new(77, 888, 9999)],
                },
            ],
            super::parse_puzzle_input(input)
        );
    }

    #[test]
    fn rotations_are_valid() {
        for rot in Rotation::all_rotations() {
            // All 3 axes show up.
            let mut axes = rot.src_axes.clone();
            axes.sort();
            assert_eq!(axes, [0, 1, 2]);

            // The 3 new axes.
            let mut x_new = [0i32; 3];
            x_new[rot.src_axes[0]] = rot.signs[0];
            let mut y_new = [0i32; 3];
            y_new[rot.src_axes[1]] = rot.signs[1];
            let mut z_new = [0i32; 3];
            z_new[rot.src_axes[2]] = rot.signs[2];

            // The cross-product of the first 2 axes yields the 3rd one.
            fn cross(a: &[i32; 3], b: &[i32; 3]) -> [i32; 3] {
                [
                    a[1] * b[2] - a[2] * b[1],
                    a[2] * b[0] - a[0] * b[2],
                    a[0] * b[1] - a[1] * b[0],
                ]
            }
            assert_eq!(z_new, cross(&x_new, &y_new));
        }
    }

    #[test]
    fn report_rotate() {
        // Take an example from the instructions.
        let report = ScannerReport {
            beacons: vec![
                Position::new(-1, -1, 1),
                Position::new(-2, -2, 2),
                Position::new(-3, -3, 3),
                Position::new(-2, -3, 1),
                Position::new(5, 6, -4),
                Position::new(8, 0, 7),
            ],
        };
        let rotation = Rotation::new([0, 2, 1], [-1, -1, -1]);
        let expected_report = ScannerReport {
            beacons: vec![
                Position::new(1, -1, 1),
                Position::new(2, -2, 2),
                Position::new(3, -3, 3),
                Position::new(2, -1, 3),
                Position::new(-5, 4, -6),
                Position::new(-8, -7, 0),
            ],
        };
        assert_eq!(expected_report, report.rotate(&rotation));
    }
}
