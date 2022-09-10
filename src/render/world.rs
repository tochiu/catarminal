use super::{
    draw::*,
    space::*
};

use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
use tui::{
    layout::Rect,
    buffer::Buffer,
    widgets::Widget
};

use std::marker::PhantomData;

pub type WorldId = usize;

#[derive(Debug)]
pub struct WorldRef<T: Drawable> {
    pub id: WorldId,
    pub tp: PhantomData<T>
}

impl<T: Drawable> Clone for WorldRef<T> {
    fn clone(&self) -> WorldRef<T> {
        WorldRef {
            id: self.id,
            tp: PhantomData
        }
    }
}

impl<T: Drawable> Copy for WorldRef<T> {}

#[derive(Debug)]
pub struct World {
    pub canvas: WorldCanvas,
    pub input: WorldInput
}

impl World {
    pub fn new() -> Self {
        World {
            canvas: WorldCanvas::default(),
            input: WorldInput::default()
        }
    }

    pub fn as_widget(&mut self) -> WorldWidget {
        WorldWidget {
            world: self
        }
    }
}

#[derive(Debug, Default)]
pub struct WorldCanvas {
    roots: Vec<WorldId>,
    nodes: Vec<Option<Box<dyn Drawable>>>,
    edges: Vec<Vec<WorldId>>,
    layout: Vec<DrawLayout>
}

impl WorldCanvas {

    pub fn mount_root<T: Drawable>(&mut self, drawable: T) -> WorldRef<T> {
        self.mount(drawable, DrawLayout::FULL, None)
    }

    pub fn mount_child<T: Drawable>(&mut self, drawable: T, parent_id: WorldId) -> WorldRef<T> {
        self.mount(drawable, DrawLayout::FULL, Some(parent_id))
    }
    
    pub fn get<T: Drawable>(&self, wref: WorldRef<T>) -> &T {
        self.get_dyn(wref.id).as_any().downcast_ref::<T>().unwrap()
    }

    pub fn get_mut<T: Drawable>(&mut self, wref: WorldRef<T>) -> &mut T {
        self.get_dyn_mut(wref.id).as_any_mut().downcast_mut::<T>().unwrap()
    }

    pub fn get_dyn(&self, id: WorldId) -> &Box<dyn Drawable> {
        self.nodes[id].as_ref().unwrap()
    }

    pub fn get_dyn_mut(&mut self, id: WorldId) -> &mut Box<dyn Drawable> {
        self.nodes[id].as_mut().unwrap()
    }

    pub fn get_layout(&self, id: WorldId) -> &DrawLayout {
        &self.layout[id]
    }

    pub fn get_layout_mut(&mut self, id: WorldId) -> &mut DrawLayout {
        &mut self.layout[id]
    }

    pub fn get_full_mut(&mut self, id: WorldId) -> WorldDrawingMut {
        WorldDrawingMut { 
            drawing: self.nodes[id].as_deref_mut().unwrap(), 
            layout: &mut self.layout[id]
        }
    }

    pub fn iter_children(&self, id: WorldId) -> impl Iterator<Item=WorldId> + '_ {
        self.edges[id].iter().cloned()
    }

    pub fn relate(&mut self, parent_id: WorldId, child_id: WorldId) {
        self.edges[parent_id].push(child_id);
    }

    fn mount<T: Drawable>(&mut self, mut drawing: T, mut layout: DrawLayout, parent_id: Option<WorldId>) -> WorldRef<T> {
        let id = self.nodes.len();

        self.nodes.push(None);
        self.edges.push(Vec::new());
        self.layout.push(DrawLayout::default());

        drawing.on_mounting(WorldMount { 
            id, 
            layout: &mut layout,
            canvas: self 
        });

        self.nodes[id].replace(Box::new(drawing));
        self.layout[id] = layout;

        if let Some(parent_id) = parent_id {
            self.edges[parent_id].push(id);
        } else {
            self.roots.push(id);
        }

        WorldRef {
            id,
            tp: PhantomData
        }
    }
}

pub struct WorldDrawingMut<'a> {
    drawing: &'a mut dyn Drawable, 
    layout: &'a mut DrawLayout
}

pub struct WorldMount<'a> {
    id: WorldId,
    pub layout: &'a mut DrawLayout,
    pub canvas: &'a mut WorldCanvas,
}

impl<'a> WorldMount<'a> {
    pub fn child<T: Drawable>(&mut self, drawable: T, layout: DrawLayout) -> WorldRef<T> {
        self.canvas.mount(drawable, layout, Some(self.id))
    }
}

pub struct WorldWidget<'a> {
    world: &'a mut World
}

impl<'a> Widget for WorldWidget<'a> {
    fn render(mut self, rect: Rect, buf: &mut Buffer) {
        let rect_space = AbsoluteSpace::from_rect(rect);

        // TODO: could be optimized ( collecting to avoid ownership )
        let roots: Vec<usize> = self.world.canvas.roots.iter().cloned().collect();

        for root_id in roots {
            self.layout(root_id, rect_space);
        }

        self.world.input.invalidate_all_inputs();

        let mut canvas = WorldArea {
            id: None,
            draw_space: rect_space,
            full_space: rect_space,
            buf,
            canvas: &self.world.canvas,
            input: &mut self.world.input
        };

        for root_id in self.world.canvas.roots.iter() {
            canvas.draw_child(*root_id);
        }
        self.world.input.clear_invalid_inputs();
    }
}

