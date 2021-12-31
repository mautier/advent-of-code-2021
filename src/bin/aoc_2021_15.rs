use advent_of_code::colormap::{LinearColorScale, Rgb};

fn main() {
    let mut logdir = None;
    for arg in std::env::args() {
        if let Some(path) = arg.strip_prefix("--log-images-to=") {
            use std::str::FromStr;
            logdir = Some(std::path::PathBuf::from_str(path).unwrap());
            println!("Will log progress images to dir: {:?}", logdir.as_ref().unwrap());
        }
    }

    for test_file in ["2021-12-15.sample.txt", "2021-12-15.txt"] {
        println!("--------------- {} ---------------", test_file);
        let input_path = advent_of_code::env::get_puzzle_input_path(test_file);
        let image = parse_input_image(advent_of_code::iter::line_iter_from_file(&input_path));

        let optimal_path_cost = find_optimal_path(&image, None);
        println!("Part 1: optimal path cost: {}", optimal_path_cost);

        let tiled_image = expand_tile_into_full_image(&image);
        let mut log = logdir
            .as_ref()
            .map(|_| ExplorationLog::new(tiled_image.height, tiled_image.width));
        let optimal_path_cost = find_optimal_path(&tiled_image, log.as_mut());
        println!("Part 2: optimal path cost: {}", optimal_path_cost);

        if let Some(log) = log {
            generate_viz_images(
                &log,
                logdir.as_ref().unwrap(),
                test_file.strip_suffix(".txt").unwrap(),
            );
        }
    }
}

struct Image<T> {
    height: u16,
    width: u16,
    /// Linear buffer of pixels, in row-major order.
    data: Vec<T>,
}

impl<T> Image<T> {
    fn pixel(&self, row: u16, col: u16) -> &T {
        &self.data[row as usize * self.width as usize + col as usize]
    }
    fn pixel_mut(&mut self, row: u16, col: u16) -> &mut T {
        &mut self.data[row as usize * self.width as usize + col as usize]
    }
}

/// Search for a minimal cost path between the top-left corner and the bottom right corner.
///
/// Uses an implementation of A* (https://en.wikipedia.org/wiki/A*_search_algorithm).
///
/// Optionally records the order in which pixels are visited (as well as their cost) to a log.
fn find_optimal_path(image: &Image<u8>, mut log: Option<&mut ExplorationLog>) -> usize {
    // A heuristic that estimates the distance to the bottom right.
    // For A* to find the optimal path first, this heuristic must never over-estimate the
    // distance (it must be an admissible heuristic).
    // To do so, we simply assume that all costs till the end are 1.
    let estimated_dist_to_end = |row: u16, col: u16| -> usize {
        (image.height - 1 - row) as usize + (image.width - 1 - col) as usize
    };

    // Using the heuristic "distance to the end", compute the heuristic "total cost".
    let estimated_total_cost = |cost_so_far: usize, row: u16, col: u16| -> usize {
        let est_remaining_cost = estimated_dist_to_end(row, col);
        cost_so_far + est_remaining_cost
    };

    // The image of visited pixels.
    let mut visited = Image {
        height: image.height,
        width: image.width,
        data: vec![false; image.height as usize * image.width as usize],
    };

    // The open set of points, ie the pixels we haven't visited and could visit in the next step,
    // starting from a previously visited pixel.
    // This is a max heap, meaning max-cost first. To flip this we'll use Reverse.
    use std::cmp::Reverse;
    let mut to_visit = std::collections::BinaryHeap::new();
    to_visit.push(Reverse(VisitCandidate {
        row: 0,
        col: 0,
        cost_so_far: 0,
        estimated_total_cost: estimated_total_cost(0, 0, 0),
    }));

    // Scratch buffer used for iterating over the valid neighbors. Cleared on every iteration.
    let mut neighbors = Vec::with_capacity(4);

    while let Some(cand) = to_visit.pop() {
        let cand = cand.0; // Peel the Reverse.

        if let Some(log) = &mut log {
            log.explore(cand.row, cand.col, cand.cost_so_far);
        }

        if cand.row == image.height - 1 && cand.col == image.width - 1 {
            return cand.cost_so_far;
        }
        *visited.pixel_mut(cand.row, cand.col) = true;

        // Add all 4 neighbors to the open set.
        neighbors.clear();
        if 0 < cand.row && !visited.pixel(cand.row - 1, cand.col) {
            neighbors.push((cand.row - 1, cand.col));
        }
        if cand.row + 1 < image.height && !visited.pixel(cand.row + 1, cand.col) {
            neighbors.push((cand.row + 1, cand.col));
        }
        if 0 < cand.col && !visited.pixel(cand.row, cand.col - 1) {
            neighbors.push((cand.row, cand.col - 1));
        }
        if cand.col + 1 < image.width && !visited.pixel(cand.row, cand.col + 1) {
            neighbors.push((cand.row, cand.col + 1));
        }

        for &(r, c) in &neighbors {
            let cost_so_far = cand.cost_so_far + *image.pixel(r, c) as usize;
            to_visit.push(Reverse(VisitCandidate {
                row: r,
                col: c,
                cost_so_far,
                estimated_total_cost: estimated_total_cost(cost_so_far, r, c),
            }));
        }
    }

    panic!("BUG: should be unreachable");
}

