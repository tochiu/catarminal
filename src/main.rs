enum Resource { Brick, Ore, Wheat, Sheep, Lumber }
enum PortResource { Specific(Resource), Mystery }
enum NodeBuilding { House, City}
enum DevelopmentName { Knight, Victory Point, Road Building, Year of Plenty, Monopoly}

// represents a tile on the Catan Board
struct Land {
    roll: u8, // unsigned 8-bit num; the number associated with a tile to gain resource
    resource: Option<Resource>, // resource gained from tile
    adjacents: [Option<Box<Land>>; 6], // list of 6 adjacent tiles
    robber: bool // is the robber on this tile?
}

impl Land {
    pub fn new() -> Land {
        Land {
            roll: 0,
            resource: None,
            adjacents: [None, None, None, None, None, None]
        }
    }
}

// a point at the corner of tile(s) where a house can potentially be placed
struct Node {
    port: Option<PortResource>, // What type of port is here if any
    status: Option<NodeBuilding>, // What type of building is here if any
    player: Player, // Player who owns building here if any
    adjacency: [None, None, None] // list of adjacent nodes
}

// represents a user playing the game
struct Player {
    id: u8, // unique id given to each player
    name: String, // screen name of each player
    public_score: u8, // victory points visible to other players
    private_score: u8, // victory points only visible to this player
    houses_left: u8, // number of potential houses player can build
    cities_left: u8, // number of potential cities a player can build
    roads_left: u8, // number of potential roads a player can build
    devel_cards: Vec new(), // list of development cards a player currently has
    brick_count: u8, // how much brick player curently has
    ore_count: u8, // how much ore player curently has
    wheat_count: u8, // how much wheat player curently has
    sheep_count: u8, // how much sheep player curently has
    lumber_count: u8, // how much lumber player curently has
}

// represents the bank who distributes resources
struct Bank {
    brick_available: u8, // how much brick is left in the bank
    ore_available: u8, // how much ore is left in the bank
    wheat_available: u8, // how much wheat is left in the bank
    sheep_available: u8, // how much sheep is left in the bank
    lumber_available: u8, // how much lumber is left in the bank
    devel_cards: Vec new() // list of undrawn development cards
}


fn expand_lands_frontier(mut lands: Vec<Land>, mut frontier: Vec<Land>) {
    let mut new_frontier: Vec<Land> = Vec::with_capacity(frontier.len() + 6 - frontier.len() % 6);

    for frontier_land in frontier.iter() {
        for (i, adjacent_land) in frontier_land.adjacents.iter().enumerate() {
            if adjacent_land.is_some() {
                continue;
            }

            let new_land = Box::new(Land::new());
            //new_land*
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