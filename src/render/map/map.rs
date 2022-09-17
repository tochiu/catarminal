use super::{
    tile::*,
    super::{
        space::*,
        draw::*,
        world::*,
        mount::*
    }
};

use tui::style::{Color, Style};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Default)]
pub struct Map {
    tiles: Vec<Tile>,
    layout: DrawLayout,
    mount: Mount
}

const MAP_TILE_ANCHOR: Scale2D = Scale2D::new(0.0, 0.5);

lazy_static! {

    static ref MAP_CONTENT: String = {
        let mut file = File::open("./assets/map.txt").expect("Cannot open the file");
        let mut file_str = String::new();
        file.read_to_string(&mut file_str).expect("Cannot read the file");
        file_str
    };

    static ref MAP_LINES: Vec<&'static str> = {
        MAP_CONTENT.lines().collect()
    };

    static ref MAP_SIZE: Size2D = {
        let height = u16::try_from(MAP_LINES.len()).unwrap();
        let mut width: u16 = 0;
        for line in MAP_LINES.iter() {
            width = width.max(u16::try_from(line.width()).unwrap());
        }

        Size2D::new(width, height)
    };

    static ref MAP_TILE_POINTS: Vec<Point2D> = {
        let mut tile_points: Vec<Point2D> = Vec::new();
        for (y, line) in MAP_LINES.iter().enumerate() {
            for (x, grapheme) in UnicodeSegmentation::graphemes(*line, false).enumerate() {
                if grapheme == "[" {
                    tile_points.push(Point2D::new(
                        i16::try_from(x).unwrap(),
                        i16::try_from(y).unwrap()
                    ));
                }
            }
        }

        tile_points
    };
}

impl Map {
    pub fn get_tile_capacity() -> usize {
        MAP_TILE_POINTS.len()
    }

    pub fn new(tiles: Vec<Tile>) -> Self {
        let mut map = Map { tiles, ..Map::default() };

        map.layout.set_size(UDim2::from_size2d(*MAP_SIZE));
        for (i, tile) in map.tiles.iter_mut().enumerate() {
            tile.layout
                .set_position(UDim2::from_point2d(MAP_TILE_POINTS[i]))
                .set_anchor(MAP_TILE_ANCHOR);
        }

        map
    }
}

impl Layoutable for Map {
    fn layout_ref(&self) -> &DrawLayout {
        &self.layout
    }
}

impl Drawable for Map {
    fn draw(&self, mut area: WorldArea) {
        area.buf.draw_lines(&MAP_LINES, area.draw_space, area.full_space, Style::default().fg(Color::White));
        area.draw_children(&self.tiles);
    }
}

impl Mountable for Map {
    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }
    fn child_ref(&self, _: usize) -> Option<&dyn Mountable> { None }
    fn child_mut(&mut self, _: usize) -> Option<&mut dyn Mountable> { None }
}