use super::{
    tile::*,
    super::{
        space::*,
        shape::*,
        draw::*,
        world::*
    }
};

use tui::style::Color;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
pub struct Map {
    pub cursor_dref: Option<DrawRef<Shape>>,
    cursor_shape: Option<Shape>,
    tile_queue: Vec<Tile>
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

    pub fn new(cursor_shape: Shape, tiles: Vec<Tile>) -> Self {
        Map {
            tile_queue: tiles,
            cursor_shape: Some(cursor_shape),
            cursor_dref: None
        }
    }

    // pub fn set_cursor_pos(&self, position: Point2D, world_canvas: &WorldCanvas) {
    //     world_canvas
    //         .get_mut(self.cursor_dref.as_ref().unwrap())
    //         .set_position(UDim2::from_point2d(position));
    // }
}

impl<'a> Drawable for Map {
    fn on_mount(map_drawing: &mut Drawing<Map>, controller: &mut MountController) {
        map_drawing.set_size(UDim2::from_size2d(*MAP_SIZE));

        for (i, tile) in map_drawing.pencil.tile_queue.drain(..).enumerate() {
            controller.mount_child(tile)
                .set_position(UDim2::from_point2d(MAP_TILE_POINTS[i]))
                .set_anchor(MAP_TILE_ANCHOR);
        }

        let mut cursor_drawing = controller.mount_child(map_drawing.pencil.cursor_shape.take().unwrap());

        cursor_drawing
            .center()
            .pencil.cell
                .set_bg(Color::LightBlue);
        
        map_drawing.pencil.cursor_dref = Some(cursor_drawing.as_dref());
    }

    fn draw(&self, mut canvas: DrawingCanvas) {
        //canvas.world
        //    .get_mut(self.cursor_dref.as_ref().unwrap())
        //    .set_position(UDim2::from_point2d(self.cursor_position));
        
        canvas.draw_children();
    }
}