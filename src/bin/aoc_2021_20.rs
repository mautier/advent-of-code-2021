use advent_of_code::image::Image;

fn main() {
    for test_file in ["2021-12-20.sample.txt", "2021-12-20.txt"] {
        println!("------------------ {} ------------------", test_file);
        let test_path = advent_of_code::env::get_puzzle_input_path(test_file);
        let (enhancer, mut inf_img) = parse_puzzle_input(&test_path);

        for _ in 0..2 {
            inf_img = enhancer.do_the_thing(&inf_img);
        }
        // If we're currently padding with 1, then there's infinity many lit pixels!
        assert_eq!(inf_img.pad_with, 0);
        let num_lit: usize = inf_img.img.data.iter().filter(|&&p| p == 1).count();
        println!(
            "Part 1: after 2 enhancement iterations, {} pixels are lit.",
            num_lit
        );

        for _ in 2..50 {
            inf_img = enhancer.do_the_thing(&inf_img);
        }
        assert_eq!(inf_img.pad_with, 0);
        let num_lit: usize = inf_img.img.data.iter().filter(|&&p| p == 1).count();
        println!(
            "Part 2: after 50 enhancement iterations, {} pixels are lit.",
            num_lit
        );
    }
}

struct ImageEnhancer {
    // A size 512 array mapping every u9 to 0 or 1.
    map: Vec<u8>,
}

struct InfiniteImage {
    img: Image<u8>,
    pad_with: u8,
}

impl ImageEnhancer {
    fn map(&self, u9: u16) -> u8 {
        self.map[u9 as usize]
    }

    fn do_the_thing(&self, img: &InfiniteImage) -> InfiniteImage {
        // The output crop is expanded by 1 pixel on all sides compared to the original.
        let out_height = img.img.height + 2;
        let out_width = img.img.width + 2;
        let mut out_img = Image {
            height: out_height,
            width: out_width,
            data: vec![0u8; out_height as usize * out_width as usize],
        };

        for out_r in 0..(out_img.height as i32) {
            for out_c in 0..(out_img.width as i32) {
                let mut u9 = 0u16;

                // Iterate over the neighborhood.
                for out_nr in (out_r - 1)..=(out_r + 1) {
                    for out_nc in (out_c - 1)..=(out_c + 1) {
                        let img_nr = out_nr - 1;
                        let img_nc = out_nc - 1;

                        let val: u16 = if img_nr < 0
                            || (img.img.height as i32) <= img_nr
                            || img_nc < 0
                            || (img.img.width as i32) <= img_nc
                        {
                            img.pad_with as u16
                        } else {
                            *img.img.pixel(img_nr as u16, img_nc as u16) as u16
                        };

                        assert!(val <= 1);
                        u9 = (u9 << 1) | val;
                    }
                }

                let new_px = self.map(u9);
                assert!(new_px <= 1);
                *out_img.pixel_mut(out_r as u16, out_c as u16) = new_px;
            }
        }

        InfiniteImage {
            img: out_img,
            pad_with: if img.pad_with == 0 {
                self.map(0)
            } else {
                self.map(0b1_1111_1111)
            },
        }
    }
}

fn parse_puzzle_input(path: &std::path::Path) -> (ImageEnhancer, InfiniteImage) {
    let reader = std::io::BufReader::new(std::fs::File::open(path).expect("Failed to open file"));
    use std::io::BufRead;
    let mut lines = reader.lines().map(|res| res.unwrap());

    let enhancer_map: Vec<u8> = lines
        .next()
        .unwrap()
        .bytes()
        .map(|b| {
            if b == b'.' {
                0u8
            } else if b == b'#' {
                1
            } else {
                panic!("Invalid char: {}", b)
            }
        })
        .collect();
    assert_eq!(enhancer_map.len(), 512);
    let enhancer = ImageEnhancer { map: enhancer_map };

    let mut height = 0;
    let mut width = 0;
    let mut pixels = Vec::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        height += 1;
        if width == 0 {
            width = line.len() as u16;
        }
        assert_eq!(width as usize, line.len());
        pixels.extend(line.bytes().map(|b| {
            if b == b'.' {
                0u8
            } else if b == b'#' {
                1
            } else {
                panic!("Invalid char: {}", b)
            }
        }));
    }
    assert_eq!(height as usize * width as usize, pixels.len());

    let img = Image {
        height,
        width,
        data: pixels,
    };
    let inf_img = InfiniteImage { img, pad_with: 0 };

    (enhancer, inf_img)
}
