use std::{
    f64::consts::PI,
    fs::File,
    io::{stdout, BufWriter, Write},
    time::Instant,
};

mod lgca;
use lgca::*;

fn propagate_grid(grid: &Grid, next_grid: &mut Grid) {
    for y in 0..grid.height as isize {
        for x in 0..grid.width as isize {
            let up = grid.get(x, y + 1) & lgca::cell::DOWN;
            let right = grid.get(x + 1, y) & lgca::cell::LEFT;
            let down = grid.get(x, y - 1) & lgca::cell::UP;
            let left = grid.get(x - 1, y) & lgca::cell::RIGHT;
            let mut next_state = up | right | down | left | (grid.get(x, y) & lgca::cell::BOUNDARY);
            next_state = resolve_collisions(next_state);
            next_grid.set(x, y, next_state);
        }
    }
}

fn resolve_collisions(cell_value: u8) -> u8 {
    if cell_value & lgca::cell::BOUNDARY == 0 {
        match cell_value {
            0b0101 => 0b1010,
            0b1010 => 0b0101,
            other => other,
        }
    } else {
        let up = cell_value & lgca::cell::UP;
        let right = cell_value & lgca::cell::RIGHT;
        let down = cell_value & lgca::cell::DOWN;
        let left = cell_value & lgca::cell::LEFT;
        up >> 2 | right >> 2 | down << 2 | left << 2 | lgca::cell::BOUNDARY
    }
}

fn block_colour_density_bw(block_x: usize, block_y: usize, grid: &Grid, config: &Config) -> RGB8 {
    let block = lgca::Block::new(block_x, block_y, config.downscale, grid);
    if block.boundary > 0 {
        lgca::RGB8::BOUNDARY
    } else {
        let val = (63 * block.total_particles() / (config.downscale * config.downscale)) as u8;
        lgca::RGB8::new(val, val, val)
    }
}

fn block_colour_velocity_rgb(block_x: usize, block_y: usize, grid: &Grid, config: &Config) -> RGB8 {
    let block = lgca::Block::new(block_x, block_y, config.downscale, grid);
    if block.boundary > 0 {
        lgca::RGB8::BOUNDARY
    } else {
        let x: f64 =
            (block.right as f64 - block.left as f64) / (config.downscale * config.downscale) as f64;
        let y: f64 = (block.up as f64 - block.down as f64) / (config.downscale * config.downscale) as f64;
        let speed = f64::powf(f64::sqrt((x * x) + (y * y))/f64::sqrt(2.0), 1.0/3.0);
        let mut angle = f64::atan2(x, y);
        angle = if angle < 0.0 {
            angle + 2.0*PI
        } else {
            angle
        };
        angle = 180.0*angle/PI;
        // println!("speed: {} \n angle: {}", speed, angle);
        lgca::RGB8::from_hsvf64(angle, speed, speed)
    }
}

fn generate_rgb_sequence(grid: &Grid, config: &Config) -> Vec<u8> {
    let mut out = Vec::<u8>::new();
    for block_x in 0..grid.width / config.downscale {
        for block_y in 0..grid.height / config.downscale {
            let block_colour = match config.colouring {
                Colouring::DensityBW => block_colour_density_bw(block_x, block_y, grid, config),
                Colouring::VelocityColour => {
                    block_colour_velocity_rgb(block_x, block_y, grid, config)
                }
            };
            out.extend(block_colour.as_array());
        }
    }
    out
}

fn save_grid_as_image(grid: &Grid, config: &Config, filename: &str) {
    let file = File::create(filename).unwrap(); // TODO handle error
    let writer = &mut BufWriter::new(file);
    let mut encoder = png::Encoder::new(
        writer,
        (grid.width / config.downscale) as u32,
        (grid.height / config.downscale) as u32,
    );
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .expect("Failed to create image writer");
    let image_data = generate_rgb_sequence(grid, config);
    writer
        .write_image_data(&image_data)
        .expect(format!("Failed to write to {}", filename).as_str());
}

fn update_sources(grid: &mut lgca::Grid, sources: &[lgca::Source]) {
    for source in sources {
        grid.fill_region(
            source.x,
            source.y,
            source.width,
            source.height,
            source.density,
        );
    }
}

fn tick(
    config: &lgca::Config,
    grid_a: &mut lgca::Grid,
    grid_b: &mut lgca::Grid,
    sources: &Vec<lgca::Source>,
    start_time: Instant,
    i: usize,
) {
    update_sources(grid_a, sources);
    propagate_grid(grid_a, grid_b);
    std::mem::swap(grid_a, grid_b);
    if i % config.frameskip == 0 {
        save_grid_as_image(
            &grid_a,
            config,
            &format!("output/image{}.png", i / config.frameskip),
        );
    }
    let iterations_remaining = config.iterations - i + 1;
    let iterations_per_second = i as f64 / start_time.elapsed().as_secs_f64();
    let time_remaining = iterations_remaining as f64 / iterations_per_second;
    let hours = time_remaining as usize / 3600;
    let minutes = (time_remaining as usize % 3600) / 60;
    let seconds = time_remaining as usize % 60;
    print!("\r\x1B[2K");
    print!("step: {}/{} ", i, config.iterations);
    print!("time remaining: {}hr {}min {}sec", hours, minutes, seconds);
    stdout().flush().unwrap();
}

fn main() {
    let config = lgca::Config::new(8192, 8192, 16, 20_000, 20, Colouring::VelocityColour);
    let mut grid_a = Grid::new(config.width, config.height);
    let mut grid_b = Grid::new(config.width, config.height);
    grid_a.fill_region(0, 0, config.width, config.height - 1, 0.25);
    grid_a.set_boundary_at_edge(&config);
    grid_a.fill_region(3072, 3072, 2048, 2048, 1.0);
    let mut sources = Vec::<lgca::Source>::new();
    // sources.push(lgca::Source::new(100, 100, 500, 500, 0.75));
    // sources.push(Source::new(3500, 3500, 500, 500, 0.00));

    save_grid_as_image(&grid_a, &config, "output/image0.png".into());
    let start_time = Instant::now();
    for i in 1..=config.iterations {
        tick(&config, &mut grid_a, &mut grid_b, &sources, start_time, i);
    }
}
