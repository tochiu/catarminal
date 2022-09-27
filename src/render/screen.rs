/*
 * screen.rs
 * a module of constructs that a relevant in the behavior of a screen
 * this includes layouting, rendering, input capture, and animation
 */

use super::{
    draw::*,
    space::*,
    mount::*,
    iter::*, anim::Animatable
};

use tui::{
    layout::Rect,
    buffer::{Buffer, Cell},
    widgets::{Widget, StatefulWidget}, style::Style
};
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
use unicode_segmentation::UnicodeSegmentation;

/* 
 * Screen:
 * 
 * structs that can be expressed as a render struct implementing StatefulWidget 
 * and therefore capable of rendering out to an area (fullscreen if log is disabled) 
 * defined by frame.render_stateful_widget using the supplied root drawing
 * 
 * a screen controls input capture and pokes mountable drawings listening for input
 * and therefore the root drawing "T" must be capable of receiving input events from the screen (MountableLayout)
 * 
 * all screens will receive state passed from frame.render_stateful_widget 
 * and therefore the root drawing "T"  must be capable of receiving state (StatefulDrawable)
 * 
 * NOTE:
 * a behavior that optimizes some drawing is "clipping descendants" which means that child layouts cannot draw themselves 
 * or capture input in an area outside their parent area
 */

// TODO: Place input & animation in a struct so they are easily passed to drawings in the root
// TODO: Write ScreenAnimationService so it doesn't need to rely on drawings notifying it when an animation is done
// TODO: ^ Motivation for this is when drawings get replaced we want to clear everything the Screen is keeping track for them
#[derive(Debug)]
pub struct Screen<T: MountableLayout + StatefulDrawable> {
    pub root: T,
    pub input: ScreenInputService,
    pub animation: ScreenAnimationService
}

impl<T: MountableLayout + StatefulDrawable> Screen<T> {
    pub fn new(mut root: T) -> Self {
        root.mount(Mount::default());

        Screen {
            root,
            input: ScreenInputService::default(),
            animation: ScreenAnimationService::default()
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
        ScreenRelayout {
            id: self.root.mount_ref().id,
            parent_absolute_draw_space: absolute_screen_space,
            parent_absolute_layout_space: absolute_screen_space,
            input: &mut self.input,
            animation: &mut self.animation
        }.execute_on(&mut self.root);
    }

    /* draw the root struct with the given state inside the given buffer */
    fn draw_root(&self, absolute_screen_space: AbsoluteSpace, buf: &mut Buffer, state: &<T as StatefulDrawable>::State) {
        ScreenArea::draw_stateful_child(
            &mut ScreenArea {
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
        self.screen.input.invalidate_all_inputs();
        self.screen.relayout_root(absolute_screen_space);

        /* draw the screen */
        self.screen.draw_root(absolute_screen_space, buf, &state);
    }
}

/* This struct defines fields and methods necessary for a given mountable to define their layout */
#[derive(Debug)]
pub struct ScreenRelayout<'a> {
    pub id: MountId,
    pub parent_absolute_draw_space: AbsoluteSpace,
    pub parent_absolute_layout_space: AbsoluteSpace,
    pub input: &'a mut ScreenInputService,
    pub animation: &'a mut ScreenAnimationService
}

impl<'a> ScreenRelayout<'a> {
    /* 
     * execute the relayout operation
     * because all layouts are animatable it will handle updating the animation state
     * then call the custom relayout function defined by the mountable
     */
    pub fn execute_on(&mut self, mountable: &mut dyn MountableLayout) {
        let layout = mountable.layout_mut();
        if let Some(anim) = layout.anim.as_mut() {
            anim.step(&mut layout.space, self.animation);
        }

        mountable.relayout(self);
    }

    /*
     * Restricts space of the given Layoutable to not exceed the bounds of the parent's absolute draw space
     * Returns Option because the restrictions could lead to no drawable space (layout out of bounds of parent drawing space)
     */
    pub fn get_draw_space_of(&self, subarea_absolute_layout_space: AbsoluteSpace) -> Option<AbsoluteSpace> {
        self.parent_absolute_draw_space.try_intersection(subarea_absolute_layout_space)
    }

    pub fn get_absolute_layout_of(&self, layoutable: &dyn Layoutable) -> AbsoluteSpace {
        layoutable.to_absolute_layout_space(self.parent_absolute_layout_space)
    }

    /* relayout the children of the given mountable (reads as relayout.children_of(mountable)) */
    pub fn children_of(&mut self, mountable: &mut dyn MountableLayout) {
        let absolute_layout_space = mountable.to_absolute_layout_space(self.parent_absolute_layout_space);
        self.children_in_space_of(mountable, absolute_layout_space);
    }

    /* 
     * relayout the children of the given mountable using a given AbsoluteSpace as the parent layout space 
     * this is useful for parents that want to redefine the parent space the child space will relayout to
     * e.g. draggers and scrollbars will draw children in some "canvas space" that can be bigger than the containing "window space"
     * children_of is this but just uses the layout space of the parent is what is used to draw the children
     */
    pub fn children_in_space_of(&mut self, mountable: &mut dyn MountableLayout, new_parent_absolute_layout_space: AbsoluteSpace) {
        let absolute_layout_space = mountable.to_absolute_layout_space(self.parent_absolute_layout_space);
        
        // TODO: find a way to cull relayout operations because the current way animations work, 
        // TODO: culling unseen animations from a relayout will never give them an opportunity to notify the screen when they are done
        // TODO: this leads to overdrawing as the screen thinks its animating some area when it is actually being culled
        // TODO: a simple solution would to be to specify the duration of each animation so the screen can remove them itself
        let absolute_draw_space = self.get_draw_space_of(absolute_layout_space).unwrap_or(AbsoluteSpace { 
            position: absolute_layout_space.position, 
            size: Size2D::new(0, 0) 
        });

        //if let Some(absolute_draw_space) = self.get_draw_space_of(absolute_layout_space) {
        let mut itr = mountable.child_iter_mut();
        while let Some(child) = itr.next() {
            ScreenRelayout {
                id: child.mount_ref().id,
                parent_absolute_draw_space: absolute_draw_space,
                parent_absolute_layout_space: new_parent_absolute_layout_space,
                input: self.input,
                animation: self.animation
            }.execute_on(child);
        }
        //}
    }

    /* reserve a subregion of the layout space for capturing input (reads as relayout.input_space_of(mountable)) */
    pub fn input_space_of(&mut self, mountable: &mut dyn MountableLayout, input_space: Space) {
        if !mountable.layout_ref().is_visible {
            return
        }
        
        let absolute_layout_space = mountable.to_absolute_layout_space(self.parent_absolute_layout_space);
        if let Some(absolute_draw_space) = self.get_draw_space_of(absolute_layout_space) {
            if let Some(absolute_interactable_input_space) = 
                input_space
                    .to_absolute_space(absolute_layout_space)
                    .try_intersection(absolute_draw_space) 
            {
                self.input.capture_space(self.id, absolute_interactable_input_space);
            }
        }
    }
}

/* this struct defines fields and methods necessary for a drawing to draw itself out onto the given space in the given buffer */
pub struct ScreenArea<'a> {
    pub absolute_draw_space: AbsoluteSpace,
    pub absolute_layout_space: AbsoluteSpace,
    pub buf: &'a mut Buffer
}

impl<'a> ScreenArea<'a> {
    pub fn transform(self, absolute_layout_space: AbsoluteSpace) -> ScreenArea<'a> {
        ScreenArea { absolute_layout_space, ..self }
    }

