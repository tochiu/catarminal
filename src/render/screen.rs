/*
 * screen.rs
 * a module of constructs that a relevant in the behavior of a screen
 * this includes layouting, rendering, input capture, and animation
 */
use super::prelude::*;
use super::iter::CustomIterator;

use tui::{
    layout::Rect,
    buffer::Buffer,
    widgets::StatefulWidget
};

/* 
 * Screen:
 * 
 * structs that can be expressed as a render struct implementing trait StatefulWidget 
 * and therefore capable of rendering out to an area (fullscreen if log is disabled) 
 * defined by frame.render_stateful_widget using the supplied root drawing
 * 
 * a screen controls input capture and pokes mountable drawings listening for input
 * the root drawing "T" must be capable of receiving input events from the screen (impl trait MountableLayout)
 * 
 * all screens will receive state passed from frame.render_stateful_widget 
 * the root drawing "T"  must be capable of receiving state (impl trait StatefulDrawable)
 * 
 * NOTE:
 * a behavior that optimizes some drawing is "clipping descendants" which means that child layouts cannot draw themselves 
 * or capture input in an area outside their parent area
 */
#[derive(Debug)]
pub struct Screen<T: MountableLayout + StatefulDrawable> {
    pub root: T,
    pub service: ScreenService
}

/*
 * ScreenService:
 * organizational struct that contains all services accessible to MountableLayouts during relayout
 */
#[derive(Debug, Default)]
pub struct ScreenService {
    pub input: InputService,
    pub animation: AnimationService
}

impl<T: MountableLayout + StatefulDrawable> Screen<T> {
    pub fn new(mut root: T) -> Self {
        root.mount(Mount::default());

        Screen {
            root,
            service: ScreenService::default()
        }
    }

    /* 
     * express Screen as a ScreenWidget which implements StatefulWidget
     * 
     * why create this indirection instead of having Screen directly implement StatefulWidget?
     * 
     * because frame.render_stateful_widget takes OWNERSHIP of the widget it draws out the screen
     * tui has philosphy of "immediate rendering" where renderable structs should be created on demand
     * this is beginner-friendly as it avoids borrow checker complications 
     * 
     * but this isnt the design philosophy of this render module since there is quite a bit of moving parts
     * and would lead to a bloated state struct in addition to making me sqeamish about the performance implications
     * 
     * there is a balance between how much performance-loss is acceptable and how much time I want to spend on this project
     * which is why I am using tui to begin with
     */
    pub fn as_stateful_widget(&mut self) -> ScreenWidget<T> {
        ScreenWidget {
            screen: self
        }
    }

    /* execute a relayout on the root struct */
    fn relayout_root(&mut self, absolute_screen_space: AbsoluteSpace) {
        LayoutContext::relayout(&mut self.root, absolute_screen_space, absolute_screen_space, &mut self.service);
        self.service.animation.cleanup();
    }

    /* draw the root struct with the given state inside the given buffer */
    fn draw_root(&self, absolute_screen_space: AbsoluteSpace, buf: &mut Buffer, state: &<T as StatefulDrawable>::State) {
        DrawContext::draw_stateful_child(
            &mut DrawContext {
                absolute_draw_space: absolute_screen_space,
                absolute_layout_space: absolute_screen_space,
                buf
            }, 
            &self.root, 
            state
        );
    }
}

/* struct that implements StatefulWidget to draw its given screen with frame.draw_stateful_widget */
pub struct ScreenWidget<'a, T: MountableLayout + StatefulDrawable> {
    screen: &'a mut Screen<T>
}

/* StatefulWidget impl */
impl<'a, T: MountableLayout + StatefulDrawable> StatefulWidget for ScreenWidget<'a, T>
    where T::State: Sized 
{
    type State = T::State;
    fn render(self, rect: Rect, buf: &mut Buffer, state: &mut T::State) {
        let absolute_screen_space = AbsoluteSpace::from_rect(rect);

        /* 
         * allow drawings to finalize their drawing area 
         * drawings that receive input redefine their capture areas
         * drawings that animate update their state and layout based on said animation state
         * drawings can update any relevant state before being drawn
         */
        self.screen.service.input.invalidate_all_inputs();
        self.screen.relayout_root(absolute_screen_space);

        /* draw the screen */
        self.screen.draw_root(absolute_screen_space, buf, &state);
    }
}

/* This struct defines fields and methods necessary for a given mountable to define their layout */
#[derive(Debug)]
pub struct LayoutContext<'a> {
    pub id: MountId,
    pub parent_absolute_draw_space: AbsoluteSpace,
    pub parent_absolute_layout_space: AbsoluteSpace,
    pub service: &'a mut ScreenService
}

