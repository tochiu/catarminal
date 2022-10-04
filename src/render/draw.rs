/*
 * draw.rs
 * module of constructs used in drawings
 */
use super::{space::*, anim::*, mount::{MountableLayout, Mount}, shape::BitShape, iter::*};

use tui::{
    buffer::{Buffer, Cell},
    widgets::{Widget, StatefulWidget}, 
    style::Style
};

use unicode_segmentation::UnicodeSegmentation;

pub trait Layoutable {
    fn layout_mut(&mut self) -> &mut DrawLayout;
    fn layout_ref(&self) -> &DrawLayout;

    /* gets the layout in absolute terms using the AbsoluteSpace of the given parent layout */
    fn to_absolute_layout_space(&self, parent_absolute_space: AbsoluteSpace) -> AbsoluteSpace {
        self.layout_ref().space.to_absolute_space(parent_absolute_space)
    }
}

pub trait Drawable: std::fmt::Debug + Layoutable {
    fn draw(&self, ctx: &mut DrawContext);
}

pub trait StatefulDrawable: std::fmt::Debug + Layoutable {
    type State: ?Sized;
    fn stateful_draw(&self, ctx: &mut DrawContext, state: &Self::State);
}

#[derive(Debug)]
pub struct DrawLayout {
    pub is_visible: bool,
    pub space: Space,
    pub anim: Option<Box<SpaceAnimation>>
}

impl Clone for DrawLayout {
    fn clone(&self) -> Self {
        DrawLayout { is_visible: self.is_visible, space: self.space, anim: None }
    }
}

impl Default for DrawLayout {
    fn default() -> Self {
        DrawLayout::FULL
    }
}

impl DrawLayout {
    pub const FULL: DrawLayout = DrawLayout {
        is_visible: true,
        space: Space::FULL,
        anim: None
    };

    pub fn set_visible(&mut self, is_visible: bool) -> &mut Self {
        self.is_visible = is_visible;
        self
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

    pub fn set_anchor(&mut self, anchor: Float2D) -> &mut Self {
        self.space.anchor = anchor;
        self
    }
}

pub type SpaceAnimation = Animation<SpaceAnimator>;

impl SpaceAnimation {
    pub fn new(space0: Space, space1: Space, duration: f32, style: EasingStyle, direction: EasingDirection) -> Self {
        Animation::with_duration(duration, SpaceAnimator { space0, space1, style, direction })
    }
}

/* SpaceAnimator is what is used to animate layout spaces */
#[derive(Debug)]
pub struct SpaceAnimator {
    space0: Space,
    space1: Space,
    style: EasingStyle,
    direction: EasingDirection
}

impl Animator for SpaceAnimator {
    type Target = Space;

    /* update animation state and target space accordingly */
    fn update(&mut self, state: &AnimationState, target: &mut Self::Target) {
        /* update target acoordingly (this still runs when alpha == 1 to put target at goal space) */
        *target = self.space0.lerp(self.space1, ease(state.get_alpha(), self.style, self.direction));
    }
}

/* this struct defines fields and methods necessary for a drawing to draw itself out onto the given space in the given buffer */
pub struct DrawContext<'a> {
    pub absolute_draw_space: AbsoluteSpace,
    pub absolute_layout_space: AbsoluteSpace,
    pub buf: &'a mut Buffer
}

impl<'a> DrawContext<'a> {
    pub fn from_buffer(buf: &'a mut Buffer) -> Self {
        let rect_space = AbsoluteSpace::from_rect(buf.area);
        DrawContext { 
            absolute_draw_space: rect_space, 
            absolute_layout_space: rect_space, 
            buf 
        }
    }

    pub fn retain<F>(&mut self, predicate: F)
        where F: Fn(Point2D) -> bool
    {
        for point in self.absolute_draw_space {
            if !predicate(point) {
                self.buf.get_mut(
                    u16::try_from(point.x).unwrap(), 
                    u16::try_from(point.y).unwrap()
                ).reset();
            }
        }
    }

    #[allow(dead_code)]
    pub fn bitmask(&mut self, mask: &BitShape) {
        let absolute_mask_layout_space = AbsoluteSpace {
            position: self.absolute_layout_space.position,
            size: mask.size
        };

        if let Some(absolute_mask_draw_space) = absolute_mask_layout_space.try_intersection(self.absolute_draw_space) {
            for point in absolute_mask_draw_space {
                let relative_point = absolute_mask_layout_space.relative_position_of(point);
                if !mask.is_filled_at(
                    u16::try_from(relative_point.x).unwrap(), 
                    u16::try_from(relative_point.y).unwrap()
                ) {
                    self.buf.get_mut(
                        u16::try_from(point.x).unwrap(), 
                        u16::try_from(point.y).unwrap()
                    ).reset();
                }
            }
        }
    }

