const SAMPLE_INPUT_PART_1: &str = "\
#############
#...........#
###B#C#B#D###
  #A#D#C#A#
  #########";

const SAMPLE_INPUT_PART_2: &str = "\
#############
#...........#
###B#C#B#D###
  #D#C#B#A#
  #D#B#A#C#
  #A#D#C#A#
  #########";

const PUZZLE_INPUT_PART_1: &str = "\
#############
#...........#
###B#D#C#A###
  #C#D#B#A#
  #########";

const PUZZLE_INPUT_PART_2: &str = "\
#############
#...........#
###B#D#C#A###
  #D#C#B#A#
  #D#B#A#C#
  #C#D#B#A#
  #########";

fn main() {
    for (name, input) in [
        ("sample (part 1)", SAMPLE_INPUT_PART_1),
        ("the real deal (part 1)", PUZZLE_INPUT_PART_1),
        ("sample (part 2)", SAMPLE_INPUT_PART_2),
        ("the real deal (part 2)", PUZZLE_INPUT_PART_2),
    ] {
        println!("---------------------- {} ----------------------", name);
        let burrow = parse_puzzle_input(input);
        let (min_energy, states) =
            find_minimum_energy_shuffling(&burrow).expect("Failed to sort burrow");
        println!(
            "Part 1: minimum energy required to properly sort: {}",
            min_energy
        );
        for (i, b) in states.iter().enumerate() {
            println!("Step {}:\n{}", i, b);
        }
    }
}

/// A representation of the burrow. Each space in the burrow is assigned an index, and stored in a
/// simple array.
/// The spaces just outside of rooms will never be occupied, so they are not assigned indices.
/// ```text
/// ###################################
/// # 0| 1|  | 2|  | 3|  | 4|  | 5|  6#
/// ######| 7|##| 9|##|11|##|13|#######
///       | 8|  |10|  |12|  |14|
///       ####  ####  ####  ####
/// ```
///
/// There are always 4 rooms, with indices:
/// - Room A: 7, 8, ..., 7 + slots_per_room - 1
/// - Room B: 7 + slots_per_room, ...
/// - Room C: 7 + 2 * slots_per_room, ...
/// - Room D: 7 + 3 * slots_per_room, ...
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Burrow {
    slots_per_room: u8,
    spaces: Vec<Space>,
}

/// A binary representation of a burrow, used for perfect hashing.
type BinaryBurrow = u128;

impl Burrow {
    /// Check a room for a free spot to move into.
    /// Returns None if there is no free spot, or if the room contains any amphipod that does not
    /// belong there.
    fn empty_space_in_my_room(&self, me: Amphipod) -> Option<usize> {
        let range = self.room_range(me);

        // Look for the deepest free slot (without obstacles).
        let mut free = None;
        for idx in range.rev() {
            match self.spaces[idx] {
                Space::Empty => free = Some(free.unwrap_or(idx)),
                Space::Occupied(x) if x == me => free = None,
                // There's another type of amphipod in the room, can't go in there at all.
                Space::Occupied(_) => return None,
            }
        }

        free
    }

    /// Returns the index of the top spot in this amphipod type's room.
    fn room_top_index(&self, amphi: Amphipod) -> usize {
        7 + amphi.room_index() * self.slots_per_room as usize
    }

    /// Returns the range of spaces inside a room.
    fn room_range(&self, amphi: Amphipod) -> std::ops::RangeInclusive<usize> {
        let top = self.room_top_index(amphi);
        let bot = top + (self.slots_per_room - 1) as usize;
        top..=bot
    }

    /// Returns true if all the spots in the room are occupied by the appropriate amphipod type.
    fn is_amphi_room_correctly_filled(&self, amphi: Amphipod) -> bool {
        self.room_range(amphi)
            .all(|idx| self.spaces[idx] == Space::Occupied(amphi))
    }

