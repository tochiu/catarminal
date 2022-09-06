use super::{
    draw::*,
    space::AbsoluteSpace
};

use tui::{
    layout::Rect,
    buffer::Buffer,
    widgets::Widget
};

use std::marker::PhantomData;
use std::cell::{Ref, RefMut, RefCell};
use std::any::Any;

pub type DrawId = usize;
pub type DrawingRef<'a, T> = Ref<'a, Drawing<T>>;
pub type DrawingRefMut<'a, T> = RefMut<'a, Drawing<T>>;

#[derive(Debug)]
pub struct DrawRef<T: Drawable> {
    pub id: DrawId,
    pub tp: PhantomData<T>
}

impl<T: Drawable> Clone for DrawRef<T> {
    fn clone(&self) -> DrawRef<T> {
        DrawRef {
            id: self.id,
            tp: PhantomData
        }
    }
}

impl<T: Drawable> Copy for DrawRef<T> {}

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
    roots: Vec<DrawId>,
    nodes: Vec<RefCell<Option<Drawing<dyn Drawable>>>>,
    edges: Vec<Vec<DrawId>>
}

impl WorldCanvas {
    pub fn as_widget(&mut self) -> WorldWidget {
        WorldWidget {
            canvas: self
        }
    }

    pub fn mount_root<T: Drawable>(&mut self, drawable: T) -> DrawingRefMut<T> {
        self.mount(drawable, None)
    }

    // pub fn get<T: Drawable>(&self, dref: &DrawRef<T>) -> &Drawing<T> {
    //     self.nodes[dref.id].borrow().as_ref().unwrap().as_any().downcast_ref::<Drawing<T>>().unwrap()
    // }
    
    pub fn get<T: Drawable>(&self, dref: DrawRef<T>) -> DrawingRef<T> {
        Ref::map(self.nodes[dref.id].borrow(), |x| x.as_ref().unwrap().as_any().downcast_ref::<Drawing<T>>().unwrap())
    }

    pub fn get_mut<T: Drawable>(&self, dref: DrawRef<T>) -> DrawingRefMut<T> {
        RefMut::map(self.nodes[dref.id].borrow_mut(), |x| x.as_mut().unwrap().as_any_mut().downcast_mut::<Drawing<T>>().unwrap())
    }

    pub fn get_dyn(&self, id: DrawId) -> DrawingRef<dyn Drawable> {
        Ref::map(self.nodes[id].borrow(), |x| x.as_ref().unwrap().as_any().downcast_ref::<Drawing<dyn Drawable>>().unwrap())
    }

    pub fn get_dyn_mut(&self, id: DrawId) -> DrawingRefMut<dyn Drawable> {
        RefMut::map(self.nodes[id].borrow_mut(), |x| x.as_mut().unwrap().as_any_mut().downcast_mut::<Drawing<dyn Drawable>>().unwrap())
    }

    pub fn iter_children(&self, id: DrawId) -> impl Iterator<Item=DrawId> + '_ {
        self.edges[id].iter().cloned()
    }

    fn mount<T: Drawable>(&mut self, drawable: T, parent_id: Option<DrawId>) -> DrawingRefMut<T> {
        let id = self.nodes.len();
        let mut drawing = Draw::new(drawable, id);
        let drawing_dref = drawing.as_dref();

        self.nodes.push(RefCell::new(None));
        self.edges.push(Vec::new());

        if let Some(parent_id) = parent_id {
            self.edges[parent_id].push(id);
        } else {
            self.roots.push(id);
        }
        
        T::on_mount(&mut drawing, &mut MountController { drawing_id: id, canvas: self });
        
        self.nodes[id].replace(Some(drawing));

        println!("{:#?} {:#?}", self.nodes[id], drawing_dref);

        self.get_mut(drawing_dref)
    }
}

pub struct MountController<'a> {
    drawing_id: DrawId,
    pub canvas: &'a mut WorldCanvas,
}

impl<'a> MountController<'a> {
    pub fn mount_child<T: Drawable>(&mut self, drawable: T) -> DrawingRefMut<T> {
        self.canvas.mount(drawable, Some(self.drawing_id))
    }
}

pub struct WorldWidget<'a> {
    canvas: &'a mut WorldCanvas
}

impl<'a> Widget for WorldWidget<'a> {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let rect_space = AbsoluteSpace::from_rect(rect);
        let mut canvas = DrawingCanvas {
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