    pub fn overlay(&mut self, overlay: &Buffer) {
        let absolute_overlay_layout_space = AbsoluteSpace {
            position: self.absolute_layout_space.position,
            size: AbsoluteSpace::from_rect(overlay.area).size
        };

        if let Some(absolute_overlay_draw_space) = absolute_overlay_layout_space.try_intersection(self.absolute_draw_space) {
            let default_cell = Cell::default();
            for point in absolute_overlay_draw_space {
                let relative_point = absolute_overlay_layout_space.relative_position_of(point);
                let src = overlay.get(
                    u16::try_from(relative_point.x).unwrap(), 
                    u16::try_from(relative_point.y).unwrap()
                );
                let dst = self.buf.get_mut(
                    u16::try_from(point.x).unwrap(), 
                    u16::try_from(point.y).unwrap()
                );

                if src != &default_cell { // technically wanting cell @ dst to be reset w/ src is indistinguishable from wanting cell @ dst to be left untouched
                    dst.set_symbol(&src.symbol);
                    dst.set_style(src.style());
                }
            }
        }
    }

    pub fn transform(&mut self, absolute_layout_space: AbsoluteSpace) -> &mut Self {
        self.absolute_layout_space = absolute_layout_space;
        self
    }

    pub fn cell_at_mut(&mut self, point: Point2D) -> Option<&mut Cell> {
        let absolute_point = self.absolute_layout_space.absolute_position_of(point);
        if self.absolute_draw_space.is_interior_point(absolute_point) {
            Some(self.buf.get_mut(
                u16::try_from(absolute_point.x).unwrap(), 
                u16::try_from(absolute_point.y).unwrap()
            ))
        } else {
            None
        }
    }

    /* an iterator of the drawable cells of the buffer from top-left to bottom-right */
    pub fn iter_cells_mut(&mut self) -> BufferCellIterMut {
        BufferCellIterMut { buf: self.buf, itr: self.absolute_draw_space.into_iter() }
    }

    /* helper method that iterates through all drawable cells and calls the given closure on a mutable ref of the cell */
    pub fn transform_cells<F>(&mut self, mut f: F) 
        where F: FnMut(&mut Cell)
    {
        let mut itr = self.iter_cells_mut();
        while let Some(cell) = itr.next() {
            f(cell);
        }
    }

    pub fn draw_child<T: Drawable>(&mut self, child: &T) {
        let subarea_absolute_layout_space = child.to_absolute_layout_space(self.absolute_layout_space);

        /* only draw child if any part of it will be displayed */
        if child.layout_ref().is_visible {
            if let Some(subarea_absolute_draw_space) = self.absolute_draw_space.try_intersection(subarea_absolute_layout_space) {
                child.draw(&mut DrawContext {
                    absolute_draw_space: subarea_absolute_draw_space,
                    absolute_layout_space: subarea_absolute_layout_space,
                    buf: self.buf
                });
            }
        }
    }
    
    pub fn draw_children<T: Drawable>(&mut self, children: &[T]) {
        for child in children {
            self.draw_child(child);
        }
    }

    /* carbon copy of draw_child but for StatefulDrawable */
    pub fn draw_stateful_child<T: StatefulDrawable>(&mut self, child: &T, state: &T::State) {
        let subarea_absolute_layout_space = child.to_absolute_layout_space(self.absolute_layout_space);
        if child.layout_ref().is_visible {
            if let Some(subarea_absolute_draw_space) = self.absolute_draw_space.try_intersection(subarea_absolute_layout_space) {
                child.stateful_draw(
                    &mut DrawContext {
                        absolute_draw_space: subarea_absolute_draw_space,
                        absolute_layout_space: subarea_absolute_layout_space,
                        buf: self.buf
                    },
                    state
                );
            }
        }
    }

    pub fn draw_stateful_children<T: StatefulDrawable>(&mut self, children: &[T], states: &[T::State])
        where T::State: Sized 
    {
        for (child, state) in std::iter::zip(children, states) {
            self.draw_stateful_child(child, state);
        }
    }

    pub fn draw_widget<T: Widget>(&mut self, widget: T, layout_space: AbsoluteSpace) {
        let absolute_layout_space = self.absolute_layout_space.absolute_space_of(layout_space);
        if let Some(absolute_draw_space) = absolute_layout_space.try_intersection(self.absolute_draw_space) {
            widget.render(absolute_draw_space.to_rect(), self.buf);
        }
    }

    #[allow(dead_code)]
    pub fn draw_stateful_widget<T: StatefulWidget>(&mut self, stateful_widget: T, mut state: T::State, layout_space: AbsoluteSpace) {
        let absolute_layout_space = self.absolute_layout_space.absolute_space_of(layout_space);
        if let Some(absolute_draw_space) = absolute_layout_space.try_intersection(self.absolute_draw_space) {
            stateful_widget.render(absolute_draw_space.to_rect(), self.buf, &mut state);
        }
    }