    /// Moves the contents of space `from` to space `to` (the destination must be empty!).
    fn move_from_to(&self, from: usize, to: usize) -> Self {
        let mut new = self.clone();
        let what = new.spaces[from];
        assert!(what != Space::Empty);
        new.spaces[from] = Space::Empty;
        assert!(new.spaces[to] == Space::Empty);
        new.spaces[to] = what;
        new
    }

    /// Returns true if the burrow is fully sorted, ie all rooms are filled with the right
    /// amphipods.
    fn is_sorted(&self) -> bool {
        use Amphipod::*;
        [A, B, C, D]
            .iter()
            .all(|amphi| self.is_amphi_room_correctly_filled(*amphi))
    }

    /// Represents this burrow in a compressed binary format. This is an injective mapping from the
    /// set of burrows to the set of u128.
    fn binary_repr(&self) -> BinaryBurrow {
        use Amphipod::*;
        // 5 states per space => 3 bits per space.
        let num_bits = 3 * self.spaces.len();
        assert!(num_bits <= 128);

        let mut repr = 0u128;
        for s in self.spaces.iter() {
            let three_bits: u8 = match s {
                Space::Empty => 0,
                Space::Occupied(A) => 1,
                Space::Occupied(B) => 2,
                Space::Occupied(C) => 3,
                Space::Occupied(D) => 4,
            };
            repr = (repr << 3) | (three_bits as u128);
        }
        repr
    }
}

/// The range of spaces that constitutes the hallway.
const HALLWAY_SPACES: std::ops::Range<usize> = 0..7;
/// The index of the top space in room A. All other spaces have indices >= this one.
const ROOM_A_TOP: usize = 7;

/// Returns true if a space index corresponds to the hallway.
fn is_hallway(space: usize) -> bool {
    HALLWAY_SPACES.contains(&space)
}

/// The amphipod types.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Amphipod {
    A,
    B,
    C,
    D,
}

impl Amphipod {
    /// Returns the cost of having this particular amphipod type take this many steps.
    fn cost(self, num_steps: u32) -> Cost {
        use Amphipod::*;
        let cost_per_step = match self {
            A => 1,
            B => 10,
            C => 100,
            D => 1000,
        };
        num_steps * cost_per_step
    }

    /// Returns the index of room for this amphipod type.
    fn room_index(self) -> usize {
        use Amphipod::*;
        match self {
            A => 0,
            B => 1,
            C => 2,
            D => 3,
        }
    }
}

/// The contents of a single space in the burrow.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Space {
    Empty,
    Occupied(Amphipod),
}

impl Space {
    fn from_ascii_byte(b: u8) -> Self {
        use Amphipod::*;
        match b {
            b'.' => Space::Empty,
            b'A' => Space::Occupied(A),
            b'B' => Space::Occupied(B),
            b'C' => Space::Occupied(C),
            b'D' => Space::Occupied(D),
            _ => panic!("Invalid space char: {}", b as char),
        }
    }

    fn content(self) -> Option<Amphipod> {
        if let Space::Occupied(x) = self {
            Some(x)
        } else {
            None
        }
    }
}

type Cost = u32;

