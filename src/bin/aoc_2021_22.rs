use std::ops::Range;

fn main() {
    for test_file in [
        "2021-12-22.sample_1.txt",
        "2021-12-22.sample_2.txt",
        "2021-12-22.txt",
    ] {
        println!("------------------ {} ------------------", test_file);
        let test_path = advent_of_code::env::get_puzzle_input_path(test_file);
        let steps = parse_puzzle_input(&std::fs::read_to_string(&test_path).unwrap());

        let mut grid = ReactorGridImage::new(State::Off);
        for step in &steps {
            grid.set_cuboid_to(&step.xrange, &step.yrange, &step.zrange, step.set_to);
        }
        let num_on = grid.data.iter().filter(|&&s| s == State::On).count();
        println!(
            "Part 1: number of on cubes in [-50,50]x[-50,50]x[-50,50]: {}",
            num_on
        );

        let mut cuboid_grid = ReactorGridCuboids::new();
        for step in &steps {
            let area = Cuboid {
                xrange: step.xrange.clone(),
                yrange: step.yrange.clone(),
                zrange: step.zrange.clone(),
            };
            if step.set_to == State::On {
                cuboid_grid.turn_on(&area);
            } else {
                cuboid_grid.turn_off(&area);
            }
        }
        println!(
            "Part 2: number of on cubes in the full space: {}",
            cuboid_grid.count_on()
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    On,
    Off,
}

#[derive(Clone, Debug)]
struct Step {
    set_to: State,
    xrange: Range<i32>,
    yrange: Range<i32>,
    zrange: Range<i32>,
}

/// A dense representation of the reactor grid, limited to the range [-50, 50]^3.
struct ReactorGridImage {
    data: Vec<State>,
}

impl ReactorGridImage {
    fn new(initial_state: State) -> Self {
        Self {
            data: vec![initial_state; 101 * 101 * 101],
        }
    }

    fn set_cuboid_to(
        &mut self,
        xrange: &Range<i32>,
        yrange: &Range<i32>,
        zrange: &Range<i32>,
        state: State,
    ) {
        let xrange = clip_range(xrange, -50, 51);
        let yrange = clip_range(yrange, -50, 51);
        let zrange = clip_range(zrange, -50, 51);

        for x in xrange.clone() {
            let x = (x + 50) as usize;
            for y in yrange.clone() {
                let y = (y + 50) as usize;
                let zstart = (zrange.start + 50) as usize;
                let zend = (zrange.end + 50) as usize;

                let istart = (x * 101 + y) * 101 + zstart;
                let iend = istart + (zend - zstart);
                for px in &mut self.data[istart..iend] {
                    *px = state;
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Cuboid {
    xrange: Range<i32>,
    yrange: Range<i32>,
    zrange: Range<i32>,
}

/// An intersection of cuboids.
enum Intersection {
    /// The intersection is empty.
    Empty,
    /// The intersectionis non-empty, and it is composed of:
    /// - An intersection cuboid.
    /// - A number of non-intersecting cuboids.
    NonEmpty(Cuboid, Vec<Cuboid>),
}

impl Cuboid {
    fn is_empty(&self) -> bool {
        self.xrange.is_empty() || self.yrange.is_empty() || self.zrange.is_empty()
    }

    fn volume(&self) -> usize {
        (self.xrange.end - self.xrange.start) as usize
            * (self.yrange.end - self.yrange.start) as usize
            * (self.zrange.end - self.zrange.start) as usize
    }

    /// Splits `self` based on the `splitter` cuboid.
    fn split_using(&self, splitter: &Self) -> Intersection {
        let i = Cuboid {
            xrange: clip_range(&splitter.xrange, self.xrange.start, self.xrange.end),
            yrange: clip_range(&splitter.yrange, self.yrange.start, self.yrange.end),
            zrange: clip_range(&splitter.zrange, self.zrange.start, self.zrange.end),
        };

        if i.is_empty() {
            return Intersection::Empty;
        }

        let mut parts = Vec::new();

        let lesser_xrange = self.xrange.start..i.xrange.start;
        if !lesser_xrange.is_empty() {
            parts.push(Cuboid {
                xrange: lesser_xrange,
                yrange: self.yrange.clone(),
                zrange: self.zrange.clone(),
            });
        }
        let upper_xrange = i.xrange.end..self.xrange.end;
        if !upper_xrange.is_empty() {
            parts.push(Cuboid {
                xrange: upper_xrange,
                yrange: self.yrange.clone(),
                zrange: self.zrange.clone(),
            });
        }
        // Now set xrange to i.xrange for all parts.

        let lesser_yrange = self.yrange.start..i.yrange.start;
        if !lesser_yrange.is_empty() {
            parts.push(Cuboid {
                xrange: i.xrange.clone(),
                yrange: lesser_yrange,
                zrange: self.zrange.clone(),
            });
        }
        let upper_yrange = i.yrange.end..self.yrange.end;
        if !upper_yrange.is_empty() {
            parts.push(Cuboid {
                xrange: i.xrange.clone(),
                yrange: upper_yrange,
                zrange: self.zrange.clone(),
            });
        }
        // Now set yrange to i.yrange for al parts.

        let lesser_zrange = self.zrange.start..i.zrange.start;
        if !lesser_zrange.is_empty() {
            parts.push(Cuboid {
                xrange: i.xrange.clone(),
                yrange: i.yrange.clone(),
                zrange: lesser_zrange,
            });
        }
        let upper_zrange = i.zrange.end..self.zrange.end;
        if !upper_zrange.is_empty() {
            parts.push(Cuboid {
                xrange: i.xrange.clone(),
                yrange: i.yrange.clone(),
                zrange: upper_zrange,
            });
        }

        Intersection::NonEmpty(i, parts)
    }
}

/// A sparse representation of the reactor grid, using a list of mutually disjoint cuboids
/// containing on cells.
struct ReactorGridCuboids {
    /// A set of mutually disjoint cuboids that are on.
    on_cuboids: Vec<Cuboid>,
}

impl ReactorGridCuboids {
    fn new() -> Self {
        Self {
            on_cuboids: Vec::new(),
        }
    }

    fn turn_off(&mut self, area: &Cuboid) {
        let mut new_on = Vec::with_capacity(self.on_cuboids.len());

        for on in self.on_cuboids.drain(..) {
            match on.split_using(area) {
                // This step has no impact on this cuboid.
                Intersection::Empty => new_on.push(on),
                // One part of the cuboid will be turned off, and the rest will be unaffected.
                Intersection::NonEmpty(_intersection, mut remaining) => new_on.append(&mut remaining),
            }
        }

        self.on_cuboids = new_on;
    }

    fn turn_on(&mut self, area: &Cuboid) {
        // Initially, we have 1 big area to turn on.
        // But if it happens to overlap with other areas that are already on, to prevent
        // overlapping cuboids we'll have to remove the intersection from the new area, and add the
        // remaining non-intersecting pieces.
        // We also record starting from which index we need to check for intersections: if we know
        // there's no intersections up to index i, no need to check again after we split a cuboid.
        let mut areas_to_turn_on: Vec<(usize, Cuboid)> = vec![(0usize, area.clone())];
        let mut non_intersecting_areas_to_add = Vec::new();

        while let Some((check_from_idx, area)) = areas_to_turn_on.pop() {
            let mut intersects_something = false;
            for (idx, on) in self.on_cuboids.iter().enumerate().skip(check_from_idx) {
                match area.split_using(on) {
                    Intersection::Empty => {}
                    Intersection::NonEmpty(_intersection, remaining) => {
                        // All the non-intersecting pieces need to be processed independently now.
                        // They can start from the next index, since we know nothing before that
                        // will intersect.
                        areas_to_turn_on.extend(remaining.into_iter().map(|r| (idx + 1, r)));
                        intersects_something = true;
                        break;
                    }
                }
            }

            if !intersects_something {
                non_intersecting_areas_to_add.push(area);
            }
        }

        self.on_cuboids.append(&mut non_intersecting_areas_to_add);
    }

    fn count_on(&self) -> usize {
        self.on_cuboids.iter().map(|c| c.volume()).sum::<usize>()
    }
}

fn clip_range(r: &Range<i32>, min: i32, max: i32) -> Range<i32> {
    Range {
        start: r.start.min(max).max(min),
        end: r.end.min(max).max(min),
    }
}

fn parse_puzzle_input(text: &str) -> Vec<Step> {
    fn parse_inclusive_range(s: &mut &str) -> Range<i32> {
        let end = s.find(',').unwrap_or(s.len());
        let mut parts = (*s)[..end].split("..");
        let r = Range {
            start: parts.next().unwrap().parse::<i32>().unwrap(),
            end: parts.next().unwrap().parse::<i32>().unwrap() + 1,
        };
        assert_eq!(parts.next(), None);
        *s = &(*s)[end..];
        r
    }

    let mut steps = Vec::new();

    for line in text.lines() {
        let (set_to, mut range_str) = if let Some(rest) = line.strip_prefix("on ") {
            (State::On, rest)
        } else if let Some(rest) = line.strip_prefix("off ") {
            (State::Off, rest)
        } else {
            panic!("Invalid line: {}", line)
        };

        range_str = range_str.strip_prefix("x=").unwrap();
        let xrange = parse_inclusive_range(&mut range_str);
        range_str = range_str.strip_prefix(",y=").unwrap();
        let yrange = parse_inclusive_range(&mut range_str);
        range_str = range_str.strip_prefix(",z=").unwrap();
        let zrange = parse_inclusive_range(&mut range_str);

        steps.push(Step {
            set_to,
            xrange,
            yrange,
            zrange,
        });
    }

    steps
}
