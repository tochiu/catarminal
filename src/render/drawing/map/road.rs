use crate::render::{draw::{Layoutable, Drawable}, mount::MountableLayout, screen::ScreenArea};

use super::super::super::{
    draw::DrawLayout,
    mount::Mount,
    space::*
};

use rand::Rng;
use tui::style::{Style, Color};

#[derive(Debug)]
pub struct Road {
    start: Point2D,
    change: Point2D,
    steps: u8,
    style: Style,
    mount: Mount,
    pub layout: DrawLayout,
}

impl Road {
    pub fn new(map_from: Point2D, map_to: Point2D, style: Style, mut layout: DrawLayout) -> Self {
        let top_left_point = Point2D::new(map_from.x.min(map_to.x) - 1, map_from.y.min(map_to.y));
        let from = map_from - top_left_point;
        let to = map_to - top_left_point;
        let dx = to.x - from.x;
        let dy = to.y - from.y;

        let mut rng = rand::thread_rng();

        layout
            .set_size(UDim2::from_offset(dx.abs() + 3, dy.abs() + 3))
            .set_position(UDim2::from_point2d(top_left_point));
        
        Road {
            start: from,
            change: Point2D::new(dx.signum(), dy.signum()),
            steps: dx.abs().max(dy.abs()) as u8,
            mount: Mount::default(),
            // test randomize style
            style: style.bg(Color::Rgb(rng.gen_range(0..=225), rng.gen_range(0..=225), rng.gen_range(0..=225))),
            layout,
        }
    }
}

impl Drawable for Road {
    fn draw(&self, mut area: ScreenArea) {
        for offset in -1..=1 {
            let mut point = self.start + Point2D::new(offset, 0);
            for _ in 0..=self.steps {
                if let Some(cell) = area.mut_cell_at(point) {
                    cell.set_symbol(" ").set_style(self.style);
                }
                point = point + self.change;
            }
        }
    }
}

impl Layoutable for Road {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl MountableLayout for Road {
    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }
    fn child_ref(&self, _: usize) -> Option<&dyn MountableLayout> { None }
    fn child_mut(&mut self, _: usize) -> Option<&mut dyn MountableLayout> { None }
}