/// Looks for a minimum-cost way to sort the burrow.
/// If successful, returns both the minimum achieved cost, and a vector of the various burrow
/// states that between the initial state and the fully sorted state.
fn find_minimum_energy_shuffling(burrow: &Burrow) -> Option<(Cost, Vec<Burrow>)> {
    // We'll search for a path in the graph of burrow states, using the A* path searching
    // algorithm.
    // We'll keep track of the incurred cost, and use a heuristic for the remaining cost.
    // The heuristic is both admissible and consistent.

    #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
    struct State {
        estimated_total_cost: Cost,
        cost_so_far: Cost,
        burrow: Burrow,
        prev_burrow: Option<Burrow>,
    }

    use std::cmp::Reverse;
    let mut to_visit = std::collections::BinaryHeap::new();
    to_visit.push(Reverse(State {
        estimated_total_cost: heuristical_cost_to_finish(&burrow),
        cost_so_far: 0,
        burrow: burrow.clone(),
        prev_burrow: None,
    }));

    // A hash map of: visited burrow state => (cost to reach, prev burrow state if any).
    let mut visited = std::collections::HashMap::<BinaryBurrow, Option<Burrow>>::new();

    while let Some(state) = to_visit.pop() {
        // Peel the Reverse.
        let state = state.0;

        if state.burrow.is_sorted() {
            // Found the final state!
            // Retrace our steps to see the sequence of moves that led us here.
            let mut burrows = vec![state.burrow.clone()];
            visited.insert(state.burrow.binary_repr(), state.prev_burrow);
            let mut bin_repr = state.burrow.binary_repr();
            while let Some(Some(prev)) = visited.get(&bin_repr) {
                burrows.push(prev.clone());
                bin_repr = prev.binary_repr();
            }
            burrows.reverse();
            return Some((state.cost_so_far, burrows));
        }

        let entry = visited.entry(state.burrow.binary_repr());
        if matches!(entry, std::collections::hash_map::Entry::Occupied(_)) {
            // Already in the visited set.
            continue;
        }
        entry.or_insert(state.prev_burrow);

        // Add all possible moves to `to_visit`.
        for (cost_of_move, new_burrow) in possible_moves(&state.burrow) {
            let cost_so_far = state.cost_so_far + cost_of_move;
            let estimated_total_cost = cost_so_far + heuristical_cost_to_finish(&new_burrow);
            to_visit.push(Reverse(State {
                estimated_total_cost,
                cost_so_far: state.cost_so_far + cost_of_move,
                burrow: new_burrow,
                prev_burrow: Some(state.burrow.clone()),
            }));
        }
    }

    None
}

/// Returns a lower bound of the cost required to finish shuffling the burrow.
/// The actual cost to fully sort the burrow is guaranteed to be larger or equal, meaning this
/// heuristic is admissible.
/// Additionally, the heuristic is consistent: est_cost(A) <= cost(move) + est_cost(A & move),
/// which guarantees that the first path A* finds to the final state will be an optimal path.
fn heuristical_cost_to_finish(burrow: &Burrow) -> Cost {
    let mut cost = 0;

    // Move all amphipods in the hallway to their room's top space, ignoring any obstacles.
    // (using the top space yields a lower cost, which is required here as we must not
    // over-estimate the cost)
    for space_idx in HALLWAY_SPACES.clone() {
        let amphi = if let Space::Occupied(x) = burrow.spaces[space_idx] {
            x
        } else {
            continue;
        };
        let top = burrow.room_top_index(amphi);
        let path = steps_along_path(space_idx, top, burrow.slots_per_room);
        cost += amphi.cost(path.num_steps);
    }

    use Amphipod::*;
    for room_type in [A, B, C, D] {
        for space_idx in burrow.room_range(room_type) {
            let amphi = if let Space::Occupied(x) = burrow.spaces[space_idx] {
                x
            } else {
                continue;
            };
            if room_type == amphi {
                // Already in the right room.
                continue;
            }
            // Not in the right room, try to walk to ours.
            let top = burrow.room_top_index(amphi);
            let num_steps = steps_between_rooms(space_idx, top);
            cost += amphi.cost(num_steps);
        }
    }

    cost
}

