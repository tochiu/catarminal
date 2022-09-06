use super::super::{
    shape::*,
    draw::*,
    space::*,
    world::*
};

use crate::enums;

use tui::{
    style::Color,
    buffer::Cell
};

const DIGITS: [BitShape128; 10] = [
    BitShape128::new(0b0111010001100011000101110, Size2D::new(5, 5)),
    BitShape128::new(0b0110000100001000010001110, Size2D::new(5, 5)),
    BitShape128::new(0b1111000001011101000011111, Size2D::new(5, 5)),
    BitShape128::new(0b1111000001111110000111110, Size2D::new(5, 5)),
    BitShape128::new(0b1000110001011110000100001, Size2D::new(5, 5)),
    BitShape128::new(0b1111110000111100000111110, Size2D::new(5, 5)),
    BitShape128::new(0b0111010000111101000101110, Size2D::new(5, 5)),
    BitShape128::new(0b1111100010001000100010000, Size2D::new(5, 5)),
    BitShape128::new(0b0111010001011101000101110, Size2D::new(5, 5)),
    BitShape128::new(0b0111010001011110000101110, Size2D::new(5, 5))
];

const TILE_SIZE: Size2D = Size2D::new(25, 11);

lazy_static! {

    static ref DEFAULT_ROLL_NUMBER_CELL: Cell = {
        let mut cell = Cell::default();
        cell
            .set_char('@')
            .set_fg(Color::White);
        cell
    };

    static ref BEST_ROLL_NUMBER_CELL: Cell = {
        let mut cell: Cell = DEFAULT_ROLL_NUMBER_CELL.clone();
        cell
            .set_fg(Color::Red);
        cell
    };

    static ref TILE_BKG_CELL: Cell = {
        let mut cell = Cell::default();
        cell.set_char('#');
        cell
    };

    static ref ROLL_RARITY_CELL: Cell = {
        let mut cell = Cell::default();
        cell
            .set_char('O')
            .set_fg(Color::Blue);
        cell
    };

    static ref TILE_ROLL_RARITY_BITSHAPES: Vec<BitShape128> = {
        let mut shapes: Vec<BitShape128> = Vec::with_capacity(13);
        for roll in 0..13 {
            let rarity = (6 as u16).saturating_sub((7 as u16).abs_diff(roll));
            let width = (2*rarity).saturating_sub(1).max(1);
            let mut bits: u128 = 0;
            for _ in 0..rarity {
                bits = bits << 2 | 1;
            }

            shapes.push(BitShape128::new(bits, Size2D::new(width, 1)));
        }

        shapes
    };

    static ref TILE_BITSHAPE: BitShape = {
        let width = TILE_SIZE.x;
        let height = TILE_SIZE.y;

        let mut buf: Vec<u128> = Vec::with_capacity(height as usize);
        let max_width_y = (height + (height + 1) % 2)/2;

        for y in 0..height {
            let padding = max_width_y.abs_diff(y);
            let fill_width = width.saturating_sub(2*padding);
            let fill_bits: u128 = if fill_width == 0 { 0 } else { (u128::MAX >> (128 - fill_width)) << padding };
            buf.push(fill_bits);
        }

        BitShape::new(buf, TILE_SIZE)
    };
}

#[derive(Debug)]
pub struct Tile {
    roll: u8,
    resource: enums::Resource
}

impl Tile {
    pub fn new(roll: u8, resource: enums::Resource) -> Self {
        Tile { roll, resource }
    }
}

impl Drawable for Tile {
    fn on_mount(tile_drawing: &mut Drawing<Self>, controller: &mut MountController) {
        let roll = tile_drawing.pencil.roll;
        let resource = tile_drawing.pencil.resource;

        tile_drawing.set_size(UDim2::from_size2d(TILE_SIZE));

        {
            let mut bkg_drawing = controller.mount_child(Shape::new(&TILE_BITSHAPE));
            bkg_drawing.center();
            bkg_drawing.pencil.cell = TILE_BKG_CELL.clone();
            bkg_drawing.pencil.cell.set_fg(resource.get_color());
        }

        {
            let mut rarity_drawing = controller.mount_child(Shape128::new(&TILE_ROLL_RARITY_BITSHAPES[roll as usize]));
            rarity_drawing
                .set_position(UDim2::new(0.5, 0, 1.0, -1))
                .set_anchor(Scale2D::new(0.5, 0.5));
            rarity_drawing.pencil.cell = ROLL_RARITY_CELL.clone();
        }

        if roll < 10 {
            let mut digit0_drawing = controller.mount_child(Shape128::new(&DIGITS[roll as usize]));
            digit0_drawing.pencil.cell = if roll.abs_diff(7) == 1 { BEST_ROLL_NUMBER_CELL.clone() } else { DEFAULT_ROLL_NUMBER_CELL.clone() };
            digit0_drawing.center();
        } else {
            {
                let mut digit0_drawing = controller.mount_child(Shape128::new(&DIGITS[(roll % 10) as usize]));
                digit0_drawing.pencil.cell = DEFAULT_ROLL_NUMBER_CELL.clone();
                digit0_drawing
                    .set_position(UDim2::new(0.5, 1, 0.5, 0))
                    .set_anchor(Scale2D::new(0.0, 0.5));
            }
            {
                let mut digit1_drawing = controller.mount_child(Shape128::new(&DIGITS[(roll / 10) as usize]));
                digit1_drawing.pencil.cell = DEFAULT_ROLL_NUMBER_CELL.clone();
                digit1_drawing
                    .set_position(UDim2::new(0.5, -1, 0.5, 0))
                    .set_anchor(Scale2D::new(1.0, 0.5));
            }
        }
    }

    fn draw(&self, mut canvas: DrawingCanvas) {
        canvas.draw_children();
    }
}