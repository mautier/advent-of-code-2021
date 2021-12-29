fn main() {
    for test_file in [
        "2021-12-12.sample_01.txt",
        "2021-12-12.sample_02.txt",
        "2021-12-12.sample_03.txt",
        "2021-12-12.txt",
    ] {
        println!("---------------- {} ----------------", test_file);
        let input_path = advent_of_code::env::get_puzzle_input_path(test_file);
        let lines = advent_of_code::iter::line_iter_from_file(&input_path);
        let graph = parse_cave_graph(lines);

        let num_paths_no_revisits =
            count_all_paths_from_start_to_end(&graph, false /* no revisits */);
        println!("Part 1: number of paths: {}", num_paths_no_revisits);

        let num_paths_with_revisits =
            count_all_paths_from_start_to_end(&graph, true /* allow 1 revisit */);
        println!("Part 2: number of paths: {}", num_paths_with_revisits);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum CaveSize {
    Small,
    Large,
}

/// An integer index assigned to a particular cave.
/// We'll use this to represent the graph as arrays / matrices indexed by cave ids.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct CaveId(usize);

/// Representation of the graph of caves. Keeps track of which caves are connect to which,
/// and their sizes.
struct CaveGraph {
    /// For each cave, its size.
    sizes: Vec<CaveSize>,
    /// For each cave, the list of its neighbors.
    neighbors: Vec<Vec<CaveId>>,
}

const START: CaveId = CaveId(0);
const END: CaveId = CaveId(1);

fn count_all_paths_from_start_to_end(graph: &CaveGraph, allow_revisits: bool) -> usize {
    // We'll perform a depth-first search through the graph, keeping track of which caves we've
    // visited. Only small caves get marked as visited (and cannot be visited again).

    fn count_paths_depth_first(
        graph: &CaveGraph,
        current: CaveId,
        allow_revisits: bool,
        times_visited: &mut [u8],
    ) -> usize {
        if current == END {
            return 1;
        }
        if graph.sizes[current.0] == CaveSize::Small {
            assert!(times_visited[current.0] <= 1);
            times_visited[current.0] += 1;
        }

        // Visit all our unvisited neighbors
        // (and make an allowance for visited neighbors if allow_revisits is true).
        let mut num_paths = 0;
        for neigh_id in &graph.neighbors[current.0] {
            if times_visited[neigh_id.0] == 0 {
                num_paths += count_paths_depth_first(
                    graph,
                    *neigh_id,
                    allow_revisits,
                    times_visited,
                );
            } else if allow_revisits && *neigh_id != START {
                assert!(graph.sizes[neigh_id.0] == CaveSize::Small);
                assert!(times_visited[neigh_id.0] == 1);
                num_paths += count_paths_depth_first(
                    graph,
                    *neigh_id,
                    false, // we used our only allowed revisit
                    times_visited,
                );
            }
        }

        // Return to our caller, resetting our visited state to its previous value, so that future
        // explored paths can go through this cave too.
        if graph.sizes[current.0] == CaveSize::Small {
            times_visited[current.0] -= 1;
        }
        num_paths
    }

    let mut times_visited = vec![0u8; graph.sizes.len()];
    count_paths_depth_first(graph, START, allow_revisits, &mut times_visited)
}

fn parse_cave_graph(lines: impl Iterator<Item = String>) -> CaveGraph {
    let mut name_to_id = std::collections::HashMap::new();

    // Create the start and end caves.
    name_to_id.insert("start".to_string(), START.0);
    name_to_id.insert("end".to_string(), END.0);
    let mut sizes = vec![CaveSize::Small; 2];
    let mut neighbors = vec![Vec::new(), Vec::new()];

    for line in lines {
        let mut parts = line.split('-');
        let names = [parts.next().unwrap(), parts.next().unwrap()];
        assert_eq!(None, parts.next());

        // Assign IDs if needed, taking care to resize `sizes` and `neighbors` appropriately.
        let mut ids = [CaveId(0), CaveId(0)];
        for (name, id) in names.iter().zip(ids.iter_mut()) {
            // Look up the id, or create a new one.
            *id = CaveId(*name_to_id.entry(name.to_string()).or_insert(sizes.len()));

            if id.0 >= sizes.len() {
                sizes.push(determine_cave_size(name));
                neighbors.push(Vec::new());
            }
        }

        neighbors[ids[0].0].push(ids[1]);
        neighbors[ids[1].0].push(ids[0]);

        if sizes[ids[0].0] == CaveSize::Large && sizes[ids[1].0] == CaveSize::Large {
            panic!(
                "Invalid graph, it contains infinitely many paths (2 connected large caves): {} and {}",
                names[0], names[1]
            );
        }
    }

    CaveGraph { sizes, neighbors }
}

fn determine_cave_size(name: &str) -> CaveSize {
    if name.chars().next().expect("Empty name!").is_lowercase() {
        CaveSize::Small
    } else {
        CaveSize::Large
    }
}
