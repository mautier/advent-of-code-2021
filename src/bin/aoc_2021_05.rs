#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Point {
    x: u16,
    y: u16,
}

#[derive(Copy, Clone, Debug)]
struct Line {
    a: Point,
    b: Point,
}

impl Line {
    fn is_axis_aligned(&self) -> bool {
        self.a.x == self.b.x || self.a.y == self.b.y
    }
}

fn parse_puzzle_input(text: impl Iterator<Item = String>) -> Vec<Line> {
    let mut lines = Vec::new();
    for l in text {
        let mut parts = l.split(" -> ");
        let mut a = parts
            .next()
            .unwrap()
            .split(',')
            .map(|n| n.parse::<u16>().unwrap());
        let mut b = parts
            .next()
            .unwrap()
            .split(',')
            .map(|n| n.parse::<u16>().unwrap());

        lines.push(Line {
            a: Point {
                x: a.next().unwrap(),
                y: a.next().unwrap(),
            },
            b: Point {
                x: b.next().unwrap(),
                y: b.next().unwrap(),
            },
        });
    }

    lines
}

fn get_max_xy(lines: &[Line]) -> Point {
    let mut max_point = Point { x: 0, y: 0 };
    for l in lines {
        for p in [l.a, l.b] {
            max_point.x = max_point.x.max(p.x);
            max_point.y = max_point.y.max(p.y);
        }
    }
    max_point
}

#[derive(Clone)]
struct Image {
    #[allow(dead_code)]
    height: u16,
    width: u16,
    data: Vec<u16>,
}

impl Image {
    fn new(height: u16, width: u16) -> Self {
        Image {
            height,
            width,
            data: vec![0; height as usize * width as usize],
        }
    }

    fn pixel_mut(&mut self, p: Point) -> &mut u16 {
        &mut self.data[p.y as usize * self.width as usize + p.x as usize]
    }
}

fn draw_line(line: &Line, img: &mut Image) {
    let dx = (line.b.x as i16 - line.a.x as i16).signum();
    let dy = (line.b.y as i16 - line.a.y as i16).signum();

    let mut pt = line.a;
    while pt != line.b {
        *img.pixel_mut(pt) += 1;
        pt.x = (pt.x as i16 + dx).try_into().unwrap();
        pt.y = (pt.y as i16 + dy).try_into().unwrap();
    }
    // Draw the final point.
    *img.pixel_mut(pt) += 1;
}

fn count_pixels_with_2_or_more_lines(img: &Image) -> usize {
    img.data.iter().copied().filter(|&count| count >= 2).count()
}

fn main() {
    let input_path = advent_of_code::env::get_puzzle_input_path("2021-12-05.txt");
    let lines = parse_puzzle_input(advent_of_code::iter::line_iter_from_file(&input_path));
    let max_point = get_max_xy(&lines[..]);
    let img = Image::new(max_point.y + 1, max_point.x + 1);

    // Consider only axis-aligned lines.
    {
        let mut img = img.clone();
        for line in &lines {
            if !line.is_axis_aligned() {
                continue;
            }
            draw_line(line, &mut img);
        }

        let num_overlaps = count_pixels_with_2_or_more_lines(&img);
        println!("Axis-aligned lines: {} pixels with >= 2 lines crossing.", num_overlaps);
    }

    // Consider all lines
    {
        let mut img = img;
        for line in &lines {
            draw_line(line, &mut img);
        }

        let num_overlaps = count_pixels_with_2_or_more_lines(&img);
        println!("All lines: {} pixels with >= 2 lines crossing.", num_overlaps);
    }
}
