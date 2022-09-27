use super::super::{
    shape::{BitShape128, Shape128},
    super::{
        draw::{DrawLayout, Layoutable, Drawable},
        mount::{Mount, MountableLayout},
        screen::{ScreenArea, ScreenAnimationService},
        space::*,
        anim::*
    }
};

use crate::enums;

use tui::{style::{Style, Color}, buffer::Cell};

// PLACEMENT TRAIT

const PLACEMENT_Y_OFFSET: i16 = -10;

pub trait Placement: MountableLayout {
    fn set_placement_style(&mut self, style: Style);
    fn get_placement_space(&self) -> Space;
    fn build(&mut self, style: Style, service: &mut ScreenAnimationService) {
        self.set_placement_style(style);
        let layout = self.layout_mut();
        if !layout.is_visible {
            layout.set_visible(true);
        }
    
        let mut start_space = layout.space;
        start_space.position.y.offset += PLACEMENT_Y_OFFSET;
        let end_space = self.get_placement_space();
    
        self.animate_space_from(service, start_space, end_space, 0.5, EasingStyle::Cubic, EasingDirection::Out);
    } 
}

// ROAD

#[derive(Debug)]
pub struct Road {
    start: Point2D,
    change: Point2D,
    steps: u8,
    mount: Mount,
    style: Style,
    placement_space: Space,
    pub layout: DrawLayout,
}

impl Road {
    pub fn new(map_from: Point2D, map_to: Point2D, style: Style, mut layout: DrawLayout) -> Self {
        let top_left_point = Point2D::new(map_from.x.min(map_to.x) - 1, map_from.y.min(map_to.y));
        let from = map_from - top_left_point;
        let to = map_to - top_left_point;
        let dx = to.x - from.x;
        let dy = to.y - from.y;

        layout
            .set_size(UDim2::from_offset(dx.abs() + 3, dy.abs() + 3))
            .set_position(UDim2::from_point2d(top_left_point));
        
        Road {
            start: from,
            change: Point2D::new(dx.signum(), dy.signum()),
            steps: dx.abs().max(dy.abs()) as u8,
            mount: Mount::default(),
            style,
            placement_space: layout.space,
            layout
        }
    }
}

impl Placement for Road {
    fn get_placement_space(&self) -> Space { self.placement_space }
    fn set_placement_style(&mut self, style: Style) { self.style = style; }
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

// BUILDING

lazy_static! {
    static ref SETTLEMENT_BITSHAPE: BitShape128 = BitShape128::new(0b0111011011, Size2D::new(5, 2));
    static ref CITY_BITSHAPE: BitShape128 = BitShape128::new(0b011100011111111101111, Size2D::new(7, 3));
    static ref HOLE_CELL: Cell = Cell::default().set_bg(Color::Black).clone();
}

const SETTLEMENT_BITSHAPE_HOLE: AbsoluteSpace = AbsoluteSpace::new(2, 1, 1, 1);
const CITY_BITSHAPE_HOLE: AbsoluteSpace = AbsoluteSpace::new(2, 2, 2, 1);

#[derive(Debug)]
pub struct Building {
    shape: &'static BitShape128,
    hole: AbsoluteSpace,
    placement_space: Space,
    
    mount: Mount,

    pub kind: enums::Building,

    pub style: Style,
    pub layout: DrawLayout,
}

impl Building {
    pub fn new(kind: enums::Building, style: Style, mut layout: DrawLayout) -> Self {
        let shape: &'static BitShape128 = if kind == enums::Building::Settlement { &SETTLEMENT_BITSHAPE } else { &CITY_BITSHAPE };
        layout.set_size(UDim2::from_size2d(shape.size));
        Building {
            shape,
            hole: if kind == enums::Building::Settlement { SETTLEMENT_BITSHAPE_HOLE } else { CITY_BITSHAPE_HOLE },
            placement_space: layout.space,
            kind,
            mount: Mount::default(),
            style,
            layout,
        }
    }
}

impl Placement for Building {
    fn get_placement_space(&self) -> Space { self.placement_space }
    fn set_placement_style(&mut self, style: Style) { self.style = style; }
}

impl Drawable for Building {
    fn draw(&self, mut area: ScreenArea) {
        area.draw_child(&Shape128::new(self.shape, " ", self.style, DrawLayout::default()));
        for point in self.hole {
            if let Some(cell) = area.mut_cell_at(point) {
                *cell = HOLE_CELL.clone();
            }
        }
    }
}

impl Layoutable for Building {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl MountableLayout for Building {
    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }
    fn child_ref(&self, _: usize) -> Option<&dyn MountableLayout> { None }
    fn child_mut(&mut self, _: usize) -> Option<&mut dyn MountableLayout> { None }
}