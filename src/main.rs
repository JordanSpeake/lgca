use std::{fs::File, io::BufWriter, path::Path};

#[derive(Clone)]
struct Cell {
    value: u8,
}

impl Cell {
    const FULL: Self = Cell::new(0b00001111);

    pub const fn new(value: u8) -> Self {
        Self { value }
    }
}

struct Grid {
    grid: Vec<Cell>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![Cell::new(0); width * height];
        Self {
            grid,
            width,
            height,
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, value: Cell) {
        let index = y * self.width + x;
        self.grid[index] = value;
    }

    pub fn fill_region(
        &mut self,
        x_min: usize,
        y_min: usize,
        width: usize,
        height: usize,
        probability: f32,
    ) {
        for y in y_min..y_min + height {
            for x in x_min..x_min + width {
                let value = Cell::FULL; // Todo, implement probability
                self.set_cell(x, y, value);
            }
        }
    }
}

fn generate_greyscale_sequence(grid: Grid) -> Vec<u8> {
    let mut out = Vec::new();
    for cell in grid.grid {
        let mask = 0b00001111;
        let masked_value = cell.value & mask;
        let bit_count = masked_value.count_ones();
        out.push((63 * bit_count) as u8);
    }
    out
}

fn save_grid_as_image(grid: Grid) {
    let path = Path::new("image.png");
    let file = File::create(path).unwrap(); // TODO handle error
    let writer = &mut BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, grid.width as u32, grid.height as u32); // todo manually handle downcasting
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap(); // TODO handle error
    let image_data = generate_greyscale_sequence(grid);
    writer.write_image_data(&image_data).unwrap(); // TODO handle error
}

fn main() {
    let width = 8;
    let height = 8;
    let mut grid = Grid::new(width, height);
    grid.fill_region(1, 2, 2, 3, 1.0);
    save_grid_as_image(grid);
}
