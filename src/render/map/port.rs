use super::super::{
    draw::*,
    space::*,
    screen::*
};

use crate::enums;

use tui::style::Color;

const PORT_SIZE: Size2D = Size2D::new(3, 2);
const PORT_SYMBOL_OFFSET: Point2D = Point2D::new(1, 0);
const PORT_RATIO_OFFSET: Point2D = Point2D::new(0, 1);

#[derive(Debug)]
pub struct Port {
    pub layout: DrawLayout,
    resource: enums::PortResource,
    ratio: String
}

impl Port {
    pub fn new(resource: enums::PortResource) -> Self {
        let (num_give, num_get) = resource.get_ratio();
        Port {
            resource,
            ratio: [char::from_digit(num_give, 10).unwrap(), ':', char::from_digit(num_get, 10).unwrap()].iter().collect(),
            layout: DrawLayout::default()
                .set_size(UDim2::from_size2d(PORT_SIZE))
                .set_anchor(Scale2D::new(0.5, 0.0))
                .clone() 
        }
    }
}

impl Layoutable for Port {
    fn layout_ref(&self) -> &DrawLayout {
        &self.layout
    }
}

impl Drawable for Port {
    fn draw(&self, mut area: ScreenArea) {
        area.draw_unicode_line(self.resource.get_symbol(), PORT_SYMBOL_OFFSET, Color::White);
        area.draw_string_line(&self.ratio, PORT_RATIO_OFFSET, Color::White);
    }
}