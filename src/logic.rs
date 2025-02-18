// logic.rs
// this module should contain everything related to starting/running the game loop

use rand::Rng;
use std::{collections::HashMap, mem};

use crate::{
    enums::{self, Resource, TileResource},
    render::drawing::map::Tile,
};

use rand::distributions::Uniform;
use tui::style::Color;

type PlayerId = usize;

struct Placement(enums::Building, PlayerId);
struct Plot {
    can_place: bool,
    placement: Option<Placement>,
}

// struct Tile {
//     roll: u8,
//     resource: enums::TileResource,
// }

struct Map {
    plots: Vec<Plot>,
    roads: Vec<Vec<Option<PlayerId>>>,
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

// ---- map generation ----

fn gen_spiral_walk<T>(map: &MapMatrix<T>, start: (usize, usize)) -> Vec<(usize, usize)> {
    let mut frontier = Vec::new();
    let mut walk = Vec::new();
    let mut visited = MapMatrix::<bool>::default();

    frontier.push(start);
    visited[start.1][start.0] = true;

    while !frontier.is_empty() {
        let spiral_frontier = walk.len();
        walk.extend(frontier.drain(..));
        for (x, y) in walk[spiral_frontier..].iter().cloned() {
            for coord in get_adjacent_coords(map, x, y) {
                let Some((nx, ny)) = coord else { continue };
                if visited[ny][nx] {
                    continue;
                };

                visited[ny][nx] = true;
                frontier.push((nx, ny));
            }
        }
    }

    walk
}

const NUMBER_COUNTS: [u8; 13] = [0, 0, 1, 2, 2, 2, 2, 1, 2, 2, 2, 2, 1];
const NUMBER_RARITIES: [u8; 13] = [0, 0, 1, 2, 3, 4, 5, 0, 5, 4, 3, 2, 1];
const ADJACENT_TILE_DELTAS: [(i8, i8); 6] = [(-1, -1), (0, -2), (1, -1), (1, 1), (0, 2), (-1, 1)];

lazy_static! {
    static ref MAP_MATRIX_SPIRAL_WALK: Vec<(usize, usize)> =
        gen_spiral_walk(&MapMatrix::<bool>::default(), (2, 4));
    static ref AVAILABLE_NUMBERS: u16 = {
        let mut available_numbers = 0;
        for (number, count) in NUMBER_COUNTS.iter().enumerate() {
            if *count > 0 {
                available_numbers |= 1 << number;
            }
        }
        available_numbers
    };
}

type MapMatrix<T> = [[T; 5]; 10];

fn get_adjacent_coords<T>(map: &MapMatrix<T>, x: usize, y: usize) -> [Option<(usize, usize)>; 6] {
    ADJACENT_TILE_DELTAS
        .map(|(dx, dy)| (dx + x as i8, dy + y as i8))
        .map(|(nx, ny)| {
            if ny >= 0
                && (ny as usize) < map.len()
                && nx >= 0
                && (nx as usize) < map[ny as usize].len()
            {
                Some((nx as usize, ny as usize))
            } else {
                None
            }
        })
}

fn get_adjacents<T>(map: &MapMatrix<T>, x: usize, y: usize) -> [Option<&T>; 6] {
    get_adjacent_coords(map, x, y).map(|x| x.map(|(nx, ny)| &map[ny][nx]))
}

fn are_coords_adjacent(x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
    let dx = x1.abs_diff(x2);
    let dy = y1.abs_diff(y2);
    (dx == 1 && dy == 1) || (dx == 0 && dy == 2)
}

#[derive(Clone, Copy)]
enum GenTileState {
    Blocked,
    Progress {
        resource: Option<TileResource>,
        number: Option<u8>,
    },
}

impl Default for GenTileState {
    fn default() -> Self {
        GenTileState::Progress {
            resource: None,
            number: None,
        }
    }
}

fn generate_map_matrix() -> MapMatrix<GenTileState> {
    let mut rng = rand::thread_rng();

    let mut number_counts = NUMBER_COUNTS;
    let mut available_numbers = *AVAILABLE_NUMBERS;

    let mut map: MapMatrix<GenTileState> = Default::default();
    map[0][0] = GenTileState::Blocked;
    map[0][4] = GenTileState::Blocked;
    map[8][0] = GenTileState::Blocked;
    map[8][4] = GenTileState::Blocked;
    map[9][1] = GenTileState::Blocked;
    map[9][3] = GenTileState::Blocked;

    // Map (number) generation constraints:
    // No two of the same numbers can be adjacent
    // No two 6s or 8s can be adjacent
    // Adjacent tile rarity cannot sum to more than 12

    let mut encounted_impossible_state = false;
    let mut will_need_another_pass = true;

    let mut candidate_tile_coords = Vec::new();

    let mut pass = 0;
    while will_need_another_pass {
        pass += 1;
        if pass == 20 {
            panic!("Too many passes");
        }
        println!("Pass {}", pass);
        will_need_another_pass = false;

        for (x, y) in MAP_MATRIX_SPIRAL_WALK.iter().cloned() {
            let GenTileState::Progress {
                number: None,
                resource: None,
            } = map[y][x]
            else {
                continue;
            };

            let adjacent_numbers = get_adjacents(&map, x, y).map(|tile| {
                tile.and_then(|state| {
                    if let &GenTileState::Progress { number, .. } = state {
                        number
                    } else {
                        None
                    }
                })
            });

            let candidate_numbers = find_candidate_numbers(adjacent_numbers);
            if candidate_numbers == 0 {
                println!("Impossible state at ({}, {})", x, y);
                encounted_impossible_state = true;
                break;
            }

            let possible_numbers = candidate_numbers & available_numbers;
            if possible_numbers == 0 {
                // sample an already used candidate number from the map state and use that
                candidate_tile_coords.clear();
                for (x2, y2) in MAP_MATRIX_SPIRAL_WALK.iter().cloned() {
                    if are_coords_adjacent(x, y, x2, y2) {
                        continue;
                    }

                    let GenTileState::Progress {
                        number: Some(number),
                        ..
                    } = map[y2][x2]
                    else {
                        continue;
                    };
                    if candidate_numbers & (1 << number) == 0 {
                        continue;
                    }

                    candidate_tile_coords.push((x2, y2, number));
                }

                if candidate_tile_coords.is_empty() {
                    println!("Impossible state at ({}, {})", x, y);
                    encounted_impossible_state = true;
                    break;
                }

                let (x2, y2, number) =
                    candidate_tile_coords[rng.gen_range(0..candidate_tile_coords.len())];

                map[y][x] = GenTileState::Progress {
                    resource: None,
                    number: Some(number),
                };
                map[y2][x2] = GenTileState::Progress {
                    resource: None,
                    number: None,
                };

                println!("Taking {} from ({}, {}) to ({}, {})", number, x2, y2, x, y);

                will_need_another_pass = true;
            } else {
                let mut sampled_index = rng.gen_range(0..possible_numbers.count_ones());
                let mut sampled_number = 0;
                for number in 0..(8 * mem::size_of_val(&possible_numbers)) {
                    if (possible_numbers >> number) & 1 == 1 {
                        if sampled_index == 0 {
                            sampled_number = number;
                            break;
                        }
                        sampled_index -= 1;
                    }
                }

                map[y][x] = GenTileState::Progress {
                    resource: None,
                    number: Some(sampled_number as u8),
                };

                number_counts[sampled_number] -= 1;
                if number_counts[sampled_number] == 0 {
                    available_numbers &= !(1 << sampled_number);
                }

                println!("Putting {} at ({}, {})", sampled_number, x, y);
            }
        }

        if encounted_impossible_state {
            number_counts = NUMBER_COUNTS;
            available_numbers = *AVAILABLE_NUMBERS;
            map = Default::default();

            encounted_impossible_state = false;
            will_need_another_pass = true;
        }
    }

    // sample resources (random sample, no constraints)

    let mut total_resource_count = 3 + 4 + 4 + 3 + 4;
    let mut resource_counts = [
        (Resource::Brick, 3),
        (Resource::Wheat, 4),
        (Resource::Lumber, 4),
        (Resource::Ore, 3),
        (Resource::Wool, 4),
    ];

    for (x, y) in MAP_MATRIX_SPIRAL_WALK.iter().cloned() {
        let GenTileState::Progress {
            resource: tile_resource,
            number: Some(tile_number),
        } = &mut map[y][x]
        else {
            continue;
        };

        if *tile_number == 7 {
            *tile_resource = Some(TileResource::OfDesert);
            continue;
        }

        if total_resource_count == 0 {
            continue;
        }

        let mut resource_index = rng.gen_range(0..total_resource_count);
        for (resource, count) in resource_counts.iter_mut() {
            if *count == 0 {
                continue;
            }

            if resource_index < *count {
                *tile_resource = Some(TileResource::Of(*resource));
                *count -= 1;
                total_resource_count -= 1;
                break;
            }

            resource_index -= *count;
        }
    }

    map
}

pub fn generate_tiles() -> Vec<Tile> {
    generate_map_matrix()
        .iter()
        .enumerate()
        .map(|(y, row)| {
            row.iter().skip(y % 2).step_by(2).filter_map(|state| {
                if let GenTileState::Progress {
                    resource: Some(resource),
                    number: Some(number),
                } = state
                {
                    Some(Tile::new(*number, *resource))
                } else {
                    None
                }
            })
        })
        .flatten()
        .collect::<Vec<_>>()
}

// let map = map.unwrap_or_else(|| {
//     let mut tiles: Vec<Tile> = Uniform::from(2..12) // from a uniform distribution from [2, 12)
//         .sample_iter(&mut rng) // create an iterator that samples from it
//         .take(*map::MAP_TILE_CAPACITY - 1) // sample as many times equal to the map capacity for tiles (minus 1 for desert tile)
//         .map(|roll| Tile::new(
//             if roll == 7 { 12 } else { roll }, // 7 becomes 12 because 7 is not on a tile its a robber round
//             enums::TileResource::Of(rand::random::<enums::Resource>())
//         ))
//         .collect();

//     // insert desert tile at a random location in the tiles vector
//     let desert_tile_index = rng.gen_range(0..=tiles.len());
//     tiles.insert(desert_tile_index, Tile::new(7, enums::TileResource::OfDesert));

//     let ports: Vec<Port> = enums::PortResource::OfAnyKind // sample_iter takes a self type so I need to do PortResource::OfAnyKind instead of just PortResource
//         .sample_iter(&mut rng) // create an iterator that takes samples of PortResource
//         .take(*map::MAP_PORT_CAPACITY) // sample as many times equal to the map capacity for ports
//         .enumerate()
//         .map(|(i, resource)| Port::new(i, resource)) // we want ports containing these resources
//         .collect();

//     Map::new(tiles, ports)
// });

fn find_candidate_numbers(adjacent_numbers: [Option<u8>; 6]) -> u16 {
    let mut candidates = 0b1111111111100;
    let mut max_rarity = 12;
    for (i, number) in adjacent_numbers.into_iter().enumerate() {
        let Some(number) = number else {
            continue;
        };
        candidates &= !(1 << number); // No two of the same numbers can be adjacent
        if number == 6 || number == 8 {
            candidates &= !(1 << 6 | 1 << 8); // No two 6s or 8s can be adjacent
        }

        max_rarity = max_rarity.min(
            12 - NUMBER_RARITIES[number as usize]
                - adjacent_numbers[(i + 1) % adjacent_numbers.len()]
                    .map_or(0, |number2| NUMBER_RARITIES[number2 as usize]),
        );
    }

    for (number, rarity) in NUMBER_RARITIES.iter().enumerate() {
        if *rarity > max_rarity {
            candidates &= !(1 << number); // Adjacent tile rarity cannot sum to more than 12
        }
    }

    candidates
}
