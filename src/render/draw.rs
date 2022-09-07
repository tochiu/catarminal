use std::any::Any;

use super::{
    space::*,
    world::{WorldArea, WorldMountController}
};

use tui::{
    buffer::Buffer,
    style::Style
};

#[derive(Debug, Default)]
pub struct DrawLayout {
    pub space: Space
}

impl DrawLayout {
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

pub trait Drawable: std::fmt::Debug + AsAny + 'static {
    fn draw(&self, area: WorldArea);

    #[allow(unused_variables)]
    fn on_mount(mut controller: WorldMountController)
        where Self: Sized 
    {}
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Drawable + 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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