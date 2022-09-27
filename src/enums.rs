/* common enums useful to the entire application */

use tui::style::Color;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

/* consts */

const WOOL_RESOURCE_COLOR: Color = Color::Rgb(140, 181, 014);
const WHEAT_RESOURCE_COLOR: Color = Color::Rgb(224, 175, 51);
const BRICK_RESOURCE_COLOR: Color = Color::Rgb(223, 97, 40);
const LUMBER_RESOURCE_COLOR: Color = Color::Rgb(9, 74, 29);
const ORE_RESOURCE_COLOR: Color = Color::Rgb(164, 170, 166);
const DESERT_RESOURCE_COLOR: Color = Color::Rgb(217, 210, 149);

const ORE_RESOURCE_SYMBOL: &'static str = "🪨";
const WOOL_RESOURCE_SYMBOL: &'static str = "🐑";
const WHEAT_RESOURCE_SYMBOL: &'static str = "🌾";
const BRICK_RESOURCE_SYMBOL: &'static str = "🧱";
const LUMBER_RESOURCE_SYMBOL: &'static str = "🌲";
const DESERT_RESOURCE_SYMBOL: &'static str = "🌵";
const ANY_RESOURCE_SYMBOL: &'static str = "?";

const PORT_SPECIFIC_RESOURCE_TRADING_RATIO: (u32, u32) = (2, 1);
const PORT_ANY_RESOURCE_TRADING_RATIO: (u32, u32) = (3, 1);

/* enums */

/* the main resources in the game */
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
            Self::Ore => ORE_RESOURCE_SYMBOL,
            Self::Wool => WOOL_RESOURCE_SYMBOL,
            Self::Wheat => WHEAT_RESOURCE_SYMBOL,
            Self::Brick => BRICK_RESOURCE_SYMBOL,
            Self::Lumber => LUMBER_RESOURCE_SYMBOL
        }
    }
}

/* impl Distribution to sample random Resource */
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

/* for a given transaction, port resources will accept either 
    1. a number of exclusively one of the main resources (the kind is predetermined)
    2. a number of exclusively one of the main resources (but you can choose which kind) */
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PortResource {
    Of(Resource),
    OfAnyKind
}

impl PortResource {
    pub fn get_symbol(&self) -> &'static str {
        match self {
            Self::OfAnyKind => ANY_RESOURCE_SYMBOL,
            Self::Of(resource) => resource.get_symbol()
        }
    }

    pub fn get_ratio(&self) -> (u32, u32) {
        match self {
            Self::OfAnyKind => PORT_ANY_RESOURCE_TRADING_RATIO,
            Self::Of(_) => PORT_SPECIFIC_RESOURCE_TRADING_RATIO
        }
    }
}

/* impl Distribution to sample random PortResource */
impl Distribution<PortResource> for PortResource {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PortResource {
        // not gen_range(0..2) because there are 5 resource types, sample weighting needs to account for this
        if rng.gen_range(0..Resource::NUM_TYPES as usize + 1) == 0 {
            Self::OfAnyKind
        } else {
            Self::Of(rand::random::<Resource>())
        }
    }
}

/* Tiles can either contain a resource or be a desert */
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TileResource {
    OfDesert,
    Of(Resource)
}

impl TileResource {
    pub fn get_symbol(&self) -> &'static str {
        match self {
            Self::OfDesert => DESERT_RESOURCE_SYMBOL,
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

/* impl Distribution to sample random TileResource */
impl Distribution<TileResource> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TileResource {
        // not ren_range(0..2) because there are 5 resource types, sampler needs to be weighted accordingly
        if rng.gen_range(0..Resource::NUM_TYPES as usize + 1) == 0 {
            TileResource::OfDesert
        } else {
            TileResource::Of(rand::random::<Resource>())
        }
    }
}

/* Kinds of buildings that can exist on a road point */
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Building {
    Settlement,
    City
}

/* impl Distribution to sample random Building */
impl Distribution<Building> for Building {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Building {
        match rng.gen_range(0..=1) {
            0 => Building::Settlement,
            _ => Building::City
        }
    }
}