use super::{
    space::*,
    draw::*, 
    screen::*
};

use tui::{buffer::Cell, style::Style};

use unicode_segmentation::{UnicodeSegmentation, Graphemes};
use unicode_width::UnicodeWidthStr;

/* 
 * TODO: allow arbitrary lifetimes for BitShape128, Shap128, BitShape, Shape
 * MountableLayout structs that need to store shapes can specify a static lifetime themselves
 */ 

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

#[derive(Debug)]
pub struct Shape128 {
    pub layout: DrawLayout,
    pub shape: &'static BitShape128,
    pub cell: Cell
}

impl Shape128 {
    pub fn new(shape: &'static BitShape128, cell: Cell) -> Self {
        Shape128 { 
            shape, 
            cell, 
            layout: DrawLayout::default()
                .set_size(UDim2::from_size2d(shape.size))
                .clone() 
        }
    }
}

impl Layoutable for Shape128 {
    fn layout_ref(&self) -> &DrawLayout {
        &self.layout
    }
}

impl Drawable for Shape128 {
    fn draw(&self, area: ScreenArea) {
        for point in area.absolute_draw_space {
            let bit_point = area.absolute_layout_space.relative_position_of(point);
            if self.shape.bits >> ((bit_point.y as u16)*self.shape.size.x + bit_point.x as u16) & 1 == 1 {
                let i = area.buf.index_of(point.x as u16, point.y as u16);
                area.buf.content[i] = self.cell.clone();
            }
        }
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

#[derive(Debug)]
pub struct Shape {
    pub layout: DrawLayout,
    pub shape: &'static BitShape,
    pub cell: Cell
}

impl Shape {
    pub fn new(shape: &'static BitShape, cell: Cell) -> Self {
        Shape { 
            shape, 
            cell, 
            layout: DrawLayout::default()
                .set_size(UDim2::from_size2d(shape.size))
                .clone() 
        }
    }
}

impl Layoutable for Shape {
    fn layout_ref(&self) -> &DrawLayout {
        &self.layout
    }
}

impl Drawable for Shape {
    fn draw(&self, area: ScreenArea) {
        for point in area.absolute_draw_space {
            let bit_point = area.absolute_layout_space.relative_position_of(point);
            let bit_index = (bit_point.y as u16)*self.shape.size.x + bit_point.x as u16;
            if self.shape.bits[bit_index as usize / 128] >> (bit_index % 128) & 1 == 1 {
                let i = area.buf.index_of(point.x as u16, point.y as u16);
                area.buf.content[i] = self.cell.clone();
            }
        }
    }
}

#[derive(Debug)]
pub struct DrawableString<'a> {
    pub lines: Vec<&'a str>,
    pub size: Size2D
}

impl<'a> DrawableString<'a> {
    pub fn new(content: &'a str) -> Self {

        let lines: Vec<&str> = content.lines().collect();

        let height = u16::try_from(lines.len()).unwrap();
        let mut width: u16 = 0;
        for &line in lines.iter() {
            width = width.max(u16::try_from(line.width()).unwrap());
        }

        DrawableString {
            lines,
            size: Size2D::new(width, height)
        }
    }

    pub fn iter(&self) -> DrawableStringIterator {
        DrawableStringIterator { 
            shape: self, 
            graphemes: self.lines[0].graphemes(false), 
            x: 0, 
            y: 0
        }
    }
}

pub struct DrawableStringIterator<'a> {
    shape: &'a DrawableString<'a>,
    graphemes: Graphemes<'a>,
    x: u16,
    y: u16
}

impl<'a> Iterator for DrawableStringIterator<'a> {
    type Item = (u16, u16, &'a str);
    fn next(&mut self) -> Option<Self::Item> {
        let maybe_grapheme = self.graphemes.next();

        if let Some(grapheme) = maybe_grapheme {
            let result = Some((self.x, self.y, grapheme));
            self.x += 1;
            result
        } else {
            self.x = 0;
            self.y += 1;
            while self.y < self.shape.size.y {
                self.graphemes = self.shape.lines[self.y as usize].graphemes(false);
                if let Some(grapheme) = self.graphemes.next() {
                    let result = Some((self.x, self.y, grapheme));
                    self.x += 1;
                    return result;
                }
            }
            None
        }
    }
}

#[derive(Debug)]
pub struct StringShape<'a> {
    pub shape: &'a DrawableString<'a>,
    pub style: Style,
    pub layout: DrawLayout
}

impl<'a> StringShape<'a> {
    pub fn new(shape: &'a DrawableString, style: Style) -> Self {
        StringShape { 
            shape, 
            style, 
            layout: DrawLayout::default()
                .set_size(UDim2::from_size2d(shape.size))
                .clone() 
        }
    }
}

impl<'a> Layoutable for StringShape<'_> {
    fn layout_ref(&self) -> &DrawLayout {
        &self.layout
    }
}

impl<'a> Drawable for StringShape<'_> {
    fn draw(&self, area: ScreenArea) {
        let absolute_draw_space = area.absolute_draw_space;
        let absolute_layout_space = area.absolute_layout_space;
        let offset_y = absolute_draw_space.position.y - absolute_layout_space.position.y;
        let offset_x = absolute_draw_space.position.x - absolute_layout_space.position.x;
        for y in 0..self.shape.lines.len().min(absolute_draw_space.size.y as usize) as i16 {
            let point = absolute_draw_space.absolute_position_of(Point2D::new(0, y as i16));
            area.buf.set_stringn(
                point.x as u16, 
                point.y as u16, 
                &self.shape.lines[(y + offset_y) as usize][offset_x as usize..], 
                absolute_draw_space.size.x as usize, 
                self.style
            );
        }
    }
}