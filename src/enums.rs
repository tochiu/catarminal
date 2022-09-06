use tui::style::Color;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

const WOOL_RESOURCE_COLOR: Color = Color::Rgb(140, 181, 014);
const WHEAT_RESOURCE_COLOR: Color = Color::Rgb(240, 185, 032);
const BRICK_RESOURCE_COLOR: Color = Color::Rgb(223, 097, 040);
const LUMBER_RESOURCE_COLOR: Color = Color::Rgb(024, 152, 055);
const ORE_RESOURCE_COLOR: Color = Color::Rgb(059, 065, 061);

#[derive(Debug, Copy, Clone)]
pub enum Resource {
    Ore,
    Wool,
    Wheat,
    Brick,
    Lumber
}

impl Resource {
    pub fn get_color(&self) -> Color {
        match self {
            Self::Ore => ORE_RESOURCE_COLOR,
            Self::Wool => WOOL_RESOURCE_COLOR,
            Self::Wheat => WHEAT_RESOURCE_COLOR,
            Self::Brick => BRICK_RESOURCE_COLOR,
            Self::Lumber => LUMBER_RESOURCE_COLOR
        }
    }
}

impl Distribution<Resource> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Resource {
        match rng.gen_range(0..=4) {
            0 => Resource::Ore,
            1 => Resource::Wool,
            2 => Resource::Wheat,
            3 => Resource::Brick,
            _ => Resource::Lumber
        }
    }
}