impl<'a> LayoutContext<'a> {
    /* 
     * execute the relayout operation
     * because all layouts are animatable it will handle updating the animation state
     * then call the custom relayout function defined by the mountable
     */
    fn relayout(
        mountable: &mut dyn MountableLayout, 
        parent_absolute_draw_space: AbsoluteSpace, 
        parent_absolute_layout_space: AbsoluteSpace, 
        service: &'a mut ScreenService
    ) {
        let id = mountable.mount_ref().id;
        let layout = mountable.layout_mut();
        if let Some(anim) = layout.anim.as_deref_mut() {
            anim.update(&mut layout.space);
            if anim.state.playback != PlaybackState::Playing {
                layout.anim = None;
            }
        }

        mountable.relayout(&mut LayoutContext {
            id,
            parent_absolute_draw_space,
            parent_absolute_layout_space,
            service
        });
    }

    /*
     * Restricts space of the given Layoutable to not exceed the bounds of the parent's absolute draw space
     * Returns Option because the restrictions could lead to no drawable space (layout out of bounds of parent drawing space)
     */
    pub fn get_absolute_draw_space_of(&self, subarea_absolute_layout_space: AbsoluteSpace) -> Option<AbsoluteSpace> {
        self.parent_absolute_draw_space.try_intersection(subarea_absolute_layout_space)
    }

    pub fn get_absolute_layout_space_of(&self, layoutable: &dyn Layoutable) -> AbsoluteSpace {
        layoutable.to_absolute_layout_space(self.parent_absolute_layout_space)
    }

    pub fn get_absolute_size_of(&self, layoutable: &dyn Layoutable) -> Size2D {
        self.get_absolute_layout_space_of(layoutable).size
    }

    /* relayout the children of the given mountable (reads as relayout.relayout_children_of(mountable)) */
    pub fn relayout_children_of(&mut self, mountable: &mut dyn MountableLayout) {
        let absolute_layout_space = mountable.to_absolute_layout_space(self.parent_absolute_layout_space);
        self.relayout_children_in_space_of(mountable, absolute_layout_space);
    }

    /* 
     * relayout the children of the given mountable using a given AbsoluteSpace as the parent layout space 
     * this is useful for parents that want to redefine the parent space the child space will relayout to
     * e.g. draggers and scrollbars will draw children in some "canvas space" that can be bigger than the containing "window space"
     * relayout_children_of is this but just uses the layout space of the parent is what is used to draw the children
     */
    pub fn relayout_children_in_space_of(&mut self, mountable: &mut dyn MountableLayout, new_parent_absolute_layout_space: AbsoluteSpace) {
        let absolute_layout_space = mountable.to_absolute_layout_space(self.parent_absolute_layout_space);
        
        // TODO: find a way to cull relayout operations because the current way animations work, 
        // TODO: culling unseen animations from a relayout will never give them an opportunity to notify the screen when they are done
        // TODO: this leads to overdrawing as the screen thinks its animating some area when it is actually being culled
        // TODO: a simple solution would to be to specify the duration of each animation so the screen can remove them itself
        let absolute_draw_space = self.get_absolute_draw_space_of(absolute_layout_space).unwrap_or(AbsoluteSpace { 
            position: absolute_layout_space.position, 
            size: Size2D::new(0, 0) 
        });

        //if let Some(absolute_draw_space) = self.get_absolute_draw_space_of(absolute_layout_space) {
        let mut itr = mountable.child_iter_mut();
        while let Some(child) = itr.next() {
            LayoutContext::relayout(child, absolute_draw_space, new_parent_absolute_layout_space, self.service);
        }
        //}
    }

    /* reserve a subregion of the layout space for capturing input (reads as relayout.relayout_input_space_of(mountable)) */
    pub fn relayout_input_space_of(&mut self, mountable: &mut dyn MountableLayout, input_space: Space) {
        if !mountable.layout_ref().is_visible {
            return
        }
        
        let absolute_layout_space = mountable.to_absolute_layout_space(self.parent_absolute_layout_space);
        if let Some(absolute_draw_space) = self.get_absolute_draw_space_of(absolute_layout_space) {
            if let Some(absolute_interactable_input_space) = 
                input_space
                    .to_absolute_space(absolute_layout_space)
                    .try_intersection(absolute_draw_space) 
            {
                self.service.input.capture_space(self.id, absolute_interactable_input_space);
            }
        }
    }
}