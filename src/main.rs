use std::process::exit;
use image::{ImageBuffer};
use rand::{random, Rng, thread_rng};
use rand::prelude::ThreadRng;
use bit_field::BitField;

fn main() {
    let tile_types = vec![
        TileType::new([0, 0, 200], 0b000000000000011),
        TileType::new([0, 0, 215], 0b000000000000111),
        TileType::new([0, 0, 230], 0b000000000001110),
        TileType::new([0, 0, 255], 0b000000000011100),

        TileType::new([230, 230, 0], 0b000000000111000),
        TileType::new([255, 255, 0], 0b000000001110000),

        TileType::new([0, 255, 0], 0b000000011100000),
        TileType::new([0, 220, 0], 0b000000111000000),
        TileType::new([0, 200, 0], 0b000001110000000),

        TileType::new([0, 120, 0], 0b000011100000000),
        TileType::new([0, 100, 0], 0b000111000000000),

        TileType::new([100, 100, 100], 0b001110000000000),
        TileType::new([120, 120, 120], 0b011100000000000),

        TileType::new([230, 230, 230], 0b111000000000000),
        TileType::new([255, 255, 255], 0b110000000000000),
    ];

    let mut world = World::new((0xFF, 0xFF), tile_types);
    let mut i = 0;
    while world.collapse_propagate() {
        // world.save_image(format!("out/img{i}.png").as_str());
        i += 1;
    }
    println!("Generated with {i} iterations!");

    world.save_image("out.png");
}

struct TileType {
    color: (u8, u8, u8),
    neighbors: u128
}

impl TileType {
    fn new(col: [u8; 3], neighbors: u128) -> Self{
        TileType { color: (col[0], col[1], col[2]), neighbors}
    }
}

struct Point {
    x: usize,
    y: usize
}

struct World {
    size: (usize, usize),
    tiles: Vec<Vec<Tile>>,
    tile_types: Vec<TileType>,
    types: u8,
    rng: ThreadRng,
    collapse_options: Vec<Point>
}

impl World {
    fn new(size: (usize, usize), tile_types: Vec<TileType>) -> Self {
        let types = tile_types.len();
        assert!(types > 0 && types <= 128, "{}", format!("Must provide between 1 ad 128 tile types, found {types}"));
        /*World { size, tiles: vec![vec![Tile::new(types as u8); size.0]; size.1], tile_types, types: (types as u8),
            rng: thread_rng(), collapse_options: vec![Point{ x: 0, y: 0 },
                                                      Point{ x: 0 / 2, y: size.1 - 1 },
                                                      Point{ x: size.0 - 1, y: 0 },
                                                      Point{ x: size.0 - 1, y: size.1 - 1 }]}*/
        /*World { size, tiles: vec![vec![Tile::new(types as u8); size.0]; size.1], tile_types, types: (types as u8),
            rng: thread_rng(), collapse_options: vec![Point{ x: size.0 / 2, y: size.1 / 2 }]}*/
        World { size, tiles: vec![vec![Tile::new(types as u8); size.0]; size.1], tile_types, types: (types as u8),
            rng: thread_rng(), collapse_options: {
                let mut points = Vec::<Point>::new();
                for x in 0..16 {
                    for y in 0..16 {
                        points.push(Point { x: x * 16, y: y * 16});
                    }
                }
                points
            }
        }
    }

    fn collapse_propagate(&mut self) -> bool {
        let i = self.rng.gen_range(0..self.collapse_options.len());
        let p = self.collapse_options.remove(i);
        self.tiles[p.x][p.y].collapse(&mut self.rng);
        self.propagate(&p);
        macro_rules! push_point {
            ($x: expr, $y: expr) => {
                if !self.tiles[$x][$y].collapsed { self.collapse_options.push(Point { x: $x, y: $y }); }
            }
        }
        if p.x > 0 { push_point!(p.x - 1, p.y) }
        if p.x < self.size.0 - 1 { push_point!(p.x + 1, p.y) }
        if p.y > 0 { push_point!(p.x, p.y - 1) }
        if p.y < self.size.1 - 1 { push_point!(p.x, p.y + 1) }
        self.collapse_options.len() > 0
    }

    fn propagate(&mut self, p: &Point){
        if p.x > 0 { self.propagate_to(&p, &Point { x: p.x - 1, y: p.y}) }
        if p.x < self.size.0 - 1 { self.propagate_to( &p, &Point { x: p.x + 1, y: p.y})}
        if p.y > 0 { self.propagate_to(&p, &Point { x: p.x, y: p.y - 1})}
        if p.y < self.size.1 - 1 { self.propagate_to( &p, &Point { x: p.x, y: p.y + 1})}
    }

    fn propagate_to(&mut self, p: &Point, next: &Point){
        let v = self.tiles[next.x][next.y].wave;
        self.tiles[next.x][next.y].wave &= self.tiles[p.x][p.y].allowed_neighbors(&self.tile_types);
        if self.tiles[next.x][next.y].wave != v { self.propagate(next); }
    }

    fn save_image(&mut self, path: &str){
        let img = ImageBuffer::from_fn(self.size.0 as u32, self.size.1 as u32, |x, y| {
            if self.tiles[x as usize][y as usize].collapsed {
                let t = self.tiles[x as usize][y as usize].collapse(&mut self.rng);
                let c = self.tile_types[t as usize].color;
                image::Rgb([c.0, c.1, c.2])
            }
            else {
                image::Rgb([0, 0, 0])
            }
        });
        img.save(path).expect("Error couldnt write file!");
    }
}

#[derive(Copy, Clone)]
struct Tile {
    wave: u128,
    collapsed: bool
}

impl Tile {
    fn new(max: u8) -> Self{
        Tile { wave: (1 << max) - 1, collapsed: false }
    }

    fn collapse(&mut self, rng: &mut ThreadRng) -> u8 {
        self.collapsed = true;
        let options = self.wave.count_ones();
        let choice = rng.gen_range(0..options);
        let mut count = 0;
        for i in 0..128 {
            if self.wave.get_bit(i) {
                if choice == count {
                    self.wave = 1<<i;
                    return i as u8;
                }
                count += 1;
            }
        }
        panic!("Couldnt find any option to collapse to for {:b}", self.wave);
    }

    fn allowed_neighbors(&mut self, tile_types: &Vec<TileType>) -> u128 {
        let mut options = 0u128;
        for i in 0..tile_types.len() as usize {
            if self.wave.get_bit(i) {
                options |= tile_types[i].neighbors;
            }
        }
        return options;
    }
}
