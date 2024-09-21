use std::{fs::File, io::BufWriter, path::Path};

#[derive(Clone)]
struct Cell {
    value: u8,
}

impl Cell {
    pub fn new() -> Self {
        Self { value: 0 }
    }
}

struct Grid {
    grid: Vec<Cell>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![Cell::new(); width * height];
        Self {
            grid,
            width,
            height,
        }
    }
}

fn generate_greyscale_sequence(grid: Grid) -> Vec<u8> {
    let mut out = Vec::new();
    for cell in grid.grid {
        out.push(cell.value.count_ones() as u8);
    }
    out
}

fn save_grid_as_image(grid: Grid) {
    let path = Path::new("image.png");
    let file = File::create(path).unwrap(); // TODO handle error
    let ref mut writer = BufWriter::new(file);
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
    let grid = Grid::new(width, height);
    save_grid_as_image(grid);
}
