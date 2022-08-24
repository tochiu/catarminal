use tui::style::Color;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

const SHEEP_RESOURCE_COLOR: Color = Color::Rgb(140, 181, 014);
const WHEAT_RESOURCE_COLOR: Color = Color::Rgb(240, 185, 032);
const BRICK_RESOURCE_COLOR: Color = Color::Rgb(223, 097, 040);
const TREE_RESOURCE_COLOR: Color = Color::Rgb(024, 152, 055);
const ORE_RESOURCE_COLOR: Color = Color::Rgb(059, 065, 061);

#[derive(Debug)]
pub enum Resource {
    Sheep,
    Wheat,
    Brick,
    Tree,
    Ore
}

impl Resource {
    pub fn get_color(&self) -> Color {
        match self {
            Self::Sheep => SHEEP_RESOURCE_COLOR,
            Self::Wheat => WHEAT_RESOURCE_COLOR,
            Self::Brick => BRICK_RESOURCE_COLOR,
            Self::Tree  => TREE_RESOURCE_COLOR,
            Self::Ore   => ORE_RESOURCE_COLOR,
        }
    }
}

impl Distribution<Resource> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Resource {
        match rng.gen_range(0..=4) {
            0 => Resource::Sheep,
            1 => Resource::Wheat,
            2 => Resource::Brick,
            3 => Resource::Tree,
            _ => Resource::Ore
        }
    }
}