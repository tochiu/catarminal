use super::{
    draw::*,
    space::*,
    mount::*
};

use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
use tui::{
    layout::Rect,
    buffer::Buffer,
    widgets::{Widget, StatefulWidget}
};

#[derive(Debug)]
pub struct World<T: MountableLayout + StatefulDrawable> {
    pub root: T,
    pub input: WorldInput
}

impl<T: MountableLayout + StatefulDrawable> World<T> {
    pub fn new(mut root: T) -> Self {
        root.mount(Mount::default());

        World {
            root,
            input: WorldInput::default()
        }
    }

    pub fn as_widget(&mut self) -> WorldWidget<T> {
        WorldWidget {
            world: self
        }
    }

    fn relayout_root(&mut self, absolute_world_space: AbsoluteSpace) {
        self.root.relayout(WorldRelayout {
            id: self.root.mount_ref().id,
            absolute_layout_space: self.root.to_absolute_layout_space(absolute_world_space),
            parent_absolute_draw_space: absolute_world_space,
            parent_absolute_layout_space: absolute_world_space,
            input: &mut self.input
        });
    }

    fn draw_root(&self, absolute_world_space: AbsoluteSpace, buf: &mut Buffer, state: &<T as StatefulDrawable>::State) {
        WorldArea::draw_stateful_child(
            &mut WorldArea {
                absolute_draw_space: absolute_world_space,
                absolute_layout_space: absolute_world_space,
                buf
            }, 
            &self.root, 
            state
        );
    }
}

pub struct WorldWidget<'a, T: MountableLayout + StatefulDrawable> {
    world: &'a mut World<T>
}

impl<'a, T: MountableLayout + StatefulDrawable> StatefulWidget for WorldWidget<'a, T> {
    type State = T::State;
    fn render(self, rect: Rect, buf: &mut Buffer, state: &mut T::State) {
        let absolute_world_space = AbsoluteSpace::from_rect(rect);

        self.world.input.invalidate_all_inputs();
        self.world.relayout_root(absolute_world_space);
        self.world.input.clear_invalid_inputs();
        self.world.draw_root(absolute_world_space, buf, &state);
    }
}

#[derive(Debug)]
pub struct WorldRelayout<'a> {
    pub id: MountId,
    pub absolute_layout_space: AbsoluteSpace,
    pub parent_absolute_draw_space: AbsoluteSpace,
    pub parent_absolute_layout_space: AbsoluteSpace,
    pub input: &'a mut WorldInput
}

impl<'a> WorldRelayout<'a> {
    /*
     * Restricts space of the given Layoutable to not exceed the bounds of the parent's absolute draw space
     * Returns Option because the restrictions could lead to no drawable space (layout out of bounds of parent drawing space)
     */
    pub fn restrict_absolute_layout_space(&self, subarea_absolute_layout_space: AbsoluteSpace) -> Option<AbsoluteSpace> {
        self.parent_absolute_draw_space.try_intersection(subarea_absolute_layout_space)
    }
}

pub struct WorldArea<'a> {
    pub absolute_draw_space: AbsoluteSpace,
    pub absolute_layout_space: AbsoluteSpace,
    pub buf: &'a mut Buffer
}

impl<'a> WorldArea<'a> {
    pub fn transform(self, absolute_layout_space: AbsoluteSpace) -> WorldArea<'a> {
        WorldArea { absolute_layout_space, ..self }
    }

    pub fn draw_child<T: Drawable>(&mut self, child: &T) {
        let subarea_absolute_layout_space = child.to_absolute_layout_space(self.absolute_layout_space);
        if let Some(subarea_absolute_draw_space) = self.absolute_draw_space.try_intersection(subarea_absolute_layout_space) {
            child.draw(WorldArea {
                absolute_draw_space: subarea_absolute_draw_space,
                absolute_layout_space: subarea_absolute_layout_space,
                buf: self.buf
            });
        }
    }

    pub fn draw_children<T: Drawable>(&mut self, children: &[T]) {
        for child in children {
            self.draw_child(child);
        }
    }

    pub fn draw_stateful_child<T: StatefulDrawable>(&mut self, child: &T, state: &T::State) {
        let subarea_absolute_layout_space = child.to_absolute_layout_space(self.absolute_layout_space);
        if let Some(subarea_absolute_draw_space) = self.absolute_draw_space.try_intersection(subarea_absolute_layout_space) {
            child.stateful_draw(
                WorldArea {
                    absolute_draw_space: subarea_absolute_draw_space,
                    absolute_layout_space: subarea_absolute_layout_space,
                    buf: self.buf
                },
                state
            );
        }
    }

    pub fn draw_stateful_children<T: StatefulDrawable>(&mut self, children: &[T], states: &[T::State]) {
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
}

