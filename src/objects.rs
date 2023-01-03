use rand::Rng;  // Random Number Generator
use petgraph::Graph; // Graph Structure

#[derive(Copy, Clone)]
//pub enum Resource { Brick, Ore, Wheat, Sheep, Lumber }
//pub enum DevelopmentCard { Knight, Victory, Road, YearOfPlenty, Monopoly }

type LandRef = Box<Land>;
type LandVertexRef = Box<LandVertex>;
type PortRef = Box<Port>;
type EdgeRef = Box<Edge>;

struct Board {
    tiles: [Land; 19],
    vertex: [LandVertex; 54],
    ports: [Port; 9]
}

impl Board {
    pub fn generate_board() -> Self {
        let mut tile_gen: Vec<Land> =  Vec::new();
        let mut vertex_gen: Vec<LandVertex> = Vec::new();
        let mut port_gen: Vec<Port> = Vec::new();

        // Vector containing all the possible tile numbers
        let mut remaining_num = vec![0, 2, 3, 3, 4, 4, 5, 5, 6, 6, 8, 8, 9, 9, 10, 10, 11, 11, 12];
        // Vector containing all the possible tile resources
        let mut remaining_resource = 
        vec![None, Some(Resource::Sheep), Some(Resource::Sheep), Some(Resource::Sheep), Some(Resource::Sheep), Some(Resource::Lumber), Some(Resource::Lumber), Some(Resource::Lumber), Some(Resource::Lumber), Some(Resource::Wheat), Some(Resource::Wheat), Some(Resource::Wheat), Some(Resource::Wheat), Some(Resource::Brick), Some(Resource::Brick), Some(Resource::Brick), Some(Resource::Ore), Some(Resource::Ore), Some(Resource::Ore)];
        // Generating all the tiles
        for i in 1..19 {
            // randomizing resource
            let rng_rsc = rand::thread_rng().gen_range(0..remaining_resource.len());
            let rsc = remaining_resource[rng_rsc];
            // randomizing numbers 
            let rng_num = rand::thread_rng().gen_range(0..remaining_num.len());
            let num = remaining_num[rng_num];
            // creating the tile
            let tile = Land::new(i, num, rsc);
            tile_gen.push(tile);
            // removing generated tile from vector
            remaining_resource.remove(rng_rsc);
            // removing generated tile from vector
            remaining_num.remove(rng_num);
        }

        // Vector containing all the possible ports
        let mut remaining_portrsc = vec![None, None, None, None, Some(Resource::Brick), Some(Resource::Ore), Some(Resource::Wheat), Some(Resource::Sheep), Some(Resource::Lumber)];
        // Vector containing all the possible vertex pairs a port can connect to
        let mut vertex_pairs = vec![[1, 2], [4,5], [11, 16], [12, 17], [27, 33], [34, 39], [43, 47], [48, 52], [50, 53]];
        // generating all ports objects
        for i in 1..9 {
            // picking a random port resource
            let rng_portrsc = rand::thread_rng().gen_range(0..remaining_portrsc.len());
            let portrsc = remaining_portrsc[rng_portrsc];
            // assigning port location
            let pairs = vertex_pairs.pop().unwrap();
            // creating single port object
            let port = Port::new(i, portrsc, pairs);
            port_gen.push(port);
            // removing generated port from list 
            remaining_portrsc.remove(rng_portrsc);
        }

        // generating all vertex objects
        for i in 1..54 {
            let mut portrsc = None;
            match i {
                1 | 2 => portrsc = port_gen[0].resource,
                4 | 5 => portrsc = port_gen[1].resource,
                11 |16 => portrsc = port_gen[2].resource,
                12 | 17 => portrsc = port_gen[3].resource,
                27 | 33 => portrsc = port_gen[4].resource,
                34 | 39 => portrsc = port_gen[5].resource,
                43 | 47 => portrsc = port_gen[6].resource,
                48 | 52 => portrsc = port_gen[7].resource,
                50 | 53 => portrsc = port_gen[8].resource,
                _ => ()
            }
            // creating single vertex object
            let vertex = LandVertex::new(i, portrsc);
            vertex_gen.push(vertex);
        }

        // Generate Graph of Vertexes and Edges
        let mut graph = Graph::new(LandVertex { id: val, port: val, building: val, adjacents: val }, Edge { id: val, linked_land: val, linked_vertex: val });


        // coverting from vector to array
        let t_gen = match <[Land; 19]>::try_from(tile_gen) {
            Ok(t_gen) => t_gen,
            _ => panic!("Tile Conversation Failed"),
        };
        let v_gen = match <[LandVertex; 54]>::try_from(vertex_gen) {
            Ok(v_gen) => v_gen,
            _ => panic!("Vertex Conversion Failed!"),
        };
        let p_gen = match <[Port; 9]>::try_from(port_gen) {
            Ok(p_gen) => p_gen,
            _ => panic!("Port Conversion Failed!"),
        };

        // putting it all together
        let board = Board {
            tiles: t_gen,
            vertex: v_gen,
            ports: p_gen
        };
        board
    }
}

// represents a tile on the Catan Board
struct Land {
    id: u8,
    roll: u8, // unsigned 8-bit num; the number associated with a tile to gain resource
    resource: Option<Resource>, // resource gained from tile
    adjacents: [Option<LandRef>; 6], // list of 6 adjacent tiles
    is_robbed: bool // is the robber on this tile?
}

impl Land {
    pub fn new(idx: u8, number: u8, rsc: Option<Resource>) -> Self {
        let tile = Land {
            id: idx,
            roll: number,
            resource: rsc,
            adjacents: [None, None, None, None, None, None],
            is_robbed: false
        };
        tile
    }
}

// a point at the corner of tile(s) where a house can potentially be placed
struct LandVertex {
    id: u8,
    port: Option<Resource>, // What type of port is here if any
    building: Option<Building>, // What type of building is here if any
    adjacents: [Option<Box<Edge>>; 3] // list of adjacent nodes
}

impl LandVertex {
    pub fn new(num: u8, port_type: Option<Resource>) -> Self {
        let vertex = LandVertex {
            id: num,
            port: port_type,
            building: None,
            adjacents: [None, None, None]
        };
        vertex
    }
}

struct Edge { 
    id: u8,
    linked_land: [LandRef; 2],
    linked_vertex: [LandVertexRef; 2]
}

impl Edge {
    pub fn new(idx: u8, lands: [LandRef; 2], vertexes: [LandVertexRef; 2]) -> Self {
        let edge = Edge {
            id: idx,
            linked_land: lands,
            linked_vertex: vertexes,
        };
        edge
    }
}

struct Port {
    id: u8,
    resource: Option<Resource>,
    linked: [u8; 2]
}

impl Port {
    pub fn new(idx: u8, rsc: Option<Resource>, nodes: [u8; 2]) -> Self {
        let port = Port {
            id: idx,
            resource: rsc,
            linked: nodes
        };
        port
    }
}

struct Building {
    is_city: bool,
    owner: Box<Player> 
}

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

/*
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
*/