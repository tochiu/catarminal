// logic.rs
// this module should contain everything related to starting/running the game loop

use crate::enums;
use crate::render::drawing::map::MAP_GRAPH;

use rand::Rng;
use tui::style::Color;

type PlayerId = usize;

struct Placement(enums::Building, PlayerId);
struct Plot {
    can_place: bool,
    placement: Option<Placement>,
}

pub struct Tile {
    pub roll: u8,
    pub resource: enums::TileResource,
}

pub struct Port {
    pub id: u8,
    pub resource: enums::PortResource,
}

pub struct Map {
    //plots: Vec<Plot>,
    //roads: Vec<Vec<Option<PlayerId>>>,
    pub tiles: Vec<Tile>,
    pub ports: Vec<Port>,
}

impl Map {
    // creates a legal catan map (of resources and numbers)
    pub fn generate_map() -> Self {
        // Vector containing all the possible tile numbers
        let mut remaining_num = vec![7, 2, 3, 3, 4, 4, 5, 5, 6, 6, 8, 8, 9, 9, 10, 10, 11, 11, 12];
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
            //TileResource::OfDesert,
        ];
        let mut tiles: Vec<Tile> = Vec::new();
        //
        // Generating all the tiles
        //
        for i in 0..19 {
            // randomizing numbers
            let rng_num = rand::thread_rng().gen_range(0..remaining_num.len());
            let roll = remaining_num[rng_num];
            let mut resource = TileResource::OfDesert;
            
            // checking if desert was not chosen
            if roll != 7 {
                // randomizing resource
                let rng_rsc = rand::thread_rng().gen_range(0..remaining_resource.len());
                resource = remaining_resource[rng_rsc];
                // removing generated resource from vector
                remaining_resource.remove(rng_rsc);
            }
            
            // creating the tile
            let tile_gen = Tile { roll, resource };
            // saving the object
            tiles.push(tile_gen);
            // removing generated number from vector
            remaining_num.remove(rng_num);
        }
        // Vector containing all the possible ports
        let mut remaining_portrsc = vec![
            PortResource::OfAnyKind,
            PortResource::OfAnyKind,
            PortResource::OfAnyKind,
            PortResource::OfAnyKind,
            PortResource::Of(Resource::Brick),
            PortResource::Of(Resource::Ore),
            PortResource::Of(Resource::Wheat),
            PortResource::Of(Resource::Wool),
            PortResource::Of(Resource::Lumber),
        ];
        let mut ports: Vec<Port> = Vec::new();
        //
        // generating all the ports
        //
        for id in 0..9 {
            // picking a random port resource
            let rng_portrsc = rand::thread_rng().gen_range(0..remaining_portrsc.len());
            let resource = remaining_portrsc[rng_portrsc];
            // creating single port object
            let port = Port{id, resource};
            // saving the object
            ports.push(port);
            // removing generated port from list
            remaining_portrsc.remove(rng_portrsc);
        }
        Map { tiles, ports }
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
    map: Map,
}

fn find_longest_road(edge_graph: Vec<Vec<usize>>) { // I think this is for the longest road card
}