impl<'a> WorldWidget<'a> {
    fn layout(&mut self, id: WorldId, parent_space: AbsoluteSpace) {
        let full_drawing = self.world.canvas.get_full_mut(id);
        let space = full_drawing.drawing.layout(full_drawing.layout.space.to_absolute_space(parent_space), full_drawing.layout);

        // TODO: could be optimized ( collecting to avoid ownership )
        let children: Vec<WorldId> = self.world.canvas.iter_children(id).collect();
        for child_id in children {
            self.layout(child_id, space);
        }
    }
}

#[derive(Debug)]
pub struct WorldArea<'a> {
    id: Option<WorldId>,
    pub draw_space: AbsoluteSpace,
    pub full_space: AbsoluteSpace,
    pub buf: &'a mut Buffer,
    canvas: &'a WorldCanvas,
    input: &'a mut WorldInput
}

impl<'a> WorldArea<'a> {
    pub fn transform(self, draw_space: AbsoluteSpace, full_space: AbsoluteSpace) -> WorldArea<'a> {
        WorldArea {
            draw_space,
            full_space,
            ..self
        }
    }

    pub fn draw_child(&mut self, child_id: WorldId) {
        let child_drawing = self.canvas.get_dyn(child_id);

        let full_space = self.canvas.get_layout(child_id).space.to_absolute_space(self.full_space);
        if !full_space.intersects(self.draw_space) {
            return
        }
        
        child_drawing.draw(WorldArea {
            id: Some(child_id),
            draw_space: full_space.intersection(self.draw_space),
            full_space,
            buf: self.buf,
            canvas: self.canvas,
            input: self.input
        });
    }

    pub fn draw_children(&mut self) {
        for child_id in self.canvas.iter_children(self.id.unwrap()) {
            self.draw_child(child_id);
        }
    }

    pub fn update_input(&mut self, space: AbsoluteSpace) {
        if self.draw_space.intersects(space) {
            self.input.update(self.id.unwrap(), self.draw_space.intersection(space));
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Input {
    id: WorldId,
    space: AbsoluteSpace
}

#[derive(Debug, Default)]
pub struct WorldInput {
    inputs: Vec<Input>,
    current_input_id: Option<WorldId>,
    valid_input_count: usize,
    did_mouse_move: bool,
    input_event_queue: Vec<WorldInputEvent>
}

#[derive(Debug)]
pub struct WorldInputEvent {
    drawing_id: WorldId,
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

    fn update(&mut self, id: WorldId, space: AbsoluteSpace) {
        self.inputs.push(Input { id, space });
        self.valid_input_count += 1;
    }

    fn query(&self, point: Point2D) -> Option<WorldId> {
        for input in self.inputs.iter().rev() {
            let id = input.id;
            let space = input.space;
            if point.x >= space.left() && point.x < space.right() && point.y >= space.top() && point.y < space.bottom() {
                return Some(id);
            }
        }

        None
    }

    pub fn handle_mouse_input(&mut self, event: MouseEvent, canvas: &mut WorldCanvas) -> bool {
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
                        drawing_id: old_id,
                        kind: WorldInputEventKind::Up(point)
                    });
                }

                if let Some(id) = maybe_id {
                    self.input_event_queue.push(WorldInputEvent { 
                        drawing_id: id, 
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
                    drawing_id: self.current_input_id.unwrap(), 
                    kind: WorldInputEventKind::Drag(point)
                });
            },
            MouseEventKind::Moved => {
                if let Some(current_input_id) = self.current_input_id {
                    self.did_mouse_move = true;
                    self.input_event_queue.push(WorldInputEvent {
                        drawing_id: current_input_id, 
                        kind: WorldInputEventKind::Drag(point)
                    });
                } else if let Some(id) = maybe_id {
                    self.input_event_queue.push(WorldInputEvent { 
                        drawing_id: id, 
                        kind: WorldInputEventKind::Move(point)
                    });
                }
            }
            MouseEventKind::Up(button) => {
                if button == MouseButton::Middle || self.current_input_id.is_none() {
                    return false
                }

                self.input_event_queue.push(WorldInputEvent { 
                    drawing_id: self.current_input_id.unwrap(), 
                    kind: WorldInputEventKind::Up(point)
                });
                if self.current_input_id == self.query(point) {
                    self.input_event_queue.push(WorldInputEvent { 
                        drawing_id: self.current_input_id.unwrap(), 
                        kind: WorldInputEventKind::Click(point)
                    });
                }

                self.current_input_id = None;
            }
            _ => ()
        }

        let mut should_redraw = false;
        for input_event in self.input_event_queue.drain(..) {
            should_redraw = should_redraw || canvas.get_dyn_mut(input_event.drawing_id).on_mouse_input(input_event);
        }

        should_redraw
    }
}