#[derive(Debug, PartialEq, Eq)]
pub struct Input {
    id: MountId,
    space: AbsoluteSpace
}

#[derive(Debug, Default)]
pub struct WorldInput {
    inputs: Vec<Input>,
    current_input_id: Option<MountId>,
    valid_input_count: usize,
    did_mouse_move: bool,
    input_event_queue: Vec<WorldInputEvent>
}

#[derive(Debug)]
pub struct WorldInputEvent {
    mount_id: MountId,
    pub kind: WorldInputEventKind
}

#[derive(Debug)]
pub enum WorldInputEventKind {
    Click(Point2D),
    Drag(Point2D),
    Move(Point2D),
    Down(Point2D),
    Up(Point2D)
}

impl WorldInput {
    fn invalidate_all_inputs(&mut self) {
        self.valid_input_count = 0;
    }

    fn clear_invalid_inputs(&mut self) {
        self.inputs.truncate(self.valid_input_count);
    }

    pub fn update(&mut self, id: MountId, space: AbsoluteSpace) {
        if self.valid_input_count == self.inputs.len() {
            self.inputs.push(Input { id, space });
        } else {
            self.inputs[self.valid_input_count] = Input { id, space };
        }
        
        self.valid_input_count += 1;
    }

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

    pub fn handle_mouse_input(&mut self, event: MouseEvent, root: &mut dyn MountableLayout) -> bool {
        let point = Point2D::new(
            i16::try_from(event.column).unwrap(), 
            i16::try_from(event.row).unwrap()
        );

        let maybe_id = self.query(point);

        match event.kind {
            MouseEventKind::Down(button) => {
                if button == MouseButton::Middle || maybe_id == self.current_input_id {
                    return false
                }

                if let Some(old_id) = self.current_input_id {
                    self.input_event_queue.push(WorldInputEvent { 
                        mount_id: old_id,
                        kind: WorldInputEventKind::Up(point)
                    });
                }

                if let Some(id) = maybe_id {
                    self.input_event_queue.push(WorldInputEvent { 
                        mount_id: id, 
                        kind: WorldInputEventKind::Down(point) 
                    });
                }

                self.current_input_id = maybe_id;
                self.did_mouse_move = false;
            },
            MouseEventKind::Drag(button) => {
                if button == MouseButton::Middle || self.current_input_id.is_none() {
                    return false
                }

                self.did_mouse_move = true;
                self.input_event_queue.push(WorldInputEvent { 
                    mount_id: self.current_input_id.unwrap(), 
                    kind: WorldInputEventKind::Drag(point)
                });
            },
            MouseEventKind::Moved => {
                if let Some(current_input_id) = self.current_input_id {
                    self.did_mouse_move = true;
                    self.input_event_queue.push(WorldInputEvent {
                        mount_id: current_input_id, 
                        kind: WorldInputEventKind::Drag(point)
                    });
                } else if let Some(id) = maybe_id {
                    self.input_event_queue.push(WorldInputEvent { 
                        mount_id: id, 
                        kind: WorldInputEventKind::Move(point)
                    });
                }
            }
            MouseEventKind::Up(button) => {
                if button == MouseButton::Middle || self.current_input_id.is_none() {
                    return false
                }

                self.input_event_queue.push(WorldInputEvent { 
                    mount_id: self.current_input_id.unwrap(), 
                    kind: WorldInputEventKind::Up(point)
                });
                if self.current_input_id == self.query(point) {
                    self.input_event_queue.push(WorldInputEvent { 
                        mount_id: self.current_input_id.unwrap(), 
                        kind: WorldInputEventKind::Click(point)
                    });
                }

                self.current_input_id = None;
            }
            _ => ()
        }

        let mut should_rerender = false;
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