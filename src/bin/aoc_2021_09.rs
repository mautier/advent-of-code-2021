fn main() {
    let input_path = advent_of_code::env::get_puzzle_input_path("2021-12-09.txt");
    let lines = advent_of_code::iter::line_iter_from_file(&input_path);
    let img = parse_input_lines_as_image(lines);

    // Part 1: look for pixels smaller than all their neighbors.
    let mut sum_of_low_point_risks = 0u32;

    // Part 2: we're looking for basins, ie connected components of pixels where
    // the smoke can "flow down" towards a low point.
    //
    // It appears that the image was generated such that basins are always separated by
    // a wall of 9s. This means that the only crests (pixels at the top between 2 basins)
    // are those 9s, which according to the instructions should not be counted.
    // This means that 2 adjacent pixels belong to the same basin iif they are not 9s.
    //
    // We'll compute the connected components and their sizes in a single pass using a
    // union find / disjoint set data structure.
    // We'll initially create one set per pixel: each pixel is its own standalone basin.
    // Pixel (row, col) corresponds to basin/set id row * width + col.
    let mut union_find = UnionFind::with_size(img.height as usize * img.width as usize);
    let pixel_id = |row: u16, col: u16| -> SetId { row as u32 * img.width as u32 + col as u32 };

    for row in 0..img.height {
        for col in 0..img.width {
            // Define the neighborhood to look at, clipping it to the valid image parts.
            let neighboring_rows = std::ops::Range {
                start: row.saturating_sub(1),
                end: u16::min(row + 2, img.height),
            };
            let neighboring_cols = std::ops::Range {
                start: col.saturating_sub(1),
                end: u16::min(col + 2, img.width),
            };

            let this_pixel = *img.pixel(row, col);

            // 9s are the maximum height; therefore they cannot ever be low points.
            if this_pixel == 9 {
                continue;
            }

            let mut is_low_point = true;
            'neighborhood_loop: for neigh_row in neighboring_rows.clone() {
                for neigh_col in neighboring_cols.clone() {
                    // Skip the center pixel.
                    if neigh_row == row && neigh_col == col {
                        continue;
                    }

                    if *img.pixel(neigh_row, neigh_col) <= this_pixel {
                        is_low_point = false;
                        break 'neighborhood_loop;
                    }
                }
            }
            if is_low_point {
                sum_of_low_point_risks += this_pixel as u32 + 1;
            }

            // For part 2: merge this pixel's basin with its neighbors to the left and up (future
            // loop iterations will take care of the neighbors to the right and down).
            if row > 0 && *img.pixel(row - 1, col) != 9 {
                union_find.merge(pixel_id(row, col), pixel_id(row - 1, col));
            }
            if col > 0 && *img.pixel(row, col - 1) != 9 {
                union_find.merge(pixel_id(row, col), pixel_id(row, col - 1));
            }
        }
    }

    // Part 2 post-processing: iterate over all the sets, find the roots, and get their tree
    // sizes, and keep only the top 3.
    let mut largest_basins = [0u32; 3];
    for set in union_find.sets {
        if set.parent.is_some() {
            // Skip internal tree nodes.
            continue;
        }
        let s = set.size_if_root;
        if s >= largest_basins[0] {
            largest_basins[2] = largest_basins[1];
            largest_basins[1] = largest_basins[0];
            largest_basins[0] = s;
        } else if s >= largest_basins[1] {
            largest_basins[2] = largest_basins[1];
            largest_basins[1] = s;
        } else if s >= largest_basins[2] {
            largest_basins[2] = s;
        }
    }

    println!("Part 1: sum of low point risks: {}", sum_of_low_point_risks);
    println!(
        "Part 2: product of 3 largest basin sizes ({:?}): {}",
        largest_basins,
        largest_basins.into_iter().product::<u32>()
    );
}

struct Image<T> {
    height: u16,
    width: u16,
    /// Row-major buffer of pixel data, of size height * width.
    data: Vec<T>,
}

impl<T> Image<T> {
    fn pixel(&self, row: u16, col: u16) -> &T {
        &self.data[row as usize * self.width as usize + col as usize]
    }
}

fn parse_input_lines_as_image(lines: impl Iterator<Item = String>) -> Image<u8> {
    let mut width = 0;
    let mut height = 0;
    let mut data = Vec::new();

    for line in lines {
        height += 1;
        if width == 0 {
            width = line.len() as u16;
        }
        assert_eq!(line.len(), width as usize);

        data.extend(line.bytes().map(|b| (b - b'0') as u8));
    }

    assert_eq!(data.len(), height as usize * width as usize);
    Image {
        height,
        width,
        data,
    }
}

/// A data structure for keeping track of various sets / subsets, as trees.
/// https://en.wikipedia.org/wiki/Union_find
///
/// Each item is created as a single-element `Set`. Sets can be merged by making them part of the
/// same tree. This tree structure is defined using the `parent` attribute of each `Set`. Within a
/// tree, only the root has no parent.
struct UnionFind {
    /// The various disjoint sets of which we're keeping track.
    sets: Vec<Set>,
}

/// An index representing a set in the UnionFind's array of sets.
type SetId = u32;

/// Information about a set.
#[derive(Clone, Debug)]
struct Set {
    /// The parent set that contains this one, or None if this set is a root.
    parent: Option<SetId>,
    /// When this set is a root (no parent), contains the total size of this set and its children.
    /// For non-root sets, ignore this value, it is not representative of anything.
    size_if_root: u32,
}

impl UnionFind {
    fn with_size(size: usize) -> Self {
        UnionFind {
            sets: vec![
                Set {
                    parent: None,
                    size_if_root: 1
                };
                size
            ],
        }
    }

    /// Returns the corresponding set.
    fn get(&self, s: SetId) -> &Set {
        &self.sets[s as usize]
    }

    /// Returns the corresponding set for mutation.
    fn get_mut(&mut self, s: SetId) -> &mut Set {
        &mut self.sets[s as usize]
    }

    /// Merges 2 sets.
    fn merge(&mut self, a: SetId, b: SetId) {
        let root_a = self.get_root_and_compress_path(a);
        let root_b = self.get_root_and_compress_path(b);

        if root_a == root_b {
            // Already part of the same set.
            return;
        }

        // To merge the 2 trees, make root_a the parent of root_b.
        // To achieve an excellent runtime complexity, we need to make sure that the size / depth
        // of the resulting tree doesn't grow too much. This is achieved by simply picking as new
        // root the set with the largest size. So if root_b is actually the largest, swap a and b.
        let (new_root, new_child) =
            if self.get(root_a).size_if_root >= self.get(root_b).size_if_root {
                (root_a, root_b)
            } else {
                (root_b, root_a)
            };

        let additional_items = self.get(new_child).size_if_root;
        self.get_mut(new_root).size_if_root += additional_items;
        self.get_mut(new_child).parent = Some(new_root);
    }

    /// Returns the root set for `s`.
    ///
    /// Performs path-compression: all sets on the way from `s` to the root are re-parented to have
    /// the root as immediate parent (this makes the tree very flat, and helps achieve the
    /// ridiculously good complexity bounds that union-find data structures have).
    fn get_root_and_compress_path(&mut self, s: SetId) -> SetId {
        // First, go up to the root.
        let mut current = s;
        while let Some(parent) = self.get(current).parent {
            current = parent;
        }
        let root = current;

        // Next, re-parent all the nodes on the way, starting from `s` again.
        current = s;
        while let Some(parent) = self.get(current).parent {
            self.get_mut(current).parent = Some(root);
            current = parent;
        }

        root
    }
}
