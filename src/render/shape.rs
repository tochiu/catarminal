/*
 * shape.rs
 * a module of utility drawings that help with drawing shapes onto the screen
 * a "shape" is a drawing with arbitrary content but one style applied
 */
use super::{
    space::*,
    draw::*
};

use tui::style::Style;

use unicode_segmentation::{UnicodeSegmentation, Graphemes};
use unicode_width::UnicodeWidthStr;

// TODO: allow arbitrary lifetimes for BitShape128, Shape128, BitShape, Shape
// TODO: MountableLayout structs that need to store shapes can specify a static lifetime themselves

 /*
  * BitShape128 defines a mxn grid where mxn < 128 filled with pixels defined by a u128
  * ex: u128 = 110000 with m = 3, n = 2 translates to the first 2 pixels of the top left row being filled
  */
#[derive(Debug, Default, Copy, Clone)]
pub struct BitShape128 {
    pub bits: u128,
    pub size: Size2D
}

impl BitShape128 {
    pub const fn new(bits: u128, size: Size2D) -> Self {
        /* 
         * translate the bits so the least significant bit corresponds to the top-left area of the layout 
         * shift bits 128 - m*n to the left then reverse them
         */
        BitShape128 { bits: (bits << (u128::BITS as u16 - size.x*size.y)).reverse_bits(), size }
    }
}

/*
 * Shape128 is a BitShape128 but with a defined symbol and style to fill the marked cells
 * and layout to position the shape
 */
#[derive(Debug, Clone)]
pub struct Shape128 {
    pub layout: DrawLayout,
    pub bitshape: &'static BitShape128,
    pub symbol: &'static str,
    pub style: Style
}

impl Shape128 {
    pub fn new(bitshape: &'static BitShape128, symbol: &'static str, style: Style, mut layout: DrawLayout) -> Self {
        layout.set_size(UDim2::from_size2d(bitshape.size));
        Shape128 {
            layout,
            bitshape, 
            symbol,
            style
        }
    }
}

impl Layoutable for Shape128 {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl Drawable for Shape128 {
    fn draw(&self, ctx: &mut DrawContext) {
        for point in ctx.absolute_draw_space {
            let bit_point = ctx.absolute_layout_space.relative_position_of(point);
            /* 
             * y*xsize + x defines the amount of right shifting to get to the bit for this cell to the right
             * bitwise and with 1 and check if == 1 to see if the bit is a 1 
             */
            if self.bitshape.bits >> ((bit_point.y as u16)*self.bitshape.size.x + bit_point.x as u16) & 1 == 1 {
                let i = ctx.buf.index_of(point.x as u16, point.y as u16);
                ctx.buf.content[i]
                    .set_symbol(self.symbol)
                    .set_style(self.style);
            }
        }
    }
}

/*
 * BitShape as BitShape128 but instead a Vec of u128s
 * each input u128 defines a row in the area
 * new fn unions the used regions of the u128s together
 * m x n size (m < 128)
 */
#[derive(Debug, Default, Clone)]
pub struct BitShape {
    pub bits: Vec<u128>,
    pub size: Size2D
}

// TODO: comment this...
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

    // assumed size.area() > 0
    pub fn paint<F>(size: Size2D, painter: F) -> Self
        where F: Fn(u16, u16) -> bool
    {
        let area = size.area();
        let capacity = (area as usize + 127)/128;
        let mut bits = Vec::with_capacity(capacity);
        let mut index = 0;
        for _ in 0..capacity {
            let mut chunk: u128 = 0;
            let chunk_size = (area - index).min(128);
            index += chunk_size;
            let mut x = index % size.x;
            let mut y = index / size.x;
            for _ in 0..chunk_size {
                if x > 0 {
                    x -= 1;
                } else {
                    y -= 1;
                    x = size.x - 1;
                }
                chunk = chunk << 1 | if painter(x, y) { 1 } else { 0 };
            }
            bits.push(chunk);
        }
        BitShape { bits, size }
    }

    // assumed lhs.size == rhs.size
    // pub fn intersect(&mut self, rhs: &BitShape) -> &mut Self {
    //     for (lhs, rhs) in self.bits.iter_mut().zip(rhs.bits.iter()) {
    //         *lhs &= rhs;
    //     }
    //     self
    // }

