use super::{
    space::*,
    world::*
};

use tui::{
    buffer::Buffer,
    style::Style
};

#[derive(Debug, Default, Clone, Copy)]
pub struct DrawLayout {
    pub space: Space
}

impl DrawLayout {
    pub const FULL: DrawLayout = DrawLayout {
        space: Space::FULL
    };

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

pub trait Drawing: std::fmt::Debug {
    fn draw(&self, area: WorldArea);
    fn layout_ref(&self) -> &DrawLayout;
}

pub trait DrawBuffer {
    fn draw_lines<S>(&mut self, lines: &Vec<S>, draw_space: AbsoluteSpace, full_space: AbsoluteSpace, style: Style)
        where S: AsRef<str>;
}

impl DrawBuffer for Buffer {
    fn draw_lines<S>(&mut self, lines: &Vec<S>, draw_space: AbsoluteSpace, full_space: AbsoluteSpace, style: Style)
        where S: AsRef<str>
    {
        let offset_y = draw_space.position.y - full_space.position.y;
        let offset_x = draw_space.position.x - full_space.position.x;
        for y in 0..lines.len().min(draw_space.size.y as usize) as i16 {
            let point = draw_space.absolute_position_of(Point2D::new(0, y as i16));
            self.set_stringn(point.x as u16, point.y as u16, &lines[(y + offset_y) as usize].as_ref()[offset_x as usize..], draw_space.size.x as usize, style);
        }
    }
}