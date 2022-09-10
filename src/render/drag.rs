use super::{
    draw::*, 
    world::*,
    space::*
};

#[derive(Debug)]
pub struct Dragger {
    pub canvas_size: UDim2,
    canvas_offset: Point2D,
    mouse_location: Point2D
}

impl Dragger {
    pub fn new() -> Self {
        Dragger {
            canvas_size: UDim2::from_scale(1.0, 1.0),
            canvas_offset: Point2D::default(),
            mouse_location: Point2D::default()
        }
    }

    fn get_constrained_canvas_offset(&self, canvas_size: Size2D, window_size: Size2D) -> Point2D {
        let max_offset_x = i16::try_from(canvas_size.x.saturating_sub(window_size.x)).unwrap_or(i16::MAX);
        let max_offset_y = i16::try_from(canvas_size.y.saturating_sub(window_size.y)).unwrap_or(i16::MAX);
        Point2D::new(
            self.canvas_offset.x.min(max_offset_x).max(0), 
            self.canvas_offset.y.min(max_offset_y).max(0)
        )
    }

    fn get_canvas_space(&self, window_space: AbsoluteSpace) -> AbsoluteSpace {
        let canvas_space = Space::sized(self.canvas_size).to_absolute_space(window_space);
        AbsoluteSpace::new(
            canvas_space.position.x.saturating_sub(self.canvas_offset.x), 
            canvas_space.position.y.saturating_sub(self.canvas_offset.y), 
            canvas_space.size.x,
            canvas_space.size.y
        )
    }
}

impl Drawable for Dragger {
    fn draw(&self, mut area: WorldArea) {
        let draw_space = area.draw_space;
        let canvas_space = self.get_canvas_space(area.full_space);
        area.update_input(draw_space);
        area.transform(draw_space, canvas_space).draw_children();
    }

    fn layout(&mut self, window_space: AbsoluteSpace, _: &mut DrawLayout) -> AbsoluteSpace {
        // get the absolute canvas size using the absolute window size and constrain the dragging range (max canvas offset)
        self.canvas_offset = self.get_constrained_canvas_offset(
            Space::sized(self.canvas_size).to_absolute_space(window_space).size, 
            window_space.size
        );

        // return the canvas space so children can layout relative to the canvas space instead of the window space
        self.get_canvas_space(window_space)
    }

    fn on_mouse_input(&mut self, event: WorldInputEvent) -> bool {
        match event.kind {
            WorldInputEventKind::Down(point) => {
                self.mouse_location = point;
                false
            },
            WorldInputEventKind::Drag(point) => {
                let canvas_offset = Point2D::new(
                    self.canvas_offset.x.saturating_sub(point.x.saturating_sub(self.mouse_location.x)),
                    self.canvas_offset.y.saturating_sub(point.y.saturating_sub(self.mouse_location.y))
                );
                self.mouse_location = point;
                
                if canvas_offset != self.canvas_offset {
                    self.canvas_offset = canvas_offset;
                    true
                } else {
                    false
                }
            },
            _ => false
        }
    }
}