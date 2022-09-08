use super::{
    draw::*,
    space::*
};

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
}

impl World {
    pub fn new() -> Self {
        World {
            canvas: WorldCanvas::default()
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
    pub fn as_widget(&mut self) -> WorldWidget {
        WorldWidget {
            canvas: self
        }
    }

    pub fn mount_root<T: Drawable>(&mut self, drawable: T) -> WorldRef<T> {
        self.mount(drawable, DrawLayout::FULL, None)
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

    pub fn iter_children(&self, id: WorldId) -> impl Iterator<Item=WorldId> + '_ {
        self.edges[id].iter().cloned()
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
    canvas: &'a mut WorldCanvas
}

impl<'a> Widget for WorldWidget<'a> {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let rect_space = AbsoluteSpace::from_rect(rect);
        let mut canvas = WorldArea {
            id: None,
            draw_space: rect_space,
            full_space: rect_space,
            buf,
            world: &self.canvas
        };

        for root_id in self.canvas.roots.iter() {
            canvas.draw_child(*root_id);
        }
    }    
}

#[derive(Debug)]
pub struct WorldArea<'a> {
    pub id: Option<WorldId>,
    pub draw_space: AbsoluteSpace,
    pub full_space: AbsoluteSpace,
    pub buf: &'a mut Buffer,
    pub world: &'a WorldCanvas
}

impl<'a> WorldArea<'a> {
    pub fn draw_child(&mut self, child_id: WorldId) {
        let child_drawing = self.world.get_dyn(child_id);

        let full_space = self.world.get_layout(child_id).space.to_absolute_space(self.full_space);
        if !full_space.intersects(self.draw_space) {
            return
        }
        
        child_drawing.draw(WorldArea {
            id: Some(child_id),
            draw_space: full_space.intersection(self.draw_space),
            full_space,
            buf: self.buf,
            world: self.world
        });
    }

    pub fn draw_children(&mut self) {
        for child_id in self.world.iter_children(self.id.unwrap()) {
            self.draw_child(child_id);
        }
    }
}