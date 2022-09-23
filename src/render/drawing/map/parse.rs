use super::{
    tile,
    super::{
        shape::*, 
        super::{
            space::*,
            draw::*,
        }
    }
};

use tui::style::{Color, Style};

use std::{fs::File, collections::{HashMap, HashSet}};
use std::io::prelude::*;

#[derive(Debug)]
pub struct MapGraph {
    pub tile_points: Vec<Point2D>,
    pub road_points: Vec<Point2D>,
    pub port_points: Vec<Point2D>,

    pub tile_edges: Vec<Vec<usize>>,
    pub road_edges: Vec<Vec<usize>>,

    pub tile_roads: Vec<Vec<usize>>,
    pub road_tiles: Vec<Vec<usize>>,

    pub port_roads: Vec<Vec<usize>>,
    pub road_ports: Vec<Option<usize>>
}

lazy_static! {
    pub static ref MAP_CONTENT: String = {
        let mut file = File::open("./res/map.txt").expect("Cannot open the file");
        let mut file_str = String::new();
        file.read_to_string(&mut file_str).expect("Cannot read the file");
        file_str
    };

    pub static ref MAP_BKG_DRAW_STRING: DrawableString<'static> = DrawableString::new(MAP_CONTENT.as_str());
    pub static ref MAP_BKG_SHAPE: StringShape<'static> = StringShape::new(&MAP_BKG_DRAW_STRING, Style::default().fg(Color::White), DrawLayout::default());

    pub static ref MAP_GRAPH: MapGraph = {

        let mut tile_points: Vec<Point2D> = Vec::new();
        let mut tile_edges: Vec<Vec<usize>> = Vec::new();
        let mut tile_roads: Vec<Vec<usize>> = Vec::new();

        let mut road_points: Vec<Point2D> = Vec::new();
        let mut road_edges: Vec<Vec<usize>> = Vec::new();
        let mut road_tiles: Vec<Vec<usize>> = Vec::new();
        let mut road_ports: Vec<Option<usize>> = Vec::new();

        let mut port_points: Vec<Point2D> = Vec::new();
        let mut port_roads: Vec<Vec<usize>> = Vec::new();

        let mut road_port_points: Vec<Point2D> = Vec::new();
        let mut road_points_hash: HashMap<Point2D, usize> = HashMap::new();
        let mut tile_edge_set: HashSet<usize> = HashSet::with_capacity(6);
        

        for (x, y, grapheme) in MAP_BKG_DRAW_STRING.iter() {
            if grapheme == "X" {
                port_points.push(Point2D::new(x as i16, y as i16));
                port_roads.push(Vec::new());
            } else if grapheme == "O" {
                road_port_points.push(Point2D::new(x as i16, y as i16));
            } else if grapheme == "[" {
                let tile_outer_semi_height = tile::TILE_SIZE.y as i16/2 + 1;

                let tile_point = Point2D::new(x as i16, y as i16);
                let road_point_left = Point2D::new(
                    tile_point.x - 1, 
                    tile_point.y
                );
                let road_point_right = Point2D::new(
                    tile_point.x + tile::TILE_SIZE.x as i16, 
                    tile_point.y
                );
                let road_point_top_left = Point2D::new(
                    road_point_left.x + tile_outer_semi_height, 
                    road_point_left.y - tile_outer_semi_height
                );
                let road_point_top_right = Point2D::new(
                    road_point_right.x - tile_outer_semi_height, 
                    road_point_right.y - tile_outer_semi_height
                );
                let road_point_bottom_left = Point2D::new(
                    road_point_left.x + tile_outer_semi_height,
                    road_point_left.y + tile_outer_semi_height
                );
                let road_point_bottom_right = Point2D::new(
                    road_point_right.x - tile_outer_semi_height,
                    road_point_right.y + tile_outer_semi_height
                );

                let tile_road_points = vec![
                    road_point_right, 
                    road_point_top_right,
                    road_point_top_left, 
                    road_point_left,
                    road_point_bottom_left, 
                    road_point_bottom_right
                ];
                
                let tile = tile_points.len();

                // roads with indexes greater than or equal to this are newly created
                let min_recent_road = road_points.len();

                // tile_road_points will be mapped to road indicies and collected into this vector
                let mut adjacent_tile_roads: Vec<usize> = Vec::with_capacity(tile_road_points.len());

                // iterate through all the road points bordering the current tile
                for (i, &road_point) in tile_road_points.iter().enumerate() {

                    // check if the road point already has an index
                    if let Some(&road) = road_points_hash.get(&road_point) {
                        
                        // since it does, all tiles adjacent to the road must be adjacent to our tile
                        for &adjacent_tile in road_tiles[road].iter() {
                            // we check for contain because we dont want duplicates in the tile_edges[adjacent_tile] vector
                            if tile_edge_set.contains(&adjacent_tile) {
                                continue
                            }

                            // add tile to adjacent_tile's edge set and vice-versa
                            tile_edge_set.insert(adjacent_tile);
                            tile_edges[adjacent_tile].push(tile);
                        }

                        // add our tile to the list of tiles adjacent to the road and vice-versa
                        road_tiles[road].push(tile);
                        adjacent_tile_roads.push(road);

                        // if the previous road along the tile was newly created then we add each road to eachothers road edge set
                        /*  
                        * NOTE: we do not do this for pre-existing roads because 
                        * if both roads are not new then they are already in each others edge set
                        * newly created roads should always add themselves to exist road edge sets
                        * or existing roads should check if the prev road is newly created to add
                        */
                        if i > 0 && adjacent_tile_roads[i - 1] >= min_recent_road {
                            road_edges[adjacent_tile_roads[i - 1]].push(road);
                            road_edges[road].push(adjacent_tile_roads[i - 1]);
                        }

                        // same logic as above but check for the edge case where we are at the last road and need to connect to the first road
                        if i + 1 == tile_road_points.len() && adjacent_tile_roads[0] >= min_recent_road {
                            road_edges[adjacent_tile_roads[0]].push(road);
                            road_edges[road].push(adjacent_tile_roads[0]);
                        }
                    } else {
                        // new road
                        let road = road_points.len();
                        road_points.push(road_point);
                        road_edges.push(Vec::new()); // to be filled in
                        road_ports.push(None);

                        // add our tile to the list of tiles adjacent to the road and vice-versa
                        road_tiles.push(vec![tile]); // we only know our current tile is adjacent to the road
                        adjacent_tile_roads.push(road);

                        road_points_hash.insert(road_point, road); // insert into hashmap so the same calculated points lead to the road index

                        // add our road to the edge set of the previously created road
                        if i > 0 {
                            road_edges[adjacent_tile_roads[i - 1]].push(road);
                            road_edges[road].push(adjacent_tile_roads[i - 1]);
                        }

                        // same logic as above but check for the edge case where we are at the last road and need to connect to the first road
                        if i + 1 == tile_road_points.len() {
                            road_edges[adjacent_tile_roads[0]].push(road);
                            road_edges[road].push(adjacent_tile_roads[0]);
                        }
                    }
                }

                tile_points.push(tile_point);
                tile_roads.push(adjacent_tile_roads);
                tile_edges.push(tile_edge_set.drain().collect()); // collect adjacent tiles into tile_edge vector
                // drain so set is empty can be reused by the next tile without need to create a new set
            }
        }

        for road_point in road_port_points {
            let &road = road_points_hash.get(&road_point).unwrap();
            let (road_port, _) = port_points
                .iter()
                .enumerate()
                .max_by_key(|(_, &port_point)| 
                    (port_point.x.abs_diff(road_point.x) as u32).pow(2) + 
                    (port_point.y.abs_diff(road_point.y) as u32).pow(2)
                )
                .unwrap();

            road_ports[road] = Some(road_port);
            port_roads[road_port].push(road);
        }

        MapGraph { 
            tile_points, 
            road_points,
            port_points,
            tile_edges, 
            road_edges, 
            tile_roads, 
            road_tiles,
            port_roads,
            road_ports
        }
    };

    pub static ref MAP_TILE_CAPACITY: usize = MAP_GRAPH.tile_points.len();
    pub static ref MAP_PORT_CAPACITY: usize = MAP_GRAPH.port_points.len();
}