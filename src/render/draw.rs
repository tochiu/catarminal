use super::{
    space::*,
    screen::*
};

pub struct NoDrawState;

pub trait Layoutable {
    fn layout_ref(&self) -> &DrawLayout;
    fn to_absolute_layout_space(&self, parent_absolute_space: AbsoluteSpace) -> AbsoluteSpace {
        self.layout_ref().space.to_absolute_space(parent_absolute_space)
    }
}

pub trait Drawable: std::fmt::Debug + Layoutable {
    fn draw(&self, area: ScreenArea);
}

pub trait StatefulDrawable: std::fmt::Debug + Layoutable {
    type State;
    fn stateful_draw(&self, area: ScreenArea, state: &Self::State);
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