/// Returns all the possible moves that can be made starting from a particular burrow state.
/// This enforces the constraints from the instructions (eg an amphipod can only go from hallway
/// into its own room, and only if that room has no amphipods of the wrong type).
fn possible_moves(burrow: &Burrow) -> impl Iterator<Item = (Cost, Burrow)> {
    let mut moves = Vec::new();

    // Start by looking at amphipods in the hallway.
    for space_idx in HALLWAY_SPACES.clone() {
        let amphi = if let Space::Occupied(amphi) = burrow.spaces[space_idx] {
            amphi
        } else {
            continue;
        };
        // `space` is now non-empty.

        // From the hallway, an amphipod can only go into its own room.
        if let Some(destination_idx) = burrow.empty_space_in_my_room(amphi) {
            if let Some(steps) =
                steps_along_path_with_obstacle_check(burrow, space_idx, destination_idx)
            {
                let new_burrow = burrow.move_from_to(space_idx, destination_idx);
                moves.push((amphi.cost(steps), new_burrow));
            }
        }
    }

    // Now look at the rooms.
    use Amphipod::*;
    for room_type in [A, B, C, D] {
        // Check if the room contains an amphipod of the wrong type.
        let room_contains_wrong_amphi =
            burrow.spaces[burrow.room_range(room_type)]
                .iter()
                .any(|sp| match sp {
                    Space::Empty => true,
                    Space::Occupied(x) => *x != room_type,
                });

        if !room_contains_wrong_amphi {
            // The room is only populated by amphipods (maybe 0) that belong here, so nothing to
            // do.
            continue;
        }

        // Loop for the first occupied slot from the top. That's the only amphipod that can do
        // anything (ie maybe go into the hallway).
        for space_idx in burrow.room_range(room_type) {
            let amphi = if let Space::Occupied(amphi) = burrow.spaces[space_idx] {
                amphi
            } else {
                continue;
            };

            // 2 possibilities:
            // - This is the right room for this amphipod, but it's blocking another type of
            //   amphipod so it must move first.
            // - This is not the right room, so the amphipod must move.
            moves.extend(HALLWAY_SPACES.clone().filter_map(|destination_idx| {
                let steps =
                    steps_along_path_with_obstacle_check(burrow, space_idx, destination_idx)?;
                let new_burrow = burrow.move_from_to(space_idx, destination_idx);
                Some((amphi.cost(steps), new_burrow))
            }));
        }
    }

    moves.into_iter()
}

/// Determines the path between 2 points in the burrow, one in the hallway, one in a room, and
/// checks if the path is actually clear of obstacles.
fn steps_along_path_with_obstacle_check(burrow: &Burrow, from: usize, to: usize) -> Option<u32> {
    if burrow.spaces[to] != Space::Empty {
        return None;
    }
    let path = steps_along_path(from, to, burrow.slots_per_room);

    for i in path.hallway_steps.chain(path.room_steps) {
        if i == from {
            continue;
        }
        if let Space::Occupied(_) = burrow.spaces[i] {
            return None;
        }
    }
    Some(path.num_steps)
}

/// A path between some spot in the hallway to some spot in a room.
/// The path is not ordered in any particular way, it simply yields the space indices that must be
/// traversed at some point.
struct HallwayToRoomPath {
    hallway_steps: std::ops::Range<usize>,
    room_steps: std::ops::Range<usize>,
    /// The total number of steps that must taken, taking into account the empty space at the top
    /// of each room.
    num_steps: u32,
}

/// Determines the path between 2 points in the burrow, one in the hallway, and one in a room.
/// There is no guarantee that the path is actually free of obstacles.
fn steps_along_path(mut from: usize, mut to: usize, slots_per_room: u8) -> HallwayToRoomPath {
    // (from->to) and (to->from) are identical when it comes to number of steps and checking for
    // obstructions.
    // We'll therefore canonicalize the input to (from hallway, to room).
    if !is_hallway(from) {
        std::mem::swap(&mut from, &mut to);
    }

    // Index of the room (A-0, B-1, C-2, D-3).
    let room_idx = (to - ROOM_A_TOP) / slots_per_room as usize;

    // Get the 2 adjacent spaces at the room entrance.
    let (top_left, top_right) = (room_idx + 1, room_idx + 2);

    let hallway_steps = if from <= top_left {
        from..(top_left + 1)
    } else {
        top_right..(from + 1)
    };

    let num_steps_room = (to - ROOM_A_TOP) % slots_per_room as usize;
    let room_steps = (to - num_steps_room)..(to + 1);

    let num_steps_hallway = {
        let mut n = 0;
        for idx in (hallway_steps.start + 1)..hallway_steps.end {
            if 2 <= idx && idx <= 5 {
                n += 2;
            } else {
                n += 1;
            }
        }
        n
    };

    HallwayToRoomPath {
        hallway_steps,
        room_steps,
        num_steps: num_steps_hallway as u32 + 2 + num_steps_room as u32,
    }
}

