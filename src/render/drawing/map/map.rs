use super::{
    parse,
    Tile,
    Road,
    port::{self, Port},
    super::{
        shape::*, 
        super::{
            space::*,
            draw::*,
            screen::*,
            mount::*,
            anim::*,
            iter::CustomIterator
        }
    }
};

use crate::enums;

use tui::style::{Color, Style};

pub const MAP_SAND_COLOR: Color = Color::Rgb(221, 178, 100);
pub const MAP_OCEAN_COLOR: Color = Color::Rgb(9, 103, 166);

const ROBBER_OFFSET: Point2D = Point2D::new(8, -4); // robber offset from tile offset

lazy_static! {
    static ref ROBBER_BITSHAPE: BitShape128 = BitShape128::new(0b011101111101110111111111111111, Size2D::new(5, 6));
    static ref ROBBER_STYLE: Style = Style::default().bg(Color::Magenta);
}

#[derive(Debug)]
pub struct Map {
    bkg: &'static StringShape<'static>,
    tiles: Vec<Tile>,
    ports: Vec<Port>,
    roads: Vec<Vec<Option<Road>>>,
    road_index: Vec<(usize, usize)>,
    robber: DrawLeaf<Shape128>,
    layout: DrawLayout,
    mount: Mount
}

impl Map {
    pub fn new(tiles: Vec<Tile>, ports: Vec<Port>) -> Self {
        let robber_init_tile_position = parse::MAP_GRAPH.tile_points
            .get(
                tiles
                    .iter()
                    .position(|tile| tile.resource == enums::TileResource::OfDesert)
                    .unwrap_or(0)
            )
            .cloned()
            .unwrap_or_default();

        let robber = DrawLeaf::new(
            Shape128::new(&ROBBER_BITSHAPE, " ", *ROBBER_STYLE, DrawLayout::FULL), 
            DrawLayout::default()
                .set_size(UDim2::from_size2d(ROBBER_BITSHAPE.size))
                .set_anchor(Scale2D::new(0.5, 1.0))
                .set_position(UDim2::from_point2d(robber_init_tile_position + ROBBER_OFFSET))
                .clone()
        );

        let roads = parse::MAP_GRAPH.road_edges
            .iter()
            .enumerate()
            .map(|(from_road, edges)| {
                edges
                    .iter()
                    .map(|&to_road| {
                        if from_road < to_road { 
                            Some(Road::new(
                                parse::MAP_GRAPH.road_points[from_road], 
                                parse::MAP_GRAPH.road_points[to_road], 
                                Style::default().bg(Color::Cyan), 
                                DrawLayout::default()
                            )) 
                        } else { 
                            None 
                        }
                    })
                    .collect()
            })
            .collect();
        
        let road_index = parse::MAP_GRAPH.road_edges
            .iter()
            .enumerate()
            .map(|(from_road, edges)| {
                edges
                    .iter()
                    .enumerate()
                    .filter_map(move |(index, &to_road)| if from_road < to_road { Some((from_road, index)) } else { None })
            })
            .flatten()
            .collect();
        
        let mut map = Map { 
            tiles, 
            ports,
            roads,
            road_index,
            robber,
            bkg: &parse::MAP_BKG_SHAPE,
            layout: DrawLayout::default(), 
            mount: Mount::default() 
        };

        map.layout.set_size(UDim2::from_size2d(parse::MAP_BKG_DRAW_STRING.size));
        for (i, tile) in map.tiles.iter_mut().enumerate() {
            tile.layout
                .set_position(UDim2::from_point2d(parse::MAP_GRAPH.tile_points[i]))
                .set_anchor(Scale2D::new(0.0, 0.5));
        }

        for (port, &port_point) in map.ports.iter_mut().zip(parse::MAP_GRAPH.port_points.iter()) {
            port.layout.set_position(UDim2::from_point2d(port_point));
        }

        map
    }

    pub fn move_robber(&mut self, tile_index: usize, anim_service: &mut ScreenAnimationService) {
        let mut to = self.robber.layout.space;
        to.position = UDim2::from_point2d(parse::MAP_GRAPH.tile_points[tile_index] + ROBBER_OFFSET);
        if self.robber.layout.space == to {
            return
        }
        log::info!("animating robber!");
        self.robber.animate_space(anim_service, to, 1.0, EasingStyle::Cubic, EasingDirection::InOut);
    }
}

impl Layoutable for Map {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl Drawable for Map {
    fn draw(&self, mut area: ScreenArea) {
        area.draw_child(self.bkg);
        let mut itr = area.iter_cells_mut();
        while let Some(cell) = itr.next() {
            if cell.symbol == " " {
                continue
            }

            let cell_bkg = match cell.symbol.as_str() {
                "*" => port::PORT_BOARDWALK_COLOR,
                "O" => port::PORT_BOARDWALK_COLOR,
                 _  => MAP_SAND_COLOR
            };

            cell.set_symbol(" ").set_bg(cell_bkg);
        }
        area.draw_children(&self.tiles);
        area.draw_children(&self.ports);

        for road in self.road_index.iter().cloned().map(|(idx0, idx1)| self.roads[idx0][idx1].as_ref().unwrap()) {
            area.draw_child(road);
        }

        area.draw_child(&self.robber);
    }
}

impl StatefulDrawable for Map {
    type State = NoDrawState;
    fn stateful_draw(&self, area: ScreenArea, _: &Self::State) {
        self.draw(area);
    }
}

impl MountableLayout for Map {
    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }
    fn child_ref(&self, i: usize) -> Option<&dyn MountableLayout> {
        match i {
            0 => Some(self.robber.as_trait_ref()),
            _ => {
                let i = i - 1;
                if i >= self.road_index.len() {
                    None
                } else {
                    let (idx0, idx1) = self.road_index[i];
                    Some(self.roads[idx0][idx1].as_ref().unwrap())
                }
            }
        }
    }
    fn child_mut(&mut self, i: usize) -> Option<&mut dyn MountableLayout> {
        match i {
            0 => Some(self.robber.as_trait_mut()),
            _ => {
                let i = i - 1;
                if i >= self.road_index.len() {
                    None
                } else {
                    let (idx0, idx1) = self.road_index[i];
                    Some(self.roads[idx0][idx1].as_mut().unwrap())
                }
            }
        }
    }
}