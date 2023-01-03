use super::tile;

use crate::render::{space::*, shape::*, draw::DrawLayout};

use tui::style::{Color, Style};

use std::{fs::File, collections::{HashMap, HashSet}};
use std::io::prelude::*;

#[derive(Debug)]
pub struct MapGraph {
    pub tile_anchor_points: Vec<Point2D>,
    pub plot_points: Vec<Point2D>,
    pub port_points: Vec<Point2D>,

    pub tile_edges: Vec<Vec<usize>>,
    pub plot_edges: Vec<Vec<usize>>,

    pub tile_plots: Vec<Vec<usize>>,
    pub plot_tiles: Vec<Vec<usize>>,

    pub port_plots: Vec<Vec<usize>>,
    pub plot_ports: Vec<Option<usize>>
}

lazy_static! {
    static ref MAP_CONTENT: String = {
        let mut file = File::open("./res/map.txt").expect("Cannot open the file");
        let mut file_str = String::new();
        file.read_to_string(&mut file_str).expect("Cannot read the file");
        file_str
    };

    // TODO: make map bkg a BitShape/Shape because now we only care about the symbols for parsing purposes, they dont get rendered
    pub static ref MAP_BKG_DRAW_STRING: DrawableString<'static> = DrawableString::new(MAP_CONTENT.as_str());
    pub static ref MAP_BKG_SHAPE: StringShape<'static> = StringShape::new(&MAP_BKG_DRAW_STRING, Style::default().fg(Color::White), DrawLayout::default());

    pub static ref MAP_GRAPH: MapGraph = {

        let mut tile_anchor_points: Vec<Point2D> = Vec::new();
        let mut tile_edges: Vec<Vec<usize>> = Vec::new();
        let mut tile_plots: Vec<Vec<usize>> = Vec::new();

        let mut plot_points: Vec<Point2D> = Vec::new();
        let mut plot_edges: Vec<Vec<usize>> = Vec::new();
        let mut plot_tiles: Vec<Vec<usize>> = Vec::new();
        let mut plot_ports: Vec<Option<usize>> = Vec::new();

        let mut port_points: Vec<Point2D> = Vec::new();
        let mut port_plots: Vec<Vec<usize>> = Vec::new();

        let mut plot_port_points: Vec<Point2D> = Vec::new();
        let mut plot_points_hash: HashMap<Point2D, usize> = HashMap::new();
        let mut tile_edge_set: HashSet<usize> = HashSet::with_capacity(6);
        

        for (x, y, grapheme) in MAP_BKG_DRAW_STRING.iter() {
            if grapheme == "X" {
                port_points.push(Point2D::new(x as i16, y as i16));
                port_plots.push(Vec::new());
            } else if grapheme == "O" {
                plot_port_points.push(Point2D::new(x as i16, y as i16));
            } else if grapheme == "[" {
                let tile_outer_semi_height = tile::TILE_SIZE.y as i16/2 + 1;

                let tile_point = Point2D::new(x as i16, y as i16);
                let plot_point_left = Point2D::new(
                    tile_point.x - 1, 
                    tile_point.y
                );
                let plot_point_right = Point2D::new(
                    tile_point.x + tile::TILE_SIZE.x as i16, 
                    tile_point.y
                );
                let plot_point_top_left = Point2D::new(
                    plot_point_left.x + tile_outer_semi_height, 
                    plot_point_left.y - tile_outer_semi_height
                );
                let plot_point_top_right = Point2D::new(
                    plot_point_right.x - tile_outer_semi_height, 
                    plot_point_right.y - tile_outer_semi_height
                );
                let plot_point_bottom_left = Point2D::new(
                    plot_point_left.x + tile_outer_semi_height,
                    plot_point_left.y + tile_outer_semi_height
                );
                let plot_point_bottom_right = Point2D::new(
                    plot_point_right.x - tile_outer_semi_height,
                    plot_point_right.y + tile_outer_semi_height
                );

                let tile_plot_points = vec![
                    plot_point_right, 
                    plot_point_top_right,
                    plot_point_top_left, 
                    plot_point_left,
                    plot_point_bottom_left, 
                    plot_point_bottom_right
                ];
                
                let tile = tile_anchor_points.len();

                // plots with indexes greater than or equal to this are newly created
                let min_recent_plot = plot_points.len();

                // tile_plot_points will be mapped to plot indicies and collected into this vector
                let mut adjacent_tile_plots: Vec<usize> = Vec::with_capacity(tile_plot_points.len());

                // iterate through all the plot points bordering the current tile
                for (i, &plot_point) in tile_plot_points.iter().enumerate() {

                    // check if the plot point already has an index
                    if let Some(&plot) = plot_points_hash.get(&plot_point) {
                        
                        // since it does, all tiles adjacent to the plot must be adjacent to our tile
                        for &adjacent_tile in plot_tiles[plot].iter() {
                            // we check for contain because we dont want duplicates in the tile_edges[adjacent_tile] vector
                            if tile_edge_set.contains(&adjacent_tile) {
                                continue
                            }

                            // add tile to adjacent_tile's edge set and vice-versa
                            tile_edge_set.insert(adjacent_tile);
                            tile_edges[adjacent_tile].push(tile);
                        }

                        // add our tile to the list of tiles adjacent to the plot and vice-versa
                        plot_tiles[plot].push(tile);
                        adjacent_tile_plots.push(plot);

                        // if the previous plot along the tile was newly created then we add each plot to eachothers plot edge set
                        /*  
                        * NOTE: we do not do this for pre-existing plots because 
                        * if both plots are not new then they are already in each others edge set
                        * newly created plots should always add themselves to exist plot edge sets
                        * or existing plots should check if the prev plot is newly created to add
                        */
                        if i > 0 && adjacent_tile_plots[i - 1] >= min_recent_plot {
                            plot_edges[adjacent_tile_plots[i - 1]].push(plot);
                            plot_edges[plot].push(adjacent_tile_plots[i - 1]);
                        }

                        // same logic as above but check for the edge case where we are at the last plot and need to connect to the first plot
                        if i + 1 == tile_plot_points.len() && adjacent_tile_plots[0] >= min_recent_plot {
                            plot_edges[adjacent_tile_plots[0]].push(plot);
                            plot_edges[plot].push(adjacent_tile_plots[0]);
                        }
                    } else {
                        // new plot
                        let plot = plot_points.len();
                        plot_points.push(plot_point);
                        plot_edges.push(Vec::new()); // to be filled in
                        plot_ports.push(None);

                        // add our tile to the list of tiles adjacent to the plot and vice-versa
                        plot_tiles.push(vec![tile]); // we only know our current tile is adjacent to the plot
                        adjacent_tile_plots.push(plot);

                        plot_points_hash.insert(plot_point, plot); // insert into hashmap so the same calculated points lead to the plot index

                        // add our plot to the edge set of the previously created plot
                        if i > 0 {
                            plot_edges[adjacent_tile_plots[i - 1]].push(plot);
                            plot_edges[plot].push(adjacent_tile_plots[i - 1]);
                        }

                        // same logic as above but check for the edge case where we are at the last plot and need to connect to the first plot
                        if i + 1 == tile_plot_points.len() {
                            plot_edges[adjacent_tile_plots[0]].push(plot);
                            plot_edges[plot].push(adjacent_tile_plots[0]);
                        }
                    }
                }

                tile_anchor_points.push(tile_point);
                tile_plots.push(adjacent_tile_plots);
                tile_edges.push(tile_edge_set.drain().collect()); // collect adjacent tiles into tile_edge vector
                // drain so set is empty can be reused by the next tile without need to create a new set
            }
        }

        for plot_point in plot_port_points {
            let &plot = plot_points_hash.get(&plot_point).unwrap();
            let (plot_port, _) = port_points
                .iter()
                .enumerate()
                .min_by_key(|(_, &port_point)| 
                    (port_point.x.abs_diff(plot_point.x) as u32).pow(2) + 
                    (port_point.y.abs_diff(plot_point.y) as u32).pow(2)
                )
                .unwrap();

            plot_ports[plot] = Some(plot_port);
            port_plots[plot_port].push(plot);
        }

        MapGraph { 
            tile_anchor_points, 
            plot_points,
            port_points,
            tile_edges, 
            plot_edges, 
            tile_plots, 
            plot_tiles,
            port_plots,
            plot_ports
        }
    };

    pub static ref MAP_TILE_CAPACITY: usize = MAP_GRAPH.tile_anchor_points.len();
    pub static ref MAP_PORT_CAPACITY: usize = MAP_GRAPH.port_points.len();
}