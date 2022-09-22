use super::{
    space::*,
    screen::*, 
    anim::SpaceAnimation, 
    mount::{MountableLayout, Mount}
};

pub struct NoDrawState;

pub trait Layoutable {
    fn layout_mut(&mut self) -> &mut DrawLayout;
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

#[derive(Debug, Default, Clone)]
pub struct DrawLayout {
    pub space: Space,
    pub anim: Option<Box<SpaceAnimation>>
}

impl DrawLayout {
    pub const FULL: DrawLayout = DrawLayout {
        space: Space::FULL,
        anim: None
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

#[derive(Debug)]
pub struct DrawContainer<T: MountableLayout + StatefulDrawable> {
    pub drawing: T,
    pub layout: DrawLayout,
    mount: Mount,
}

impl<T: MountableLayout + StatefulDrawable> DrawContainer<T> {
    pub fn new(drawing: T) -> Self {
        DrawContainer {
            drawing,
            layout: DrawLayout::FULL,
            mount: Mount::default()
        }
    }
}

impl<T: MountableLayout + StatefulDrawable> Layoutable for DrawContainer<T> {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl<T: MountableLayout + StatefulDrawable> StatefulDrawable for DrawContainer<T> {
    type State = T::State;
    fn stateful_draw(&self, mut area: ScreenArea, state: &Self::State) {
        area.draw_stateful_child(&self.drawing, state);
    }
}

impl<T: MountableLayout + StatefulDrawable> MountableLayout for DrawContainer<T> {
    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }

    fn child_ref(&self, i: usize) -> Option<&dyn MountableLayout> {
        match i {
            0 => Some(self.drawing.as_trait_ref()),
            _ => None
        }
    }

    fn child_mut(&mut self, i: usize) -> Option<&mut dyn MountableLayout> {
        match i {
            0 => Some(self.drawing.as_trait_mut()),
            _ => None
        }
    }
}