#[derive(Clone, Debug, Eq)]
struct VisitCandidate {
    row: u16,
    col: u16,
    cost_so_far: usize,
    estimated_total_cost: usize,
}

impl std::cmp::Ord for VisitCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.estimated_total_cost.cmp(&other.estimated_total_cost)
    }
}

impl std::cmp::PartialOrd for VisitCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for VisitCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.cost_so_far == other.cost_so_far
    }
}

fn parse_input_image(lines: impl Iterator<Item = String>) -> Image<u8> {
    let mut height = 0;
    let mut width = 0;
    let mut data = Vec::new();

    for line in lines {
        height += 1;
        if width == 0 {
            width = line.as_bytes().len() as u16;
        }
        assert_eq!(width, line.as_bytes().len() as u16);

        data.extend(line.chars().map(|c| (c as u32 - '0' as u32) as u8));
    }

    Image {
        height,
        width,
        data,
    }
}

fn expand_tile_into_full_image(tile: &Image<u8>) -> Image<u8> {
    let mut img = Image {
        height: tile.height * 5,
        width: tile.width * 5,
        data: vec![0u8; tile.height as usize * 5 * tile.width as usize * 5],
    };

    for row_in_img in 0..img.height {
        for col_in_img in 0..img.width {
            let row_in_tile = row_in_img % tile.height;
            let col_in_tile = col_in_img % tile.width;

            let tile_index_rows = row_in_img / tile.height;
            let tile_index_cols = col_in_img / tile.width;
            let extra_risk = (tile_index_rows + tile_index_cols) as u8;

            // This is a little more awkward than just "%10" because there is no risk level 0.
            // So we remap the risk to [0,8], compute the extra risk %9, and then remap back to
            // [1,9].
            *img.pixel_mut(row_in_img, col_in_img) =
                1 + (*tile.pixel(row_in_tile, col_in_tile) - 1 + extra_risk) % 9;
        }
    }

    img
}

struct ExplorationLog {
    image_height: u16,
    image_width: u16,
    /// The visited pixels, as (row, col, cost) tuples.
    visits: Vec<(u16, u16, usize)>,
}

impl ExplorationLog {
    fn new(image_height: u16, image_width: u16) -> Self {
        Self {
            image_height,
            image_width,
            visits: Vec::new(),
        }
    }

    fn explore(&mut self, row: u16, col: u16, cost: usize) {
        self.visits.push((row, col, cost));
    }
}

fn generate_viz_images(log: &ExplorationLog, dir: &std::path::Path, name_prefix: &str) {
    let get_path = |step: usize| -> std::path::PathBuf {
        let mut path = std::path::PathBuf::new();
        path.push(dir);
        path.push(format!("{}.step_{:05}.ppm", name_prefix, step));
        path
    };

    let min_cost = 0;
    let max_cost = *log.visits.iter().map(|(_row, _col, cost)| cost).max().unwrap();
    let cs = LinearColorScale { min: min_cost as f32, max: max_cost as f32 };

    // The output image that we'll progressively modify.
    // Starts out as all black.
    let mut img = Image {
        height: log.image_height,
        width: log.image_width,
        data: vec![Rgb::new(0, 0, 0); log.image_height as usize * log.image_width as usize],
    };

    // There are (likely) way too many steps to save an image for each.
    // Instead, we'll aim for a N second clip at M fps.
    let target_len_s = 20.0;
    let fps = 30.0;
    let target_len_frames = target_len_s * fps;
    // We'll save a frame every x:
    let save_step = usize::max(1, log.visits.len() / (target_len_frames as usize));

    let mut num_saved_images = 0;
    for (step, (row, col, cost)) in log.visits.iter().enumerate() {
        *img.pixel_mut(*row, *col) = cs.map(*cost as f32);

        if step % save_step == 0 || step + 1 == log.visits.len() {
            let path = get_path(num_saved_images);
            save_image_as_ppm(&img, &path).expect("Failed to save image");
            num_saved_images += 1;
        }
    }
    println!("Saved {} images!", num_saved_images);
}

/// Saves an RGB image as a Portable PixMap (PPM). See https://en.wikipedia.org/wiki/Netpbm for
/// details.
fn save_image_as_ppm(img: &Image<Rgb>, path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Write;
    let mut writer = std::io::BufWriter::new(std::fs::File::create(path)?);

    // Magic number for a binary PPM.
    writer.write_all(b"P6\n")?;
    // Width, then height, as ASCII.
    writeln!(&mut writer, "{} {}", img.width, img.height)?;
    // The maximum value for each color.
    writeln!(&mut writer, "255")?;
    // Now the binary pixel data.
    for px in &img.data {
        writer.write_all(&[px.r(), px.g(), px.b()])?;
    }
    writer.flush()?;

    Ok(())
}