    /* draw a string containing on 1 line without unicode characters at the given position relative to layout space of the drawing with the given style*/
    pub fn draw_string_line(&mut self, line: &str, position: Point2D, style: Style) {
        let absolute_line_layout_space = AbsoluteSpace {
            position: self.absolute_layout_space.absolute_position_of(position),
            size: Size2D::new(u16::try_from(line.len()).unwrap_or(u16::MAX), 1)
        };

        if let Some(absolute_line_draw_space) = self.absolute_draw_space.try_intersection(absolute_line_layout_space) {
            let range_lb = (absolute_line_draw_space.left() - absolute_line_layout_space.left()) as usize;
            let range_ub = (absolute_line_draw_space.right() - absolute_line_layout_space.left()) as usize;
            let buf_index = self.buf.index_of(
                absolute_line_draw_space.position.x as u16, 
                absolute_line_draw_space.position.y as u16
            );
            for (i, c) in line[range_lb..range_ub].chars().enumerate() {
                self.buf.content[buf_index + i].set_char(c).set_style(style);
            }
        }
    }

    /* draw a string containing on 1 line with unicode characters at the given position relative to layout space of the drawing with the given style */
    pub fn draw_unicode_line(&mut self, line: &str, pos: Point2D, style: Style) {
        let bufy = self.absolute_layout_space.top() + pos.y;
        let bufx = self.absolute_layout_space.left() + pos.x;

        if 
            bufy >= self.absolute_draw_space.top() && 
            bufy < self.absolute_draw_space.bottom() &&
            bufx < self.absolute_draw_space.right()
        {
            // TODO: refactor this hot garbage
            self.buf.set_stringn(
                (bufx + (self.absolute_draw_space.left() - bufx).max(0)) as u16, 
                bufy as u16, 
                &line.graphemes(true).skip((self.absolute_draw_space.left() - bufx).max(0) as usize).collect::<String>(), 
                (self.absolute_draw_space.right() - bufx).max(0) as usize, 
                style
            );
        }
    }
}

/* mutable iterator for cells in buffer using coordinates from the given AbsoluteSpaceIterator */
pub struct BufferCellIterMut<'a> {
    buf: &'a mut Buffer,
    itr: AbsoluteSpaceIterator    
}

impl<'a> CustomIterator for BufferCellIterMut<'a> {
    type Item = MutRefFamily<Cell>;
    fn next<'s>(&'s mut self) -> Option<<Self::Item as FamilyLt<'s>>::Out> {
        if let Some(point) = self.itr.next() {
            Some(self.buf.get_mut(
                u16::try_from(point.x).unwrap(), 
                u16::try_from(point.y).unwrap()
            ))
        } else {
            None
        }
    }
}

// TODO: find a better way to write this, these structs exist only to satisfy trait bounds of generic drawing structs (ex. Dragger)
// some generic drawings require a StatefulDrawable + MountableLayout in order to pass down state / events but the struct doesnt need that so an
// "adapter" generic struct can be sandwiched between the two to drop said state / event
// alternatively can be used as simple containers
// thats what these next 4 structs are
// DrawDuplex contains MountableLayout + StatefulDrawable
// DrawPasser contains StatefulDrawable
// DrawBranch contains MountableLayout + Drawable
// DrawLeaf contains Drawable

#[derive(Debug)]
pub struct DrawDuplex<T: MountableLayout + StatefulDrawable> {
    pub drawing: T,
    pub layout: DrawLayout,
    mount: Mount,
}

impl<T: MountableLayout + StatefulDrawable> DrawDuplex<T> {
    #[allow(dead_code)]
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
    fn stateful_draw(&self, ctx: &mut DrawContext, state: &Self::State) {
        ctx.draw_stateful_child(&self.drawing, state);
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
    #[allow(dead_code)]
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
    fn stateful_draw(&self, ctx: &mut DrawContext, state: &Self::State) {
        ctx.draw_stateful_child(&self.drawing, state);
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
    #[allow(dead_code)]
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
    fn draw(&self, ctx: &mut DrawContext) {
        ctx.draw_child(&self.drawing);
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
    fn draw(&self, ctx: &mut DrawContext) {
        ctx.draw_child(&self.drawing);
    }
}

impl<T: Drawable + 'static> MountableLayout for DrawLeaf<T> {
    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }
    fn child_ref(&self, _: usize) -> Option<&dyn MountableLayout> { None }
    fn child_mut(&mut self, _: usize) -> Option<&mut dyn MountableLayout> { None }
}