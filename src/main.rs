enum Resource { Brick, Ore, Wheat, Sheep, Lumber }
enum PortResource { Specific(Resource), Mystery }
enum DevelopmentCard { Knight, Victory, Road, YearOfPlenty, Monopoly }

type LandRef = Box<Land>;

// represents a tile on the Catan Board
struct Land {
    roll: u8, // unsigned 8-bit num; the number associated with a tile to gain resource
    resource: Option<Resource>, // resource gained from tile
    adjacents: [Option<LandRef>; 6], // list of 6 adjacent tiles
    is_robbed: bool // is the robber on this tile?
}



impl Land {
    pub fn new() -> Self {
        Land {
            roll: 0,
            resource: None,
            adjacents: [None, None, None, None, None, None],
            is_robbed: false
        }
    }

    pub fn set_adjacent_land(&mut self, index: usize, land: Option<LandRef>) {

    }
}

struct Building {
    is_city: bool,
    owner: Box<Player> 
}

// a point at the corner of tile(s) where a house can potentially be placed
struct LandVertex {
    port: Option<PortResource>, // What type of port is here if any
    building: Option<Building>, // What type of building is here if any
    adjacents: [Option<Box<LandEdge>>; 3] // list of adjacent nodes
}

struct LandEdge { }

// represents a user playing the game
struct Player {
    id: u8, // unique id given to each player
    name: String, // screen name of each player
    score: u8,
    buildings: Vec<Box<Building>>,
    development_cards_used: [u8; 5],
    development_cards_remaining: [u8; 5],
    placements_remaining: [u8; 3],
    resources: [u8; 5],
}

// represents the bank who distributes resources
struct Bank {
    resources: [u8; 5],
    development_cards: [u8; 5]
}


fn expand_lands_frontier(mut lands: Vec<LandRef>, mut frontier: Vec<LandRef>) {
    let mut new_frontier: Vec<Land> = Vec::with_capacity(frontier.len() + 6 - frontier.len() % 6);

    for frontier_land in frontier.iter() {
        for (i, adjacent_land) in frontier_land.adjacents.iter().enumerate() {
            if adjacent_land.is_some() {
                continue;
            }

            let mut new_land = Land::new();
            //new_land.adjacents[i + 3 % 6] = adjacent_land;
        }
    }
}

fn main() {
    println!("ran!");
}

// fn generate_lands(min_width: u8, max_width: u8) -> &[Land] {

// }

// pub struct Junction {

//     paths: {
//         north: Path,
//         seast: Path,
//         swest: Path
//     },

//     lands: {
//         south: Land,
//         neast: Land,
//         nwest: Land
//     }
// }



// pub struct Path {

// }