/// Returns the minimum number of steps between spaces in two rooms.
fn steps_between_rooms(mut from: usize, mut to: usize) -> u32 {
    // Ensure that from <= to
    if from > to {
        std::mem::swap(&mut from, &mut to);
    }

    if from == to {
        return 0;
    } else if from % 2 == 1 && from + 1 == to {
        // Same room, different spaces.
        return 1;
    }

    let mut steps = 0;
    if from % 2 == 0 {
        // from is at the bottom, replace it by the top.
        steps += 1;
        from -= 1;
    }
    if to % 2 == 0 {
        // Same for to.
        steps += 1;
        to -= 1;
    }

    let num_rooms_to_travel = (to - from) / 2;
    // 1 step to exit the room into the hallway, then 2 steps per room to travel, then 1 step to
    // enter the room.
    steps += 1 + 2 * num_rooms_to_travel + 1;

    steps as u32
}

impl std::fmt::Display for Burrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(
            f,
            "{}{}.{}.{}.{}.{}{}",
            self.spaces[0],
            self.spaces[1],
            self.spaces[2],
            self.spaces[3],
            self.spaces[4],
            self.spaces[5],
            self.spaces[6]
        )?;
        let spr = self.slots_per_room as usize;
        for row in 0..spr {
            let a = self.spaces[7 + row + 0 * spr];
            let b = self.spaces[7 + row + 1 * spr];
            let c = self.spaces[7 + row + 2 * spr];
            let d = self.spaces[7 + row + 3 * spr];
            writeln!(f, "  {} {} {} {}", a, b, c, d)?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Burrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use std::fmt::Write;
        use Amphipod::*;
        match self {
            Space::Empty => f.write_char('.'),
            Space::Occupied(A) => f.write_char('A'),
            Space::Occupied(B) => f.write_char('B'),
            Space::Occupied(C) => f.write_char('C'),
            Space::Occupied(D) => f.write_char('D'),
        }
    }
}

fn parse_puzzle_input(text: &str) -> Burrow {
    let mut lines = text.lines();
    assert_eq!(lines.next().unwrap(), "#############");

    // For each space, as indexed in `Burrow`, the corresponding char in the input.
    let mut space_rows = Vec::new();

    let hallway = lines.next().unwrap();
    match hallway.as_bytes() {
        [b'#', c0, c1, b'.', c2, b'.', c3, b'.', c4, b'.', c5, c6, b'#'] => {
            space_rows.push(vec![*c0, *c1, *c2, *c3, *c4, *c5, *c6]);
        }
        _ => panic!("Invalid hallway line: {}", hallway),
    };

    loop {
        let room_line = lines.next().unwrap();

        if let Some(_) = room_line.strip_prefix("  #########") {
            // Reached the end of the room slots.
            break;
        }

        match room_line.as_bytes() {
            [_, _, b'#', a, b'#', b, b'#', c, b'#', d, b'#', ..] => {
                space_rows.push(vec![*a, *b, *c, *d]);
            }
            _ => panic!("Invalid room line: {}", room_line),
        };
    }

    let slots_per_room = space_rows.len() - 1;
    assert!(slots_per_room > 0);
    assert!(slots_per_room < 256);

    let mut spaces = Vec::with_capacity(7 + 4 * slots_per_room);

    // Fill in the hallway.
    for b in &space_rows[0] {
        spaces.push(Space::from_ascii_byte(*b));
    }

    // For each room, fill in the slots.
    for room_idx in 0..4 {
        for slot in 0..slots_per_room {
            spaces.push(Space::from_ascii_byte(space_rows[1 + slot][room_idx]));
        }
    }

    Burrow {
        slots_per_room: slots_per_room as u8,
        spaces,
    }
}

