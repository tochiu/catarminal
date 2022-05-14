enum Resource { Brick, Ore, Wheat, Sheep, Lumber }
enum PortResource {
    Specific(Resource),
    Mystery
}

struct Land {
    roll: u8,
    resource: Option<Resource>,
    adjacents: [Option<Box<Land>>; 6]
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