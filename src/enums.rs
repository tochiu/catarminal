use tui::style::Color;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

const WOOL_RESOURCE_COLOR: Color = Color::Rgb(140, 181, 014);
const WHEAT_RESOURCE_COLOR: Color = Color::Rgb(224, 175, 51);
const BRICK_RESOURCE_COLOR: Color = Color::Rgb(223, 97, 40);
const LUMBER_RESOURCE_COLOR: Color = Color::Rgb(9, 74, 29);
const ORE_RESOURCE_COLOR: Color = Color::Rgb(164, 170, 166);
const DESERT_RESOURCE_COLOR: Color = Color::Rgb(217, 210, 149);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Resource {
    Ore,
    Wool,
    Wheat,
    Brick,
    Lumber
}

impl Resource {
    const NUM_TYPES: usize = 5;

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PortResource {
    Of(Resource),
    OfAnyKind
}

impl PortResource {
    pub fn get_symbol(&self) -> &'static str {
        match self {
            Self::OfAnyKind => "?",
            Self::Of(resource) => resource.get_symbol()
        }
    }

    pub fn get_ratio(&self) -> (u32, u32) {
        match self {
            Self::OfAnyKind => (3, 1),
            Self::Of(_) => (2, 1)
        }
    }
}

impl Distribution<PortResource> for PortResource {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PortResource {
        if rng.gen_range(0..Resource::NUM_TYPES as usize + 1) == 0 {
            Self::OfAnyKind
        } else {
            Self::Of(rand::random::<Resource>())
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TileResource {
    OfDesert,
    Of(Resource)
}

impl TileResource {
    pub fn get_symbol(&self) -> &'static str {
        match self {
            Self::OfDesert => "🌵",
            Self::Of(resource) => resource.get_symbol()
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            Self::OfDesert => DESERT_RESOURCE_COLOR,
            Self::Of(resource) => resource.get_color()
        }
    }
}

impl Distribution<TileResource> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TileResource {
        if rng.gen_range(0..Resource::NUM_TYPES as usize + 1) == 0 {
            TileResource::OfDesert
        } else {
            TileResource::Of(rand::random::<Resource>())
        }
    }
}