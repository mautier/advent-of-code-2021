fn main() {
    let test_file = advent_of_code::env::get_puzzle_input_path("2021-12-18.txt");
    let test_lines = std::fs::read_to_string(&test_file).unwrap();
    let numbers = parse_number_list(&test_lines);
    let summed = sum(&numbers);
    println!("Part 1: magnitude of the sum = {}", magnitude(&summed));

    let mut max_magnitude: Option<u64> = None;
    for (i, a) in numbers.iter().enumerate() {
        for b in &numbers[i + 1..] {
            let magnitude_ab = magnitude(&add(a, b));
            let magnitude_ba = magnitude(&add(b, a));
            let max = u64::max(magnitude_ab, magnitude_ba);

            if let Some(m) = max_magnitude {
                max_magnitude = Some(m.max(max));
            } else {
                max_magnitude = Some(max);
            }
        }
    }

    println!(
        "Part 2: max magnitude of any 2 additions = {}",
        max_magnitude.unwrap()
    );
}

/// A snailfish number, represented as a binary tree.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Number {
    nodes: Vec<Node>,
}

/// The index of a `Number` tree node within the array of nodes.
type NodeIdx = usize;

/// The root node of the tree is always index 0.
const ROOT: NodeIdx = 0;

/// A single node within the `Number` tree.
/// Internal nodes are `Pair`s, leaves are `Scalar`s.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Node {
    Scalar {
        parent: Option<NodeIdx>,
        val: u64,
    },
    Pair {
        parent: Option<NodeIdx>,
        left: NodeIdx,
        right: NodeIdx,
    },
}

impl Number {
    fn new_scalar(val: u64) -> Number {
        Number {
            nodes: vec![Node::Scalar { parent: None, val }],
        }
    }

    fn node(&self, idx: NodeIdx) -> &Node {
        &self.nodes[idx]
    }

    fn node_mut(&mut self, idx: NodeIdx) -> &mut Node {
        &mut self.nodes[idx]
    }
}

impl Node {
    fn parent(&self) -> Option<NodeIdx> {
        match self {
            Node::Scalar { parent, .. } => *parent,
            Node::Pair { parent, .. } => *parent,
        }
    }

    fn left_and_right(&self) -> Option<(NodeIdx, NodeIdx)> {
        match self {
            Node::Scalar { .. } => None,
            Node::Pair { left, right, .. } => Some((*left, *right)),
        }
    }

    fn scalar_value(&self) -> Option<u64> {
        match self {
            Node::Scalar { val, .. } => Some(*val),
            Node::Pair { .. } => None,
        }
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        fn display_node(
            num: &Number,
            idx: NodeIdx,
            f: &mut std::fmt::Formatter<'_>,
        ) -> Result<(), std::fmt::Error> {
            match num.node(idx) {
                Node::Scalar { val, .. } => write!(f, "{}", val),
                Node::Pair { left, right, .. } => {
                    write!(f, "[")?;
                    display_node(num, *left, f)?;
                    write!(f, ",")?;
                    display_node(num, *right, f)?;
                    write!(f, "]")?;
                    Ok(())
                }
            }
        }

        display_node(self, ROOT, f)
    }
}

fn parse_number(mut number_str: &str) -> Number {
    number_str = number_str.trim();

    let mut nodes = Vec::new();

    fn parse_node(nodes: &mut Vec<Node>, s: &mut &str, parent: Option<NodeIdx>) -> NodeIdx {
        // Make a new node.
        let idx = nodes.len();
        // For now, assume it's a scalar.
        nodes.push(Node::Scalar {
            parent: None,
            val: 0,
        });

        let first = s.chars().next().unwrap();
        if first.is_ascii_digit() {
            nodes[idx] = Node::Scalar {
                parent,
                val: first.to_digit(10).unwrap() as u64,
            };
            *s = &(*s)[1..];
        } else {
            *s = s.strip_prefix('[').unwrap();
            let left = parse_node(nodes, s, Some(idx));
            *s = s.strip_prefix(',').unwrap();
            let right = parse_node(nodes, s, Some(idx));
            *s = s.strip_prefix(']').unwrap();
            nodes[idx] = Node::Pair {
                parent,
                left,
                right,
            };
        }

        idx
    }

    let root = parse_node(&mut nodes, &mut number_str, None);
    assert_eq!(root, ROOT);
    assert!(number_str.is_empty());

    Number { nodes }
}

fn parse_number_list(numbers: &str) -> Vec<Number> {
    numbers.lines().map(parse_number).collect()
}

