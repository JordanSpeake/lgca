use std::{
    fs::File,
    io::{stdout, BufWriter, Write},
    time::Instant,
};
mod lgca;


fn propagate_grid(grid: &lgca::Grid, next_grid: &mut lgca::Grid) {
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

fn count_particles_in_block(
    grid: &lgca::Grid,
    block_x: usize,
    block_y: usize,
    downscale: usize,
) -> Option<usize> {
    let mut particles = 0;
    for cell_x in downscale * block_x..downscale * block_x + downscale - 1 {
        for cell_y in downscale * block_y..downscale * block_y + downscale - 1 {
            let cell_in_block = grid.get(cell_x as isize, cell_y as isize);
            if cell_in_block & lgca::cell::BOUNDARY != 0 {
                return None;
            } else {
                let masked_value = cell_in_block & lgca::cell::FULL;
                particles += masked_value.count_ones() as usize;
            }
        }
    }
    Some(particles)
}

fn generate_rgb_sequence(grid: &lgca::Grid, downscale: usize) -> Vec<u8> {
    let mut out = Vec::<u8>::new();
    for block_x in 0..grid.width / downscale {
        for block_y in 0..grid.height / downscale {
            let particles_in_block = count_particles_in_block(grid, block_x, block_y, downscale);
            let block_colour = match particles_in_block {
                Some(count) => {
                    let val = (63 * count / (downscale * downscale)) as u8;
                    lgca::RGB8::new(val, val, val)
                }
                None => lgca::RGB8::BOUNDARY,
            };
            out.extend(block_colour.as_array());
        }
    }
    out
}

fn save_grid_as_image(grid: &lgca::Grid, downscale: usize, filename: &String) {
    let file = File::create(filename).expect(format!("Failed to create {}", filename).as_str());
    let writer = &mut BufWriter::new(file);
    let mut encoder = png::Encoder::new(
        writer,
        (grid.width / downscale) as u32,
        (grid.height / downscale) as u32,
    );
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("Failed to create image writer");
    let image_data = generate_rgb_sequence(grid, downscale);
    writer.write_image_data(&image_data).expect(format!("Failed to write to {}", filename).as_str());
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
            config.downscale,
            &format!("image{}.png", i / config.frameskip),
        );
    }
    let iterations_remaining = config.iterations - i;
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
    // let config = Config::new(4096, 4096, 8, 1000, 20);
    let config = lgca::Config::new(2048, 2048, 8, 1000, 20);
    let mut grid_a = lgca::Grid::new(config.width, config.height);
    let mut grid_b = lgca::Grid::new(config.width, config.height);

    grid_a.fill_region(0, 0, config.width, config.height - 1, 0.25);
    grid_a.set_boundary_at_edge(&config);
    let mut sources = Vec::<lgca::Source>::new();
    sources.push(lgca::Source::new(100, 100, 500, 500, 0.75));
    // sources.push(Source::new(3500, 3500, 500, 500, 0.00));

    save_grid_as_image(&grid_a, config.downscale, &"image0.png".into());
    let start_time = Instant::now();
    for i in 1..=config.iterations {
        tick(&config, &mut grid_a, &mut grid_b, &sources, start_time, i);
    }
}
