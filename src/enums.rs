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

#[derive(Debug, Copy, Clone)]
pub enum PortResource {
    Kind(Resource),
    Any
}

impl PortResource {
    pub fn get_symbol(&self) -> &'static str {
        match self {
            Self::Any => "?",
            Self::Kind(resource) => resource.get_symbol()
        }
    }

    pub fn get_ratio(&self) -> (u32, u32) {
        match self {
            Self::Any => (3, 1),
            Self::Kind(_) => (2, 1)
        }
    }
}

impl Distribution<PortResource> for PortResource {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PortResource {
        if rng.gen_range(0..6) == 0 {
            Self::Any
        } else {
            Self::Kind(rand::random())
        }
    }
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
    pub fn get_symbol(&self) -> &'static str {
        match self {
            Self::Ore => "🪨",
            Self::Wool => "🐑",
            Self::Wheat => "🌾",
            Self::Brick => "🧱",
            Self::Lumber => "🌲"
        }
    }
}

impl Distribution<Resource> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Resource {
        match rng.gen_range(0..5) {
            0 => Resource::Ore,
            1 => Resource::Wool,
            2 => Resource::Wheat,
            3 => Resource::Brick,
            _ => Resource::Lumber
        }
    }
}