fn sum(numbers: &[Number]) -> Number {
    let mut result = match numbers.get(0) {
        None => return Number::new_scalar(0),
        Some(n) => n.clone(),
    };

    for other in &numbers[1..] {
        result = add(&result, other);
    }

    result
}

fn add(a: &Number, b: &Number) -> Number {
    let added = add_no_reduce(a, b);
    reduce(added)
}

fn add_no_reduce(a: &Number, b: &Number) -> Number {
    // First we join the 2 trees, by:
    // - Creating a new root.
    // - Shifting all node indices in a by 1.
    // - Shifting all node indices in b by 1 + a.len().
    let len_a = a.nodes.len();
    let len_b = b.nodes.len();
    let mut new_nodes = Vec::with_capacity(1 + len_a + len_b);
    new_nodes.push(Node::Pair {
        parent: None,
        left: 1,
        right: 1 + len_a,
    });
    for (offset, nodes) in [(1, &a.nodes), (1 + len_a, &b.nodes)] {
        new_nodes.extend(nodes.iter().map(|n| match n {
            Node::Scalar { parent, val } => Node::Scalar {
                parent: Some(parent.map(|p| p + offset).unwrap_or(ROOT)),
                val: *val,
            },
            Node::Pair {
                parent,
                left,
                right,
            } => Node::Pair {
                parent: Some(parent.map(|p| p + offset).unwrap_or(ROOT)),
                left: left + offset,
                right: right + offset,
            },
        }));
    }
    Number { nodes: new_nodes }
}

fn find_pair_to_explode(num: &Number, idx: NodeIdx, current_depth: usize) -> Option<NodeIdx> {
    match num.node(idx) {
        Node::Scalar { .. } => None,
        Node::Pair { left, right, .. } => {
            let left_is_scalar = matches!(num.node(*left), Node::Scalar { .. });
            let right_is_scalar = matches!(num.node(*right), Node::Scalar { .. });
            if current_depth >= 4 && left_is_scalar && right_is_scalar {
                Some(idx)
            } else {
                find_pair_to_explode(num, *left, current_depth + 1)
                    .or_else(|| find_pair_to_explode(num, *right, current_depth + 1))
            }
        }
    }
}

fn explode_pair(num: &mut Number, idx: NodeIdx) {
    let (idx_parent, idx_left_val, idx_right_val) = match num.node(idx) {
        Node::Scalar { .. } => panic!("BUG: can't explode a scalar."),
        Node::Pair {
            parent,
            left,
            right,
        } => (
            *parent,
            num.node(*left)
                .scalar_value()
                .expect("BUG: exploded pair must contain scalars."),
            num.node(*right)
                .scalar_value()
                .expect("BUG: exploded pair must contain scalars."),
        ),
    };

    // An array of node indices forming the path from `ROOT` to `idx`.
    let path_to_idx = {
        let mut curr = idx;
        let mut v = vec![];
        while let Some(parent) = num.node(curr).parent() {
            v.push(parent);
            curr = parent;
        }
        v.reverse();
        v
    };
    assert!(path_to_idx.len() >= 1);
    assert_eq!(*path_to_idx.first().unwrap(), ROOT);

    // Add left_val to the next scalar on the left.
    {
        let mut target_subtree = None; // The root of the subtree where we'll find our target.

        let mut current = idx;
        let mut path_pos = path_to_idx.len() - 1;
        // Go up the `path_to_idx` until we find a `current` that is a right child.
        while let Some(parent) = path_to_idx.get(path_pos) {
            let (left, right) = num
                .node(*parent)
                .left_and_right()
                .expect("BUG: all internal nodes should be pairs.");

            if left == current {
                // `current` is a left child, we must go up some more.
                current = *parent;
                path_pos = match path_pos.checked_sub(1) {
                    None => break,
                    Some(pp) => pp,
                };
            } else {
                assert_eq!(right, current);
                target_subtree = Some(left);
                break;
            }
        }

        if let Some(mut current) = target_subtree {
            let target = loop {
                match num.node(current) {
                    Node::Scalar { .. } => break current,
                    Node::Pair { right, .. } => current = *right,
                }
            };
            match num.node_mut(target) {
                Node::Scalar { val, .. } => *val += idx_left_val,
                _ => panic!("BUG: target should be a scalar."),
            }
        }
    }

    // Add right_val to the next scalar on the right.
    {
        let mut target_subtree = None; // The root of the subtree where we'll find our target.

        let mut current = idx;
        let mut path_pos = path_to_idx.len() - 1;
        // Go up the `path_to_idx` until we find a `current` that is a left child.
        while let Some(parent) = path_to_idx.get(path_pos) {
            let (left, right) = num
                .node(*parent)
                .left_and_right()
                .expect("BUG: all internal nodes should be pairs.");

            if right == current {
                // `current` is a right child, we must go up some more.
                current = *parent;
                path_pos = match path_pos.checked_sub(1) {
                    None => break,
                    Some(pp) => pp,
                };
            } else {
                assert_eq!(left, current);
                target_subtree = Some(right);
                break;
            }
        }

        if let Some(mut current) = target_subtree {
            let target = loop {
                match num.node(current) {
                    Node::Scalar { .. } => break current,
                    Node::Pair { left, .. } => current = *left,
                }
            };
            match num.node_mut(target) {
                Node::Scalar { val, .. } => *val += idx_right_val,
                _ => panic!("BUG: target should be a scalar."),
            }
        }
    }

    // Replace idx with a 0 scalar.
    // NOTE: for simplicity, we leave the left and right scalar nodes in the list of nodes.
    //       They become dangling nodes, as no pair links to them.
    *num.node_mut(idx) = Node::Scalar {
        parent: idx_parent,
        val: 0,
    };
}

