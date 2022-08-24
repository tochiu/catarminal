use super::super::{
    shape::*,
    space::*,
    draw::*
};

pub use super::tile::*;

use tui::{
    style::{Style, Color}
};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub struct Map<'a> {
    size: Size2D,
    lines: Vec<&'a str>,
    cursor: Drawing<Shape<'a>>,
    tiles: Vec<Drawing<Tile>>,
    tile_points: Vec<Point2D>,
    pub tile_capacity: usize
}

const MAP_TILE_ANCHOR: Scale2D = Scale2D::new(0.0, 0.5);

impl<'a> Map<'a> {
    pub fn from(content: &'a String, cursor_shape: &'a BitShape) -> Self {
        let lines: Vec<&'a str> = content.lines().collect();
        let height = u16::try_from(lines.len()).unwrap();
        let mut width: u16 = 0;
        let mut tile_points: Vec<Point2D> = Vec::new();

        for (y, line) in lines.iter().enumerate() {
            for (x, grapheme) in UnicodeSegmentation::graphemes(*line, false).enumerate() {
                if grapheme == "[" {
                    tile_points.push(Point2D::new(
                        i16::try_from(x).unwrap(), 
                        i16::try_from(y).unwrap()
                    ));
                    
                    // let mut tile = Drawing::a(Tile::from(11));
                    // tile
                    //     .set_position(UDim2::from_offset(
                    //         i16::try_from(x).unwrap(), 
                    //         i16::try_from(y).unwrap()
                    //     ))
                    //     .set_anchor(Scale2D::new(0.0, 0.5));
                    // tiles.push(tile);
                }
            }

            width = width.max(u16::try_from(line.width()).unwrap());
        }
        
        let mut cursor = Drawing::a(Shape::from(&cursor_shape));
        cursor
            .set_space(cursor.space.center())
            .pencil.cell
                .set_bg(Color::LightBlue);

        Map {
            size: Size2D::new(width, height),
            lines,
            cursor,
            tiles: Vec::with_capacity(tile_points.len()),
            tile_capacity: tile_points.len(),
            tile_points
        }
    }

    pub fn set_tiles(&mut self, mut tiles: Vec<Drawing<Tile>>) {
        tiles.truncate(self.tile_capacity);
        for (i, tile) in tiles.iter_mut().enumerate() {
            tile
                .set_position(UDim2::from_point2d(self.tile_points[i]))
                .set_anchor(MAP_TILE_ANCHOR);
        }
        self.tiles = tiles;
    }

    pub fn set_cursor_pos(&mut self, position: UDim2) {
        self.cursor.set_position(position);
    }
}

impl<'a> Drawable for Map<'_> {
    fn get_space(&self) -> Space {
        Space::from_size2d(self.size)
    }

    fn draw(&self, canvas: &mut DrawCanvas) {
        canvas.buf.draw_lines(&self.lines, canvas.draw_space, Style::default().fg(Color::White));
        for tile in self.tiles.iter() {
            tile.draw_in(canvas);
        }
        self.cursor.draw_in(canvas);
    }
}