#[cfg(test)]
mod tests {
    // The moves that yield the optimal solution for the part 1 sample board.
    const SAMPLE_OPTIMAL_MOVES: &[(usize, usize)] = &[
        (11, 2),
        (9, 3),
        (3, 11),
        (10, 3),
        (2, 10),
        (7, 2),
        (2, 9),
        (13, 4),
        (14, 5),
        (4, 14),
        (3, 13),
        (5, 7),
    ];

    /// Manually replays the optimal solution, to make sure that we estimate its cost correctly.
    #[test]
    fn replay_optimal_sample_solution() {
        let mut burrow = super::parse_puzzle_input(super::SAMPLE_INPUT_PART_1);
        let mut total_cost = 0;
        for &(from, to) in SAMPLE_OPTIMAL_MOVES {
            let num_steps = match super::steps_along_path_with_obstacle_check(&burrow, from, to) {
                None => panic!(
                    "Step from {} to {} failed. Current burrow:\n{}",
                    from, to, burrow
                ),
                Some(x) => x,
            };
            let cost = burrow.spaces[from].content().unwrap().cost(num_steps);
            burrow = burrow.move_from_to(from, to);
            total_cost += cost;
        }
        assert!(burrow.is_sorted());
        assert_eq!(total_cost, 12521);
    }

    /// Solve the part 1 sample problem backwards (starting from a solved board), step by step.
    #[test]
    fn solve_sample_step_by_step() {
        let mut burrow = super::parse_puzzle_input(
            "\
#############
#...........#
###A#B#C#D###
  #A#B#C#D#
  #########",
        );

        let (min_energy, _states) = super::find_minimum_energy_shuffling(&burrow).unwrap();
        assert_eq!(min_energy, 0);

        let mut total_cost = 0;
        for &(from, to) in SAMPLE_OPTIMAL_MOVES.iter().rev() {
            // We're going backwards, so flip from and to.
            let (from, to) = (to, from);

            let num_steps = super::steps_along_path_with_obstacle_check(&burrow, from, to).unwrap();
            total_cost += burrow.spaces[from].content().unwrap().cost(num_steps);
            burrow = burrow.move_from_to(from, to);

            let (min_energy, _states) = match super::find_minimum_energy_shuffling(&burrow) {
                None => panic!(
                    "Failed to find move ({}, {}). Burrow:\n{}",
                    to, from, burrow
                ),
                Some(x) => x,
            };
            assert_eq!(min_energy, total_cost);
        }

        assert_eq!(total_cost, 12521);
    }

    #[test]
    fn steps_along_path() {
        let path = super::steps_along_path(0, super::ROOM_A_TOP + 2, 2);
        assert_eq!(path.hallway_steps, 0..3);
        assert_eq!(path.room_steps, 9..10);
        assert_eq!(path.num_steps, 5);
        assert_eq!(super::Amphipod::B.cost(path.num_steps), 50);

        let path = super::steps_along_path(1, super::ROOM_A_TOP + 4 + 3, 4);
        assert_eq!(path.hallway_steps, 1..3);
        assert_eq!(path.room_steps, 11..15);
        assert_eq!(path.num_steps, 7);
        assert_eq!(super::Amphipod::B.cost(path.num_steps), 70);

        let path = super::steps_along_path(4, 10, 2);
        assert_eq!(path.hallway_steps, 3..5);
        assert_eq!(path.room_steps, 9..11);
        assert_eq!(path.num_steps, 5);

        let path = super::steps_along_path(6, 12, 4);
        assert_eq!(path.hallway_steps, 3..7);
        assert_eq!(path.room_steps, 11..13);
        assert_eq!(path.num_steps, 8);
    }
}
