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
    type State: ?Sized;
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
pub struct DrawDuplex<T: MountableLayout + StatefulDrawable> {
    pub drawing: T,
    pub layout: DrawLayout,
    mount: Mount,
}

impl<T: MountableLayout + StatefulDrawable> DrawDuplex<T> {
    pub fn new(drawing: T, layout: DrawLayout) -> Self {
        DrawDuplex {
            drawing,
            layout,
            mount: Mount::default()
        }
    }
}

impl<T: MountableLayout + StatefulDrawable> Layoutable for DrawDuplex<T> {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl<T: MountableLayout + StatefulDrawable> StatefulDrawable for DrawDuplex<T> {
    type State = T::State;
    fn stateful_draw(&self, mut area: ScreenArea, state: &Self::State) {
        area.draw_stateful_child(&self.drawing, state);
    }
}

impl<T: MountableLayout + StatefulDrawable> MountableLayout for DrawDuplex<T> {
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

#[derive(Debug)]
pub struct DrawPasser<T: StatefulDrawable + 'static> {
    pub drawing: T,
    pub layout: DrawLayout,
    mount: Mount,
}

impl<T: StatefulDrawable + 'static> DrawPasser<T> {
    pub fn new(drawing: T, layout: DrawLayout) -> Self {
        DrawPasser {
            drawing,
            layout,
            mount: Mount::default()
        }
    }
}

impl<T: StatefulDrawable + 'static> Layoutable for DrawPasser<T> {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl<T: StatefulDrawable + 'static> StatefulDrawable for DrawPasser<T> {
    type State = T::State;
    fn stateful_draw(&self, mut area: ScreenArea, state: &Self::State) {
        area.draw_stateful_child(&self.drawing, state);
    }
}

impl<T: StatefulDrawable + 'static> MountableLayout for DrawPasser<T> {
    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }
    fn child_ref(&self, _: usize) -> Option<&dyn MountableLayout> { None }
    fn child_mut(&mut self, _: usize) -> Option<&mut dyn MountableLayout> { None }
}

#[derive(Debug)]
pub struct DrawBranch<T: MountableLayout + Drawable> {
    pub drawing: T,
    pub layout: DrawLayout,
    mount: Mount,
}

impl<T: MountableLayout + Drawable> DrawBranch<T> {
    pub fn new(drawing: T, layout: DrawLayout) -> Self {
        DrawBranch {
            drawing,
            layout,
            mount: Mount::default()
        }
    }
}

impl<T: MountableLayout + Drawable> Layoutable for DrawBranch<T> {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl<T: MountableLayout + Drawable> Drawable for DrawBranch<T> {
    fn draw(&self, mut area: ScreenArea) {
        area.draw_child(&self.drawing);
    }
}

impl<T: MountableLayout + Drawable> MountableLayout for DrawBranch<T> {
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

#[derive(Debug)]
pub struct DrawLeaf<T: Drawable + 'static> {
    pub drawing: T,
    pub layout: DrawLayout,
    mount: Mount,
}

impl<T: Drawable + 'static> DrawLeaf<T> {
    pub fn new(drawing: T, layout: DrawLayout) -> Self {
        DrawLeaf {
            drawing,
            layout,
            mount: Mount::default()
        }
    }
}

impl<T: Drawable + 'static> Layoutable for DrawLeaf<T> {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl<T: Drawable + 'static> Drawable for DrawLeaf<T> {
    fn draw(&self, mut area: ScreenArea) {
        area.draw_child(&self.drawing);
    }
}

impl<T: Drawable + 'static> MountableLayout for DrawLeaf<T> {
    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }
    fn child_ref(&self, _: usize) -> Option<&dyn MountableLayout> { None }
    fn child_mut(&mut self, _: usize) -> Option<&mut dyn MountableLayout> { None }
}