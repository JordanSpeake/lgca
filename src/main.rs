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
        // TODO break above line into multiple asserts
        let index = y as usize * self.width + x as usize;
        self.grid[index] = value;
    }

    pub fn fill_region(
        &mut self,
        x_min: isize,
        y_min: isize,
        width: usize,
        height: usize,
        _probability: f32,
    ) {
        for y in y_min..y_min + height as isize {
            for x in x_min..x_min + width as isize {
                let value = cell::FULL; // TODO, implement probability
                self.set(x, y, value);
            }
        }
    }
}

fn resolve_collisions(cell_value: u8) -> u8 {
    match cell_value {
        0b0101 => 0b1010,
        0b1010 => 0b0101,
        other => other,
    }
}

fn propagate_grid(grid: &Grid, next_grid: &mut Grid) {
    for y in 0..grid.height as isize {
        for x in 0..grid.width as isize {
            let up = grid.get(x, y + 1) & cell::DOWN;
            let right = grid.get(x + 1, y) & cell::LEFT;
            let down = grid.get(x, y - 1) & cell::UP;
            let left = grid.get(x - 1, y) & cell::RIGHT;
            let mut next_state = up | right | down | left;
            next_state = resolve_collisions(next_state);
            next_grid.set(x, y, next_state)
        }
    }
}

fn generate_greyscale_sequence(grid: &Grid) -> Vec<u8> {
    let mut out = Vec::new();
    for cell in &grid.grid {
        let mask = 0b00001111;
        let masked_value = cell & mask;
        let bit_count = masked_value.count_ones();
        out.push((63 * bit_count) as u8);
    }
    out
}

fn save_grid_as_image(grid: &Grid, filename: String) {
    let file = File::create(filename).unwrap(); // TODO handle error
    let writer = &mut BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, grid.width as u32, grid.height as u32); // todo manually handle downcasting
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap(); // TODO handle error
    let image_data = generate_greyscale_sequence(grid);
    writer.write_image_data(&image_data).unwrap(); // TODO handle error
}

fn main() {
    let width = 512;
    let height = 512;
    let mut grid_a = Grid::new(width, height);
    let mut grid_b = Grid::new(width, height);
    grid_a.fill_region(100, 200, 100, 200, 1.0);

    let frames = 500;
    for f in 0..frames {
        propagate_grid(&grid_a, &mut grid_b);
        std::mem::swap(&mut grid_a, &mut grid_b);
        let image_name = format!("image{}.png", f);
        save_grid_as_image(&grid_a, image_name);
    }
}
