use rand::prelude::*;
use std::io::{stdout, Write};
use std::ops::AddAssign;
use std::{fs::File, io::BufWriter};

type Cell = u8;

mod cell {
    pub const FULL: u8 = 0b00001111;
    pub const UP: u8 = 0b0000_1000;
    pub const RIGHT: u8 = 0b0000_0100;
    pub const DOWN: u8 = 0b0000_0010;
    pub const LEFT: u8 = 0b0000_0001;
    pub const EMPTY: u8 = 0b0000_0000;
    pub const BOUNDARY: u8 = 0b0001_0000;
}

struct RGB8 {
    red: u8,
    green: u8,
    blue: u8,
}

impl RGB8 {
    const BLACK: Self = RGB8::new(0, 0, 0);
    const BOUNDARY: Self = RGB8::new(0, 0, 0);

    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn as_array(self) -> [u8; 3] {
        [self.red, self.green, self.blue]
    }
}

impl AddAssign for RGB8 {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            red: self.red + other.red,
            green: self.green + other.green,
            blue: self.blue + other.blue,
        }
    }
}

struct Grid {
    grid: Vec<Cell>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![cell::EMPTY; width * height];
        Self {
            grid,
            width,
            height,
        }
    }

    pub fn get(&self, x: isize, y: isize) -> Cell {
        if (x < 0) || (x as usize >= self.width) || (y < 0) || (y as usize >= self.height) {
            cell::BOUNDARY
        } else {
            let index = y as usize * self.width + x as usize;
            self.grid[index]
        }
    }

    pub fn set(&mut self, x: isize, y: isize, value: Cell) {
        assert!(
            (x >= 0) && ((x as usize) < self.width) && (y >= 0) && ((y as usize) < self.height)
        );
        let index = y as usize * self.width + x as usize;
        self.grid[index] = value;
    }

    pub fn fill_boundary(&mut self, x_min: isize, y_min: isize, width: usize, height: usize) {
        for y in y_min..y_min + height as isize {
            for x in x_min..x_min + width as isize {
                self.set(x, y, cell::BOUNDARY);
            }
        }
    }

    pub fn fill_region(
        &mut self,
        x_min: isize,
        y_min: isize,
        width: usize,
        height: usize,
        probability: f64,
    ) {
        for y in y_min..y_min + height as isize {
            for x in x_min..x_min + width as isize {
                let mut rng = thread_rng();
                let n = rng.gen_bool(probability);
                let s = rng.gen_bool(probability);
                let e = rng.gen_bool(probability);
                let w = rng.gen_bool(probability);
                let value = (n as u8) << 3 | (s as u8) << 2 | (e as u8) << 1 | (w as u8);
                self.set(x, y, value);
            }
        }
    }
}

fn propagate_grid(grid: &Grid, next_grid: &mut Grid) {
    for y in 0..grid.height as isize {
        for x in 0..grid.width as isize {
            let up = grid.get(x, y + 1) & cell::DOWN;
            let right = grid.get(x + 1, y) & cell::LEFT;
            let down = grid.get(x, y - 1) & cell::UP;
            let left = grid.get(x - 1, y) & cell::RIGHT;
            let mut next_state = up | right | down | left | (grid.get(x, y) & cell::BOUNDARY);
            next_state = resolve_collisions(next_state);
            next_grid.set(x, y, next_state);
        }
    }
}

fn resolve_collisions(cell_value: u8) -> u8 {
    if cell_value & cell::BOUNDARY == 0 {
        match cell_value {
            0b0101 => 0b1010,
            0b1010 => 0b0101,
            other => other,
        }
    } else {
        let up = cell_value & cell::UP;
        let right = cell_value & cell::RIGHT;
        let down = cell_value & cell::DOWN;
        let left = cell_value & cell::LEFT;
        up >> 2 | right >> 2 | down << 2 | left << 2 | cell::BOUNDARY
    }
}

fn count_particles_in_block(
    grid: &Grid,
    block_x: usize,
    block_y: usize,
    downscale: usize,
) -> Option<usize> {
    let mut particles = 0;
    for cell_x in downscale * block_x..downscale * block_x + downscale - 1 {
        for cell_y in downscale * block_y..downscale * block_y + downscale - 1 {
            let cell_in_block = grid.get(cell_x as isize, cell_y as isize);
            if cell_in_block & cell::BOUNDARY != 0 {
                return None;
            } else {
                let masked_value = cell_in_block & cell::FULL;
                particles += masked_value.count_ones() as usize;
            }
        }
    }
    Some(particles)
}

fn generate_rgb_sequence(grid: &Grid, downscale: usize) -> Vec<u8> {
    let mut out = Vec::<u8>::new();
    for block_x in 0..grid.width / downscale {
        for block_y in 0..grid.height / downscale {
            let particles_in_block = count_particles_in_block(grid, block_x, block_y, downscale);
            let block_colour = match particles_in_block {
                Some(count) => {
                    let val = (63 * count / (downscale * downscale)) as u8;
                    RGB8::new(val, val, val)
                }
                None => RGB8::BOUNDARY,
            };
            out.extend(block_colour.as_array());
        }
    }
    out
}

fn save_grid_as_image(grid: &Grid, downscale: usize, filename: String) {
    let file = File::create(filename).unwrap(); // TODO handle error
    let writer = &mut BufWriter::new(file);
    let mut encoder = png::Encoder::new(
        writer,
        (grid.width / downscale) as u32,
        (grid.height / downscale) as u32,
    );
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap(); // TODO handle error
    let image_data = generate_rgb_sequence(grid, downscale);
    writer.write_image_data(&image_data).unwrap(); // TODO handle error
}

fn main() {
    let width = 4096;
    let height = 4096;
    let downscale = 8;
    let mut grid_a = Grid::new(width, height);
    let mut grid_b = Grid::new(width, height);
    // Features to add:
    // -> sources and sinks
    // -> load start state from a png?
    // -> microphone?

    grid_a.fill_region(0, 0, width, height - 1, 0.75);
    grid_a.fill_region(1024, 1024, 1024, 1024, 0.0);

    grid_a.fill_boundary(0, 0, 1, height);
    grid_a.fill_boundary(0, 0, width, 1);
    grid_a.fill_boundary(0, height as isize - 1, width, 1);
    grid_a.fill_boundary(width as isize - 1, 0, 1, height);

    save_grid_as_image(&grid_a, downscale, "image0.png".into());
    let iterations = 10000;
    let frameskip = 5;
    let mut frame = 0;
    for i in 1..=iterations {
        propagate_grid(&grid_a, &mut grid_b);
        std::mem::swap(&mut grid_a, &mut grid_b);
        if i % frameskip == 0 {
            save_grid_as_image(&grid_a, downscale, format!("image{}.png", frame));
            frame += 1;
        }
        if i % 100 == 0 {
            print!(
                "\r{}",
                format!("frame:{}/{}", frame, iterations / frameskip)
            );
            stdout().flush().unwrap();
        }
    }
}
