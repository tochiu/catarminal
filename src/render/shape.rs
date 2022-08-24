use super::{
    space::*,
    draw::*
};

use tui::buffer::Cell;

#[derive(Debug, Default, Copy, Clone)]
pub struct BitShape128 {
    pub bits: u128,
    pub size: Size2D
}

impl BitShape128 {
    pub const fn new(bits: u128, size: Size2D) -> Self {
        BitShape128 { bits: (bits << (u128::BITS as u16 - size.x*size.y)).reverse_bits(), size }
    }
}

#[derive(Debug, Default, Clone)]
pub struct BitShape {
    pub bits: Vec<u128>,
    pub size: Size2D
}

impl BitShape {
    pub fn new(rows: Vec<u128>, size: Size2D) -> Self {
        let width = size.x;

        let mut buf: Vec<u128> = Vec::with_capacity((size.area() as usize + 127)/128);
        let mut bits: u128 = 0;
        let mut bits_width: u16 = 0;

        for row_bits in rows {

            let new_bits_width = bits_width + width;
            
            if new_bits_width < 128 {
                bits = bits << width | row_bits;
                bits_width = new_bits_width;
            } else if new_bits_width > 128 {
                let overflow_count = new_bits_width % 128;
                bits = (bits << (width - overflow_count)) | (row_bits >> overflow_count);
                buf.push(bits.reverse_bits());
                bits = row_bits & (u128::MAX >> (128 - overflow_count));
                bits_width = overflow_count;
            } else {
                bits = bits << width | row_bits;
                buf.push(bits.reverse_bits());
                bits = 0;
                bits_width = 0;
            }
        }

        if bits_width > 0 {
            buf.push(bits.reverse_bits() >> (128 - bits_width));
        }

        BitShape {
            bits: buf,
            size: size
        }
    }
}

pub struct Shape<'a> {
    pub shape: &'a BitShape,
    pub cell: Cell
}

impl<'a> Shape<'a> {
    pub fn from(shape: &'a BitShape) -> Self {
        Shape { shape, cell: Cell::default() }
    }
}

impl Drawable for Shape<'_> {
    fn get_space(&self) -> Space {
        Space::from_size2d(self.shape.size)
    }

    fn draw(&self, canvas: &mut DrawCanvas) {
        for point in canvas.draw_space {
            let bit_point = canvas.full_space.relative_position_of(point);
            let bit_index = (bit_point.y as u16)*self.shape.size.x + bit_point.x as u16;
            if self.shape.bits[bit_index as usize / 128] >> (bit_index % 128) & 1 == 1 {
                let i = canvas.buf.index_of(point.x as u16, point.y as u16);
                canvas.buf.content[i] = self.cell.clone();
            }
        }
    }
}

pub struct Shape128<'a> {
    pub shape: &'a BitShape128,
    pub cell: Cell
}

impl<'a> Shape128<'a> {
    pub fn from(shape: &'a BitShape128) -> Self {
        Shape128 { shape, cell: Cell::default() }
    }
}

impl Drawable for Shape128<'_> {
    fn get_space(&self) -> Space {
        Space::from_size2d(self.shape.size)
    }

    fn draw(&self, canvas: &mut DrawCanvas) {
        for point in canvas.draw_space {
            let bit_point = canvas.full_space.relative_position_of(point);
            if self.shape.bits >> ((bit_point.y as u16)*self.shape.size.x + bit_point.x as u16) & 1 == 1 {
                let i = canvas.buf.index_of(point.x as u16, point.y as u16);
                canvas.buf.content[i] = self.cell.clone();
            }
        }
    }
}