fn find_scalar_to_split(num: &Number, idx: NodeIdx) -> Option<NodeIdx> {
    match num.node(idx) {
        Node::Scalar { val, .. } => {
            if *val >= 10 {
                Some(idx)
            } else {
                None
            }
        }
        Node::Pair { left, right, .. } => {
            find_scalar_to_split(num, *left).or_else(|| find_scalar_to_split(num, *right))
        }
    }
}
fn split_scalar(num: &mut Number, idx: NodeIdx) {
    let (original_parent, original_value) = match num.node(idx) {
        Node::Scalar { val, .. } if *val <= 9 => panic!("Can't split a number < 10"),
        Node::Scalar { parent, val } => (*parent, *val),
        Node::Pair { .. } => panic!("Can't split a pair!"),
    };

    let left = num.nodes.len();
    let right = num.nodes.len() + 1;

    num.nodes.push(Node::Scalar {
        parent: Some(idx),
        val: original_value / 2,
    });
    num.nodes.push(Node::Scalar {
        parent: Some(idx),
        val: original_value - original_value / 2,
    });
    *num.node_mut(idx) = Node::Pair {
        parent: original_parent,
        left,
        right,
    };
}

fn reduce(mut num: Number) -> Number {
    loop {
        if let Some(idx) = find_pair_to_explode(&num, ROOT, 0) {
            explode_pair(&mut num, idx);
            continue;
        }
        if let Some(idx) = find_scalar_to_split(&num, ROOT) {
            split_scalar(&mut num, idx);
            continue;
        }
        // Nothing happened, stop here.
        break;
    }

    num
}