    pub fn mut_cell_at(&mut self, point: Point2D) -> Option<&mut Cell> {
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
    pub fn iter_cells_mut(&mut self) -> ScreenCellIterMut {
        ScreenCellIterMut { buf: self.buf, itr: self.absolute_draw_space.into_iter() }
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
                child.draw(ScreenArea {
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
                    ScreenArea {
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

    pub fn draw_widget<T: Widget>(&mut self, widget: T, rect: Rect) {
        widget.render(rect, self.buf)
    }

    pub fn draw_stateful_widget<T: StatefulWidget>(&mut self, widget: T, mut state: T::State, rect: Rect) {
        widget.render(rect, self.buf, &mut state)
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
pub struct ScreenCellIterMut<'a> {
    buf: &'a mut Buffer,
    itr: AbsoluteSpaceIterator    
}

impl<'a> CustomIterator for ScreenCellIterMut<'a> {
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

/* struct that stores the id of the mount that requested input capture at the defined space */
#[derive(Debug, PartialEq, Eq)]
pub struct Input {
    id: MountId,
    space: AbsoluteSpace
}

/* this struct is what stores all requested input capture spaces and calculates and handles input events */
#[derive(Debug, Default)]
pub struct ScreenInputService {
    inputs: Vec<Input>,
    capturing_mount_id: Option<MountId>,
    did_mouse_move: bool,
    input_event_queue: Vec<ScreenInputEvent>
}

/* this struct details what mount triggered what input and is what is passed onto the input_event_queue */
#[derive(Debug)]
pub struct ScreenInputEvent {
    mount_id: MountId,
    pub kind: ScreenInputEventKind
}

/* the kind of input events that can occur with specific input information wrapped inside */
#[derive(Debug)]
pub enum ScreenInputEventKind {
    Click(Point2D),
    Drag(Point2D),
    Move(Point2D),
    Down(Point2D),
    Up(Point2D)
}

impl ScreenInputService {
    /* 
     * invalidating inputs is necessary because drawings can change or be removed between render calls 
     * so drawings must redefine their input space between each time to keep it alive
     */
    fn invalidate_all_inputs(&mut self) {
        self.inputs.clear();
    }

    pub fn capture_space(&mut self, id: MountId, space: AbsoluteSpace) {
        self.inputs.push(Input { id, space });
    }

    /* 
     * get the id of the top-most id that is capturing input at the given point 
     * NOTE: inputs in the vec are considered above the ones before them
     */
    fn query(&self, point: Point2D) -> Option<MountId> {
        for input in self.inputs.iter().rev() {
            let id = input.id;
            let space = input.space;
            if space.is_interior_point(point) {
                return Some(id);
            }
        }

        None
    }

    /*
     * handles a crossterm mouse event, returns a boolean indicating if the screen needs to be rendered again
     */
    pub fn handle_mouse_input(&mut self, event: MouseEvent, root: &mut dyn MountableLayout) -> bool {
        /* get the point the mouse event occured... */
        let point = Point2D::new(
            i16::try_from(event.column).unwrap(), 
            i16::try_from(event.row).unwrap()
        );

        /* ...with a query of the id of the mount that would capture it */
        let maybe_id = self.query(point);

        /* how we handle the event depends on our internal state and the event kind */
        match event.kind {
            MouseEventKind::Down(button) => {
                /* do nothing if its middle mouse or the capturing mount is the same */
                if button == MouseButton::Middle || maybe_id == self.capturing_mount_id {
                    return false
                }

                /* if there is a previously capturing mount, queue a mouse up event for it */
                if let Some(old_id) = self.capturing_mount_id {
                    self.input_event_queue.push(ScreenInputEvent { 
                        mount_id: old_id,
                        kind: ScreenInputEventKind::Up(point)
                    });
                }

                /* if we queried a capturing mount, queue up a mouse down event for it */
                if let Some(id) = maybe_id {
                    self.input_event_queue.push(ScreenInputEvent { 
                        mount_id: id, 
                        kind: ScreenInputEventKind::Down(point) 
                    });
                }

                self.capturing_mount_id = maybe_id;
                self.did_mouse_move = false; // mouse hasnt moved yet so reset
            },
            MouseEventKind::Drag(button) => {
                /* do nothing if its middle mouse or we dont have a currently capturing mount */
                if button == MouseButton::Middle || self.capturing_mount_id.is_none() {
                    return false
                }

                self.did_mouse_move = true;

                /* push drag event for currently capturing mount */
                self.input_event_queue.push(ScreenInputEvent { 
                    mount_id: self.capturing_mount_id.unwrap(), 
                    kind: ScreenInputEventKind::Drag(point)
                });
            },
            /* mouse move is mouse drag but without the held down click */
            MouseEventKind::Moved => {
                /* if there is a currently capturing mount then queue up a drag instead */
                if let Some(capturing_mount_id) = self.capturing_mount_id {
                    self.did_mouse_move = true;
                    self.input_event_queue.push(ScreenInputEvent {
                        mount_id: capturing_mount_id, 
                        kind: ScreenInputEventKind::Drag(point)
                    });
                /* otherwise if we queried a capturing mount, queue up a mouse move event for it */
                } else if let Some(id) = maybe_id {
                    self.input_event_queue.push(ScreenInputEvent { 
                        mount_id: id, 
                        kind: ScreenInputEventKind::Move(point)
                    });
                }
            }
            MouseEventKind::Up(button) => {
                /* do nothing if its middle mouse or the capturing mount is the same */
                if button == MouseButton::Middle || self.capturing_mount_id.is_none() {
                    return false
                }
                /* queue up a mouse up event for the currently capturing mount */
                self.input_event_queue.push(ScreenInputEvent { 
                    mount_id: self.capturing_mount_id.unwrap(), 
                    kind: ScreenInputEventKind::Up(point)
                });
                /* if the mouse didnt move at all, we know it was a click so queue that up */
                if self.capturing_mount_id == self.query(point) {
                    self.input_event_queue.push(ScreenInputEvent { 
                        mount_id: self.capturing_mount_id.unwrap(), 
                        kind: ScreenInputEventKind::Click(point)
                    });
                }

                /* up means we are no longer the capturing mount */
                self.capturing_mount_id = None;
            }
            _ => () // otherwise do nothing
        }

        let mut should_rerender = false;

        /* 
         * for every input event in the queue we want to find the mount with the given id and have it run its input handler 
         * if it returns true then the screen needs to be rendered again
         */
        for input_event in self.input_event_queue.drain(..) {
            if let Some(descendant) = root.find_descendant_mut(MountFinder::new(input_event.mount_id)) {
                // call on separate line because we dont want short-circuiting to prevent descendant mouse input handler from running
                let descendant_requires_rerender = descendant.on_mouse_input(input_event);
                should_rerender = should_rerender || descendant_requires_rerender;
            }
        }

        should_rerender
    }
}

/* 
 * basic struct that keeps a count of the number of animations running inside the screen
 * the onus is on the drawings the increment / decrement this counter otherwise there will be incorrect rerender timings :)
 * its not the most "sophisticated" but it works
 */
#[derive(Debug, Default)]
pub struct ScreenAnimationService {
    animation_count: usize
}

impl ScreenAnimationService {
    pub fn sub(&mut self) {
        self.animation_count = self.animation_count.saturating_sub(1);
    }
    pub fn add(&mut self) {
        self.animation_count += 1;
    }
    pub fn contains_any(&self) -> bool {
        self.animation_count > 0
    }
}