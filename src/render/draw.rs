use std::marker::PhantomData;
use std::any::Any;

use super::{
    space::*,
    world::*
};

use tui::{
    buffer::Buffer,
    style::Style
};

#[derive(Debug)]
pub struct DrawingCanvas<'a> {
    pub id: Option<DrawId>,
    pub draw_space: AbsoluteSpace,
    pub full_space: AbsoluteSpace,
    pub buf: &'a mut Buffer,
    pub world: &'a WorldCanvas
}

impl<'a> DrawingCanvas<'a> {
    pub fn draw_child(&mut self, child_id: DrawId) {
        let child_drawing = self.world.get_dyn(child_id);

        let full_space = child_drawing.space.to_absolute_space(self.full_space);
        if !full_space.intersects(self.draw_space) {
            return
        }
        
        child_drawing.pencil.draw(DrawingCanvas {
            id: Some(child_drawing.id),
            draw_space: full_space.intersection(self.draw_space),
            full_space,
            buf: self.buf,
            world: self.world
        });
    }

    pub fn draw_children(&mut self) {
        for child_id in self.world.iter_children(self.id.unwrap()) {
            self.draw_child(child_id);
        }
    }
}

pub trait Drawable: std::fmt::Debug + 'static {
    fn draw(&self, canvas: DrawingCanvas);

    #[allow(unused_variables)]
    fn on_mount(dref: &mut Drawing<Self>, controller: &mut MountController)
        where Self: Sized 
    {}
}

pub type Drawing<T> = Draw<Box<T>>;

#[derive(Debug)]
pub struct Draw<T> {
    pub id: DrawId,
    pub space: Space,
    pub pencil: T
}

impl<T: Drawable> Drawing<T> {

    pub fn new(pencil: T, id: DrawId) -> Self {
        Draw {
            id,
            space: Space::FULL, 
            pencil: Box::new(pencil) 
        }
    }

    pub fn as_dref(&self) -> DrawRef<T> {
        DrawRef {
            id: self.id,
            tp: PhantomData
        }
    }

    pub fn as_dyn(self) -> Drawing<dyn Drawable> {
        Draw {
            id: self.id,
            space: self.space, 
            pencil: self.pencil as Box<dyn Drawable>
        }
    }

    pub fn into_any(self) -> Drawing<dyn Any> {
        Draw {
            id: self.id,
            space: self.space, 
            pencil: self.pencil as Box<dyn Any>
        }
    }
}

impl<T: Drawable + ?Sized> Drawing<T> {
    pub fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    pub fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
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