fn magnitude(num: &Number) -> u64 {
    fn node_magn(num: &Number, idx: NodeIdx) -> u64 {
        match num.node(idx) {
            Node::Scalar { val, .. } => *val as u64,
            Node::Pair { left, right, .. } => {
                3 * node_magn(num, *left) + 2 * node_magn(num, *right)
            }
        }
    }

    node_magn(num, ROOT)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse() {
        use super::{Node, Number};
        assert_eq!(
            super::parse_number("7"),
            Number {
                nodes: vec![Node::Scalar {
                    parent: None,
                    val: 7
                }]
            }
        );
        assert_eq!(
            super::parse_number("[3,5]"),
            Number {
                nodes: vec![
                    Node::Pair {
                        parent: None,
                        left: 1,
                        right: 2
                    },
                    Node::Scalar {
                        parent: Some(0),
                        val: 3
                    },
                    Node::Scalar {
                        parent: Some(0),
                        val: 5
                    }
                ]
            }
        );
        assert_eq!(
            super::parse_number("[[2,3],5]"),
            Number {
                nodes: vec![
                    Node::Pair {
                        parent: None,
                        left: 1,
                        right: 4
                    },
                    Node::Pair {
                        parent: Some(0),
                        left: 2,
                        right: 3
                    },
                    Node::Scalar {
                        parent: Some(1),
                        val: 2
                    },
                    Node::Scalar {
                        parent: Some(1),
                        val: 3
                    },
                    Node::Scalar {
                        parent: Some(0),
                        val: 5
                    },
                ]
            }
        );
    }

    #[test]
    fn test_display() {
        for input in [
            "[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]",
            "[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]",
            "[[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]",
            "[[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]",
            "[7,[5,[[3,8],[1,4]]]]",
            "[[2,[2,2]],[8,[8,1]]]",
            "[2,9]",
            "[1,[[[9,3],9],[[9,0],[0,7]]]]",
            "[[[5,[7,4]],7],1]",
            "[[[[4,2],2],6],[8,7]]",
        ] {
            // Test the round-trip: str -> Number -> str.
            assert_eq!(input, format!("{}", super::parse_number(input)));
        }
    }

    #[test]
    fn test_add_no_reduce() {
        use super::{Node, Number};
        assert_eq!(
            super::add_no_reduce(
                &Number {
                    nodes: vec![
                        Node::Pair {
                            parent: None,
                            left: 1,
                            right: 2
                        },
                        Node::Scalar {
                            parent: Some(0),
                            val: 2
                        },
                        Node::Scalar {
                            parent: Some(0),
                            val: 5
                        }
                    ]
                },
                &Number {
                    nodes: vec![
                        Node::Pair {
                            parent: None,
                            left: 1,
                            right: 2
                        },
                        Node::Scalar {
                            parent: Some(0),
                            val: 7
                        },
                        Node::Scalar {
                            parent: Some(0),
                            val: 9
                        }
                    ]
                },
            ),
            Number {
                nodes: vec![
                    Node::Pair {
                        parent: None,
                        left: 1,
                        right: 4
                    },
                    Node::Pair {
                        parent: Some(0),
                        left: 2,
                        right: 3
                    },
                    Node::Scalar {
                        parent: Some(1),
                        val: 2
                    },
                    Node::Scalar {
                        parent: Some(1),
                        val: 5
                    },
                    Node::Pair {
                        parent: Some(0),
                        left: 5,
                        right: 6
                    },
                    Node::Scalar {
                        parent: Some(4),
                        val: 7
                    },
                    Node::Scalar {
                        parent: Some(4),
                        val: 9
                    },
                ]
            }
        );
    }

    #[test]
    fn test_add() {
        let input_01 = "[1,1]\n[2,2]\n[3,3]\n[4,4]";
        let result_01 = "[[[[1,1],[2,2]],[3,3]],[4,4]]";

        let input_02 = "[1,1]\n[2,2]\n[3,3]\n[4,4]\n[5,5]";
        let result_02 = "[[[[3,0],[5,3]],[4,4]],[5,5]]";

        let input_03 = "[1,1]\n[2,2]\n[3,3]\n[4,4]\n[5,5]\n[6,6]";
        let result_03 = "[[[[5,0],[7,4]],[5,5]],[6,6]]";

        let input_04 = r"[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]
[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]
[[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]
[[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]
[7,[5,[[3,8],[1,4]]]]
[[2,[2,2]],[8,[8,1]]]
[2,9]
[1,[[[9,3],9],[[9,0],[0,7]]]]
[[[5,[7,4]],7],1]
[[[[4,2],2],6],[8,7]]";
        let result_04 = "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]";

        let input_05 = r"[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
[[[5,[2,8]],4],[5,[[9,9],0]]]
[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
[[[[5,4],[7,7]],8],[[8,3],8]]
[[9,3],[[9,9],[6,[4,9]]]]
[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]";
        let result_05 = "[[[[6,6],[7,6]],[[7,7],[7,0]]],[[[7,7],[7,7]],[[7,8],[9,9]]]]";

        for (input, expected) in [
            (input_01, result_01),
            (input_02, result_02),
            (input_03, result_03),
            (input_04, result_04),
            (input_05, result_05),
        ] {
            let numbers = super::parse_number_list(input);
            let summed = super::sum(&numbers);
            let summed_string = summed.to_string();
            assert_eq!(expected, summed_string);
        }
    }

    #[test]
    fn test_magnitude() {
        let number_and_magnitude = [
            ("[[1,2],[[3,4],5]]", 143),
            ("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]", 1384),
            ("[[[[1,1],[2,2]],[3,3]],[4,4]]", 445),
            ("[[[[3,0],[5,3]],[4,4]],[5,5]]", 791),
            ("[[[[5,0],[7,4]],[5,5]],[6,6]]", 1137),
            (
                "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]",
                3488,
            ),
            (
                "[[[[6,6],[7,6]],[[7,7],[7,0]]],[[[7,7],[7,7]],[[7,8],[9,9]]]]",
                4140,
            ),
        ];

        for (number_string, expected_magnitude) in number_and_magnitude {
            let number = super::parse_number(number_string);
            let magnitude = super::magnitude(&number);
            assert_eq!(expected_magnitude, magnitude);
        }
    }
}
