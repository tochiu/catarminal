use super::{
    draw::*,
    space::*,
    mount::*
};

use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
use tui::{
    layout::Rect,
    buffer::Buffer,
    widgets::{Widget, StatefulWidget}, style::Color
};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct Screen<T: MountableLayout + StatefulDrawable> {
    pub root: T,
    pub input: ScreenInput
}

impl<T: MountableLayout + StatefulDrawable> Screen<T> {
    pub fn new(mut root: T) -> Self {
        root.mount(Mount::default());

        Screen {
            root,
            input: ScreenInput::default()
        }
    }

    pub fn as_widget(&mut self) -> ScreenWidget<T> {
        ScreenWidget {
            screen: self
        }
    }

    fn relayout_root(&mut self, absolute_screen_space: AbsoluteSpace) {
        self.root.relayout(ScreenRelayout {
            id: self.root.mount_ref().id,
            absolute_layout_space: self.root.to_absolute_layout_space(absolute_screen_space),
            parent_absolute_draw_space: absolute_screen_space,
            parent_absolute_layout_space: absolute_screen_space,
            input: &mut self.input
        });
    }

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

pub struct ScreenWidget<'a, T: MountableLayout + StatefulDrawable> {
    screen: &'a mut Screen<T>
}

impl<'a, T: MountableLayout + StatefulDrawable> StatefulWidget for ScreenWidget<'a, T> {
    type State = T::State;
    fn render(self, rect: Rect, buf: &mut Buffer, state: &mut T::State) {
        let absolute_screen_space = AbsoluteSpace::from_rect(rect);

        self.screen.input.invalidate_all_inputs();
        self.screen.relayout_root(absolute_screen_space);
        self.screen.input.clear_invalid_inputs();
        self.screen.draw_root(absolute_screen_space, buf, &state);
    }
}

#[derive(Debug)]
pub struct ScreenRelayout<'a> {
    pub id: MountId,
    pub absolute_layout_space: AbsoluteSpace,
    pub parent_absolute_draw_space: AbsoluteSpace,
    pub parent_absolute_layout_space: AbsoluteSpace,
    pub input: &'a mut ScreenInput
}

impl<'a> ScreenRelayout<'a> {
    /*
     * Restricts space of the given Layoutable to not exceed the bounds of the parent's absolute draw space
     * Returns Option because the restrictions could lead to no drawable space (layout out of bounds of parent drawing space)
     */
    pub fn restrict_absolute_layout_space(&self, subarea_absolute_layout_space: AbsoluteSpace) -> Option<AbsoluteSpace> {
        self.parent_absolute_draw_space.try_intersection(subarea_absolute_layout_space)
    }
}

pub struct ScreenArea<'a> {
    pub absolute_draw_space: AbsoluteSpace,
    pub absolute_layout_space: AbsoluteSpace,
    pub buf: &'a mut Buffer
}

impl<'a> ScreenArea<'a> {
    pub fn transform(self, absolute_layout_space: AbsoluteSpace) -> ScreenArea<'a> {
        ScreenArea { absolute_layout_space, ..self }
    }

    pub fn draw_child<T: Drawable>(&mut self, child: &T) {
        let subarea_absolute_layout_space = child.to_absolute_layout_space(self.absolute_layout_space);
        if let Some(subarea_absolute_draw_space) = self.absolute_draw_space.try_intersection(subarea_absolute_layout_space) {
            child.draw(ScreenArea {
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
                ScreenArea {
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

    pub fn draw_string_line(&mut self, line: &str, position: Point2D, color: Color) {
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
                self.buf.content[buf_index + i].set_char(c).set_fg(color);
            }
        }
    }

    pub fn draw_unicode_line(&mut self, line: &str, pos: Point2D, color: Color) {
        let bufy = self.absolute_layout_space.top() + pos.y;
        let bufx = self.absolute_layout_space.left() + pos.x;
        if 
            bufy >= self.absolute_draw_space.top() && 
            bufy < self.absolute_draw_space.bottom() &&
            bufx < self.absolute_draw_space.right()
        {
            let bufx = self.absolute_layout_space.left() + pos.x;
            let index = self.buf.index_of(bufx as u16, bufy as u16);
            for (offset, grapheme) in 
                line.graphemes(false) 
                    .enumerate()
                    .skip((self.absolute_draw_space.left() - bufx).max(0) as usize)
                    .take((self.absolute_draw_space.right() - bufx).max(0) as usize)
            {
                self.buf.content[index + offset].set_symbol(grapheme).set_fg(color);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Input {
    id: MountId,
    space: AbsoluteSpace
}

#[derive(Debug, Default)]
pub struct ScreenInput {
    inputs: Vec<Input>,
    current_input_id: Option<MountId>,
    valid_input_count: usize,
    did_mouse_move: bool,
    input_event_queue: Vec<ScreenInputEvent>
}

#[derive(Debug)]
pub struct ScreenInputEvent {
    mount_id: MountId,
    pub kind: ScreenInputEventKind
}

#[derive(Debug)]
pub enum ScreenInputEventKind {
    Click(Point2D),
    Drag(Point2D),
    Move(Point2D),
    Down(Point2D),
    Up(Point2D)
}

impl ScreenInput {
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
                    self.input_event_queue.push(ScreenInputEvent { 
                        mount_id: old_id,
                        kind: ScreenInputEventKind::Up(point)
                    });
                }

                if let Some(id) = maybe_id {
                    self.input_event_queue.push(ScreenInputEvent { 
                        mount_id: id, 
                        kind: ScreenInputEventKind::Down(point) 
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
                self.input_event_queue.push(ScreenInputEvent { 
                    mount_id: self.current_input_id.unwrap(), 
                    kind: ScreenInputEventKind::Drag(point)
                });
            },
            MouseEventKind::Moved => {
                if let Some(current_input_id) = self.current_input_id {
                    self.did_mouse_move = true;
                    self.input_event_queue.push(ScreenInputEvent {
                        mount_id: current_input_id, 
                        kind: ScreenInputEventKind::Drag(point)
                    });
                } else if let Some(id) = maybe_id {
                    self.input_event_queue.push(ScreenInputEvent { 
                        mount_id: id, 
                        kind: ScreenInputEventKind::Move(point)
                    });
                }
            }
            MouseEventKind::Up(button) => {
                if button == MouseButton::Middle || self.current_input_id.is_none() {
                    return false
                }

                self.input_event_queue.push(ScreenInputEvent { 
                    mount_id: self.current_input_id.unwrap(), 
                    kind: ScreenInputEventKind::Up(point)
                });
                if self.current_input_id == self.query(point) {
                    self.input_event_queue.push(ScreenInputEvent { 
                        mount_id: self.current_input_id.unwrap(), 
                        kind: ScreenInputEventKind::Click(point)
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