    pub fn is_filled_at(&self, x: u16, y: u16) -> bool {
        if x >= self.size.x || y >= self.size.y {
            false
        } else {
            let index = (self.size.x*y + x) as usize;
            self.bits[index / 128] >> (index % 128) & 1 == 1
        }
    }
}

#[derive(Debug, Clone)]
pub struct Shape<'a> {
    pub layout: DrawLayout,
    pub bitshape: &'a BitShape,
    pub symbol: &'a str,
    pub style: Style
}

impl<'a> Shape<'a> {
    pub fn new(bitshape: &'a BitShape, symbol: &'a str, style: Style, mut layout: DrawLayout) -> Self {
        layout.set_size(UDim2::from_size2d(bitshape.size));
        Shape { 
            bitshape, 
            symbol,
            style,
            layout
        }
    }
}

impl<'a> Layoutable for Shape<'_> {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

// TODO: comment this...
impl<'a> Drawable for Shape<'_> {
    fn draw(&self, ctx: &mut DrawContext) {
        for point in ctx.absolute_draw_space {
            let bit_point = ctx.absolute_layout_space.relative_position_of(point);
            let bit_index = (bit_point.y as u16)*self.bitshape.size.x + bit_point.x as u16;
            if self.bitshape.bits[bit_index as usize / 128] >> (bit_index % 128) & 1 == 1 {
                let i = ctx.buf.index_of(point.x as u16, point.y as u16);
                ctx.buf.content[i]
                    .set_symbol(self.symbol)
                    .set_style(self.style);
            }
        }
    }
}

/* DrawableString takes a string and holds references to each line segment */
#[derive(Debug, Clone)]
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
            graphemes: self.lines[0].graphemes(true), 
            x: 0, 
            y: 0
        }
    }
}

/* Implement an iterator that goes through all the graphemes in the string with their x y coordinates */
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
                self.graphemes = self.shape.lines[self.y as usize].graphemes(true);
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

#[derive(Debug, Clone)]
pub struct StringShape<'a> {
    pub shape: &'a DrawableString<'a>,
    pub style: Style,
    pub layout: DrawLayout
}

impl<'a> StringShape<'a> {
    pub fn new(shape: &'a DrawableString, style: Style, mut layout: DrawLayout) -> Self {
        layout.set_size(UDim2::from_size2d(shape.size));
        StringShape { 
            shape, 
            style, 
            layout
        }
    }
}

impl<'a> Layoutable for StringShape<'_> {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl<'a> Drawable for StringShape<'_> {
    fn draw(&self, ctx: &mut DrawContext) {
        let absolute_draw_space = ctx.absolute_draw_space;
        let absolute_layout_space = ctx.absolute_layout_space;
        let offset_y = absolute_draw_space.position.y - absolute_layout_space.position.y;
        let offset_x = absolute_draw_space.position.x - absolute_layout_space.position.x;
        for y in 0..self.shape.lines.len().min(absolute_draw_space.size.y as usize) as i16 {
            let point = absolute_draw_space.absolute_position_of(Point2D::new(0, y as i16));
            ctx.buf.set_stringn(
                point.x as u16, 
                point.y as u16, 
                &self.shape.lines[(y + offset_y) as usize][offset_x as usize..], 
                absolute_draw_space.size.x as usize, 
                self.style
            );
        }
    }
}

/* struct is abstract (not meant to be instanced) for organization */
pub struct Ellipse;

impl Ellipse {
    pub fn painter(center: Float2D, semiaxis_size_x: f32, semiaxis_size_y: f32) -> impl Fn(Point2D) -> bool 
    {
        move |point| 
            ((point.x as f32 + 0.5 - center.x)/semiaxis_size_x).powi(2) + 
            ((point.y as f32 + 0.5 - center.y)/semiaxis_size_y).powi(2) <= 1.0
    }

    pub fn scaled_circle_painter(canvas_size: Size2D, center: Float2D, alpha: f32) -> impl Fn(Point2D) -> bool {
        let a = 0.5*alpha*canvas_size.x.max(2*canvas_size.y) as f32; // 2*y because font height is twice font width
        let b = 0.5*a; // font height in terminal is twice width so to account for this the ellipsis y axis must be half the x axis
        Ellipse::painter(center, a, b)
    }
}