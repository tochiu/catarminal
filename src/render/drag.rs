use tui::style::Style;

use super::{
    space::*, 
    mount::{MountableLayout, Mount, AsTrait}, 
    draw::{DrawLayout, StatefulDrawable, Layoutable}, 
    screen::{ScreenArea, ScreenRelayout, ScreenInputEvent, ScreenInputEventKind}, iter::CustomIterator
};

#[derive(Debug)]
pub struct Dragger<T: MountableLayout + StatefulDrawable> {
    pub drawing: T,
    pub layout: DrawLayout,
    mount: Mount,
    canvas_offset: Point2D,
    mouse_location: Point2D,
    style: Style
}

impl<T: MountableLayout + StatefulDrawable> Dragger<T> {
    pub fn new(drawing: T, style: Style) -> Self {
        Dragger {
            drawing,
            style,
            layout: DrawLayout::FULL,
            mount: Mount::default(),
            canvas_offset: Point2D::default(),
            mouse_location: Point2D::default()
        }
    }

    fn get_absolute_canvas_size(&self, absolute_window_space: AbsoluteSpace) -> Size2D {
        self.drawing.to_absolute_layout_space(absolute_window_space).size
    }

    fn get_constrained_canvas_offset(&self, absolute_window_space: AbsoluteSpace) -> Point2D {
        let canvas_size = self.get_absolute_canvas_size(absolute_window_space);
        let max_offset_x = i16::try_from(canvas_size.x.saturating_sub(absolute_window_space.size.x)).unwrap_or(i16::MAX);
        let max_offset_y = i16::try_from(canvas_size.y.saturating_sub(absolute_window_space.size.y)).unwrap_or(i16::MAX);
        Point2D::new(
            self.canvas_offset.x.min(max_offset_x).max(0), 
            self.canvas_offset.y.min(max_offset_y).max(0)
        )
    }

    fn get_absolute_canvas_space(&self, window_space: AbsoluteSpace) -> AbsoluteSpace {
        AbsoluteSpace::new(
            window_space.position.x.saturating_sub(self.canvas_offset.x), 
            window_space.position.y.saturating_sub(self.canvas_offset.y), 
            window_space.size.x,
            window_space.size.y
        )
    }
}

impl<T: MountableLayout + StatefulDrawable> Layoutable for Dragger<T> {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl<T: MountableLayout + StatefulDrawable> StatefulDrawable for Dragger<T> {
    type State = T::State;
    fn stateful_draw(&self, mut area: ScreenArea, state: &Self::State) {
        let mut itr = area.iter_cells_mut();
        while let Some(cell) = itr.next() {
            cell.set_style(self.style);
        }

        let canvas_space = self.get_absolute_canvas_space(area.absolute_layout_space);
        area.transform(canvas_space).draw_stateful_child(&self.drawing, state);
    }
}

impl<T: MountableLayout + StatefulDrawable> MountableLayout for Dragger<T> {
    fn mount_ref(&self) -> &Mount {
        &self.mount
    }

    fn mount_mut(&mut self) -> &mut Mount {
        &mut self.mount
    }

    fn child_ref(&self, i: usize) -> Option<&dyn MountableLayout> {
        match i {
            0 => Some(self.drawing.as_trait_ref()),
            _ => None
        }
    }

    fn child_mut(&mut self, i: usize) -> Option<&mut dyn MountableLayout> {
        match i {
            0 => Some(self.drawing.as_trait_mut()),
            _ => None
        }
    }

    fn relayout(&mut self, relayout: &mut ScreenRelayout) {
        let absolute_window_space = relayout.get_absolute_layout_of(self);
        self.canvas_offset = self.get_constrained_canvas_offset(absolute_window_space);
        let absolute_canvas_space = self.get_absolute_canvas_space(absolute_window_space);

        relayout.input_space_of(self.as_trait_mut(), Space::FULL);
        relayout.children_in_space_of(self.as_trait_mut(), absolute_canvas_space);
    }

    fn on_mouse_input(&mut self, event: ScreenInputEvent) -> bool {
        match event.kind {
            ScreenInputEventKind::Down(point) => {
                self.mouse_location = point;
                false
            },
            ScreenInputEventKind::Drag(point) => {
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