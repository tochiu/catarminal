use tui::{
    widgets::Widget,
    layout::Rect,
    buffer::Buffer,
    style::Style
};

use super::space::*;

pub struct DrawCanvas<'a> {
    pub draw_space: AbsoluteSpace,
    pub full_space: AbsoluteSpace,
    pub buf: &'a mut Buffer
}

impl<'a> DrawCanvas<'a> {
    pub fn new(draw_space: AbsoluteSpace, full_space: AbsoluteSpace, buf: &'a mut Buffer) -> Self {
        DrawCanvas {
            draw_space,
            full_space,
            buf
        }
    }
}

pub trait Drawable {
    fn draw(&self, canvas: &mut DrawCanvas);
    fn get_space(&self) -> Space {
        Space::FULL
    }
}

pub struct Drawing<T: Drawable> {
    pub space: Space,
    pub pencil: T
}

impl<T: Drawable> Drawing<T> {
    pub fn a(pencil: T) -> Self {
        Drawing { space: pencil.get_space(), pencil }
    }

    pub fn to_widget(&self) -> DrawingWidget<T> {
        DrawingWidget {
            drawing: self
        }
    }

    pub fn draw_in(&self, canvas: &mut DrawCanvas) {
        // draw entity if it intersects with the drawable space
        let full_space = self.space.to_absolute_space(canvas.full_space);
        if full_space.intersects(canvas.draw_space) {
            self.pencil.draw(&mut DrawCanvas::new(
                full_space.intersection(canvas.draw_space),
                full_space,
                canvas.buf
            ));
        }
    }

    pub fn center(&mut self) -> &mut Self {
        self.set_space(self.space.center())
    }

    pub fn set_space(&mut self, space: Space) -> &mut Self {
        self.space = space;
        self
    }

    pub fn set_position(&mut self, position: UDim2) -> &mut Self {
        self.space.position = position;
        self
    }

    pub fn set_size(&mut self, size: UDim2) -> &mut Self {
        self.space.size = size;
        self
    }

    pub fn set_anchor(&mut self, anchor: Scale2D) -> &mut Self {
        self.space.anchor = anchor;
        self
    }
}

pub struct DrawingWidget<'a, T: Drawable> {
    drawing: &'a Drawing<T>
}

impl<'a, T: Drawable> Widget for DrawingWidget<'a, T> {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let rect_space = AbsoluteSpace::from_rect(rect);
        self.drawing.draw_in(&mut DrawCanvas::new(rect_space, rect_space, buf));
    }
}

pub trait DrawBuffer {
    fn draw_lines<S>(&mut self, lines: &Vec<S>, space: AbsoluteSpace, style: Style)
        where S: AsRef<str>;
}

impl DrawBuffer for Buffer {
    fn draw_lines<S>(&mut self, lines: &Vec<S>, space: AbsoluteSpace, style: Style)
        where S: AsRef<str>
    {
        let height = lines.len().min(space.size.y as usize) as u16;
        for y in 0..height {
            let point = space.absolute_position_of(Point2D::new(0, y as i16));
            self.set_stringn(point.x as u16, point.y as u16, &lines[y as usize], space.size.x as usize, style);
        }
    }
}