use super::{
    space::*,
    world::*
};

pub trait Layoutable {
    fn layout_ref(&self) -> &DrawLayout;
}

pub trait Drawable: std::fmt::Debug + Layoutable {
    fn draw(&self, area: WorldArea);
}

pub trait StatefulDrawable: std::fmt::Debug + Layoutable {
    type State;
    fn stateful_draw(&self, area: WorldArea, state: &Self::State);
}

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