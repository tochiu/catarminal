use crate::render::shape::{DrawableString, StringShape};

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

use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
pub struct Map {
    bkg: &'static StringShape<'static>,
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

    static ref MAP_BKG_DRAW_STRING: DrawableString<'static> = {
        DrawableString::new(MAP_CONTENT.as_str())
    };

    static ref MAP_BKG_SHAPE: StringShape<'static> = {
        StringShape::new(&MAP_BKG_DRAW_STRING, Style::default().fg(Color::White))
    };

    static ref MAP_TILE_POINTS: Vec<Point2D> = {
        let mut tile_points: Vec<Point2D> = Vec::new();
        for (x, y, grapheme) in MAP_BKG_DRAW_STRING.iter() {
            if grapheme == "[" {
                tile_points.push(Point2D::new(
                    i16::try_from(x).unwrap(),
                    i16::try_from(y).unwrap()
                ));
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
        let mut map = Map { tiles, bkg: &MAP_BKG_SHAPE, layout: DrawLayout::default(), mount: Mount::default() };

        map.layout.set_size(UDim2::from_size2d(MAP_BKG_DRAW_STRING.size));
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
        //area.buf.draw_lines(&MAP_LINES, area.draw_space, area.full_space, Style::default().fg(Color::White));
        area.draw_child(self.bkg);
        area.draw_children(&self.tiles);
    }
}

impl Mountable for Map {
    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }
    fn child_ref(&self, _: usize) -> Option<&dyn Mountable> { None }
    fn child_mut(&mut self, _: usize) -> Option<&mut dyn Mountable> { None }
}