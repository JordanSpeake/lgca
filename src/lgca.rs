#![allow(dead_code)]

use std::ops::AddAssign;
use rand::{thread_rng, Rng};

pub type Cell = u8;

pub mod cell {
    pub const FULL: u8 = 0b0000_1111;
    pub const UP: u8 = 0b0000_1000;
    pub const RIGHT: u8 = 0b0000_0100;
    pub const DOWN: u8 = 0b0000_0010;
    pub const LEFT: u8 = 0b0000_0001;
    pub const EMPTY: u8 = 0b0000_0000;
    pub const BOUNDARY: u8 = 0b0001_0000;
}

pub struct Config {
    pub width: usize,
    pub height: usize,
    pub downscale: usize,
    pub iterations: usize,
    pub frameskip: usize,
}

impl Config {
    pub fn new(
        width: usize,
        height: usize,
        downscale: usize,
        iterations: usize,
        frameskip: usize,
    ) -> Self {
        Self {
            width,
            height,
            downscale,
            iterations,
            frameskip,
        }
    }
}

pub struct RGB8 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RGB8 {
    pub const BLACK: Self = RGB8::new(0, 0, 0);
    pub const BOUNDARY: Self = RGB8::new(0, 0, 0);

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

pub struct Grid {
    pub grid: Vec<Cell>,
    pub width: usize,
    pub height: usize,
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

    pub fn set_boundary_at_edge(&mut self, config: &Config) {
        self.fill_boundary(0, 0, 1, config.height);
        self.fill_boundary(0, 0, config.width, 1);
        self.fill_boundary(0, config.height as isize - 1, config.width, 1);
        self.fill_boundary(config.width as isize - 1, 0, 1, config.height);
    }
}

pub struct Source {
    pub x: isize,
    pub y: isize,
    pub width: usize,
    pub height: usize,
    pub density: f64,
}

impl Source {
    pub fn new(x: isize, y: isize, width: usize, height: usize, density: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            density,
        }
    }
}
