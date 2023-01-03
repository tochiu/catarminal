// logic.rs
// this module should contain everything related to starting/running the game loop

use std::collections::HashMap;

use crate::enums::{self, TileResource, Resource};
use crate::render::drawing::map::MAP_GRAPH;

use rand::Rng;
use tui::style::Color;

type PlayerId = usize;

struct Placement(enums::Building, PlayerId);
struct Plot {
    can_place: bool,
    placement: Option<Placement>,
}

struct Tile {
    roll: u8,
    resource: enums::TileResource,
}

struct Map {
    //plots: Vec<Plot>,
    //roads: Vec<Vec<Option<PlayerId>>>,
    tiles: Vec<Tile>,
}

impl Map {
    // creates a legal catan map (of resources and numbers)
    pub fn generate_map() -> Self {
        // Vector containing all the possible tile numbers
        let mut remaining_num = vec![0, 2, 3, 3, 4, 4, 5, 5, 6, 6, 8, 8, 9, 9, 10, 10, 11, 11, 12];
        // Vector containing all the possible tile resources
        let mut remaining_resource = vec![
            TileResource::Of(Resource::Wool),
            TileResource::Of(Resource::Wool),
            TileResource::Of(Resource::Wool),
            TileResource::Of(Resource::Wool),
            TileResource::Of(Resource::Lumber),
            TileResource::Of(Resource::Lumber),
            TileResource::Of(Resource::Lumber),
            TileResource::Of(Resource::Lumber),
            TileResource::Of(Resource::Wheat),
            TileResource::Of(Resource::Wheat),
            TileResource::Of(Resource::Wheat),
            TileResource::Of(Resource::Wheat),
            TileResource::Of(Resource::Brick),
            TileResource::Of(Resource::Brick),
            TileResource::Of(Resource::Brick),
            TileResource::Of(Resource::Ore),
            TileResource::Of(Resource::Ore),
            TileResource::Of(Resource::Ore),
            TileResource::OfDesert,
        ];
        let mut tiles: Vec<Tile> =  Vec::new();
        // Generating all the tiles
        for i in 1..19 {
            // randomizing resource
            let rng_rsc = rand::thread_rng().gen_range(0..remaining_resource.len());
            let resource = remaining_resource[rng_rsc];
            // randomizing numbers
            let rng_num = rand::thread_rng().gen_range(0..remaining_num.len());
            let roll = remaining_num[rng_num];
            // creating the tile
            let tile_gen = Tile{roll, resource};
            tiles.push(tile_gen);
            // removing generated resource from vector
            remaining_resource.remove(rng_rsc);
            // removing generated number from vector
            remaining_num.remove(rng_num);
        }
        Map { tiles }
        //Map { plots: (), roads: (), tiles: tile_gen }
    }
}

struct Player {
    id: PlayerId,
    name: String,
    color: Color,
}

struct PlayerState {
    roads_remaining: u8,
    settlements_reamining: u8,
    cities_remaining: u8,
}

struct Game {
    players: Vec<Player>,
    map: Map
}

fn find_longest_road(edge_graph: Vec<Vec<usize>>) {// I think this is for the longest road card
    
}
