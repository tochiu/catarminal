// logic.rs
// this module should contain everything related to starting/running the game loop

/*
    I think we need:
    Structs to keep track of each player's:
    Resource numbers
    Development Cards
    ID
    "Color Name"
    Turn state


    Game state struct
*/
use crate::enums;
use crate::render::drawing::map::MAP_GRAPH;

use tui::style::Color;

type PlayerId = usize;

struct Placement(enums::Building, PlayerId);
struct Plot {
    can_place: bool,
    placement: Option<Placement>
}

struct Tile {
    roll: u8,
    resource: enums::TileResource
}

struct Map {
    plots: Vec<Plot>,
    roads: Vec<Vec<Option<PlayerId>>>
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

fn find_longest_road(edge_graph: Vec<Vec<usize>>) {
    
}

