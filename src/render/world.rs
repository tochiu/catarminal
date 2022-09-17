use super::{
    draw::*,
    space::*,
    mount::*,
    map::Map, 
    drag::Dragger
};

use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
use tui::{
    layout::Rect,
    buffer::Buffer,
    widgets::Widget
};

#[derive(Debug)]
pub struct World {
    pub root: Dragger<Map>,
    pub input: WorldInput
}

impl World {
    pub fn new(map: Map) -> Self {
        let mut root = Dragger::new(map);
        root.mount(Mount {
            id: 0,
            children: 0
        });

        World {
            root,
            input: WorldInput::default()
        }
    }

    pub fn as_widget(&mut self) -> WorldWidget {
        WorldWidget {
            world: self
        }
    }
}

pub struct WorldWidget<'a> {
    world: &'a mut World
}

impl<'a> Widget for WorldWidget<'a> {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let rect_space = AbsoluteSpace::from_rect(rect);

        self.world.input.invalidate_all_inputs();
        self.world.root.layout(WorldLayout {
            id: self.world.root.mount_ref().id,
            calculated_space: self.world.root.layout_ref().space.to_absolute_space(rect_space),
            parent_draw_space: rect_space,
            parent_full_space: rect_space,
            input: &mut self.world.input
        });
        self.world.input.clear_invalid_inputs();

        let mut canvas = WorldArea {
            draw_space: rect_space,
            full_space: rect_space,
            buf
        };

        canvas.draw_child(&self.world.root);
    }
}

#[derive(Debug)]
pub struct WorldLayout<'a> {
    pub id: MountId,
    pub calculated_space: AbsoluteSpace,
    pub parent_draw_space: AbsoluteSpace,
    pub parent_full_space: AbsoluteSpace,
    pub input: &'a mut WorldInput
}

#[derive(Debug)]
pub struct WorldArea<'a> {
    pub draw_space: AbsoluteSpace,
    pub full_space: AbsoluteSpace,
    pub buf: &'a mut Buffer
}

impl<'a> WorldArea<'a> {
    pub fn transform(self, full_space: AbsoluteSpace) -> WorldArea<'a> {
        WorldArea { full_space, ..self }
    }

    pub fn draw_child<T: Drawing>(&mut self, child: &T) {
        let full_space = child.layout_ref().space.to_absolute_space(self.full_space);
        if !full_space.intersects(self.draw_space) {
            return
        }
        
        child.draw(WorldArea {
            draw_space: full_space.intersection(self.draw_space),
            full_space,
            buf: self.buf
        });
    }

    pub fn draw_children<T: Drawing>(&mut self, children: &[T]) {
        for child in children {
            self.draw_child(child);
        }
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
        self.inputs.push(Input { id, space });
        self.valid_input_count += 1;
    }

    fn query(&self, point: Point2D) -> Option<MountId> {
        for input in self.inputs.iter().rev() {
            let id = input.id;
            let space = input.space;
            if point.x >= space.left() && point.x < space.right() && point.y >= space.top() && point.y < space.bottom() {
                return Some(id);
            }
        }

        None
    }

    pub fn handle_mouse_input(&mut self, event: MouseEvent, root_mount: &mut dyn Mountable) -> bool {
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

        let mut should_redraw = false;
        for input_event in self.input_event_queue.drain(..) {
            if let Some(descendant) = root_mount.find_descendant_mut(MountFinder::new(input_event.mount_id)) {
                should_redraw = should_redraw || descendant.on_mouse_input(input_event);
            }
        }

        should_redraw
    }
}