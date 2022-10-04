use super::{Tile, TileDigitsAnimation, Port, parse, placement::*, TILE_SIZE};

use crate::render::{prelude::*, iter::CustomIterator};
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
    tile_digit_anims: Vec<(usize, TileDigitsAnimation)>,
    ports: Vec<Port>,
    roads: Vec<Vec<Option<Road>>>,
    road_index: Vec<(usize, usize)>,
    buildings: Vec<Building>,
    robber: DrawLeaf<Shape128>,
    layout: DrawLayout,
    mount: Mount
}

impl Map {
    pub fn new(tiles: Vec<Tile>, ports: Vec<Port>) -> Self {
        let robber_init_tile_position = parse::MAP_GRAPH.tile_anchor_points
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
                .set_anchor(Float2D::new(0.5, 1.0))
                .set_position(UDim2::from_point2d(robber_init_tile_position + ROBBER_OFFSET))
                .set_visible(false)
                .clone()
        );

        let roads = parse::MAP_GRAPH.plot_edges
            .iter()
            .enumerate()
            .map(|(from_plot, edges)| {
                edges
                    .iter()
                    .map(|&to_plot| {
                        if from_plot < to_plot { 
                            Some(Road::new(
                                parse::MAP_GRAPH.plot_points[from_plot], 
                                parse::MAP_GRAPH.plot_points[to_plot], 
                                Style::default().bg(Color::Cyan), 
                                DrawLayout::default().set_visible(false).clone()
                            )) 
                        } else { 
                            None 
                        }
                    })
                    .collect()
            })
            .collect();
        
        let road_index = parse::MAP_GRAPH.plot_edges
            .iter()
            .enumerate()
            .map(|(from_plot, edges)| {
                edges
                    .iter()
                    .enumerate()
                    .filter_map(move |(index, &to_plot)| if from_plot < to_plot { Some((from_plot, index)) } else { None })
            })
            .flatten()
            .collect();
        
        let buildings = parse::MAP_GRAPH.plot_points
            .iter()
            .map(|&plot_point| Building::new(
                enums::Building::Settlement, 
                Style::default(), 
                DrawLayout::default()
                    .set_position(UDim2::from_point2d(plot_point))
                    .set_anchor(Float2D::new(0.5, 1.0))
                    .set_visible(false)
                    .clone()
            ))
            .collect();
        
        let mut map = Map { 
            tiles,
            tile_digit_anims: Vec::new(),
            ports,
            roads,
            road_index,
            buildings,
            robber,
            bkg: &parse::MAP_BKG_SHAPE,
            layout: DrawLayout::default(), 
            mount: Mount::default() 
        };

        map.layout.set_size(UDim2::from_size2d(parse::MAP_BKG_DRAW_STRING.size));
        for (i, tile) in map.tiles.iter_mut().enumerate() {
            tile.layout
                .set_position(UDim2::from_point2d(parse::MAP_GRAPH.tile_anchor_points[i]))
                .set_anchor(Float2D::new(0.0, 0.5));
        }

        map
    }

    #[allow(dead_code)]
    pub fn move_robber(&mut self, tile_index: usize, anim_service: &mut AnimationService) {
        if !self.robber.layout.is_visible {
            return
        }

        let mut to = self.robber.layout.space;
        to.position = UDim2::from_point2d(parse::MAP_GRAPH.tile_anchor_points[tile_index] + ROBBER_OFFSET);
        if self.robber.layout.space == to {
            return
        }
        self.robber.animate_space(anim_service, to, 1.0, EasingStyle::Cubic, EasingDirection::InOut);
    }

    pub fn show_port(&mut self, port: usize, anim_service: &mut AnimationService) {
        self.ports[port].animate(anim_service);
    }

    pub fn place_road(&mut self, plot_a: usize, plot_b: usize, style: Style, anim_service: &mut AnimationService) {
        let idx0 = plot_a.min(plot_b);
        let idx1 = parse::MAP_GRAPH.plot_edges[idx0].iter().position(|&plot| plot == plot_a.max(plot_b)).unwrap();
        self.roads[idx0][idx1].as_mut().unwrap().build(style, anim_service);
    }

    pub fn place_tile(&mut self, tile_index: usize, anim_service: &mut AnimationService) {
        let tile = &mut self.tiles[tile_index];
        let mut anim = TileDigitsAnimation::new(tile);
        anim.play(anim_service);
        tile.play(anim_service);
        
        if tile.resource == enums::TileResource::OfDesert {
            let (start, duration) = tile.get_map_fall_parameters(Point2D::new(0, TILE_SIZE.y as i16/2) + ROBBER_OFFSET, 0);
            let mut to = self.robber.layout.space;
            to.position = UDim2::from_point2d(parse::MAP_GRAPH.tile_anchor_points[tile_index] + ROBBER_OFFSET);
            self.robber.layout
                .set_visible(true)
                .set_position(UDim2::from_point2d(start));
            self.robber.animate_space(anim_service, to, duration, EasingStyle::Cubic, EasingDirection::Out);
        }

        self.tile_digit_anims.push((tile_index, anim));
    }

    pub fn place_building(&mut self, plot: usize, kind: enums::Building, style: Style, anim_service: &mut AnimationService) {
        if self.buildings[plot].kind != kind {
            let mut mount = *self.buildings[plot].mount_ref(); // manual remounting
            mount.children = 0;
            let mut building = Building::new(
                kind, 
                Style::default(), 
                DrawLayout::default()
                    .set_position(UDim2::from_point2d(parse::MAP_GRAPH.plot_points[plot]))
                    .set_anchor(Float2D::new(0.5, 1.0))
                    .set_visible(false)
                    .clone()
            );
            building.mount(mount);
            self.buildings[plot] = building;
        }

        self.buildings[plot].build(style, anim_service);
    }
}

impl Layoutable for Map {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl Drawable for Map {
    fn draw(&self, ctx: &mut DrawContext) {
        ctx.draw_child(self.bkg);
        let mut itr = ctx.iter_cells_mut();
        while let Some(cell) = itr.next() {
            if let " " = cell.symbol.as_str() {
                continue
            }

            if let "*" | "?" | "X" = cell.symbol.as_str() {
                cell.set_symbol(" ");
            } else {
                cell.set_symbol(" ").set_bg(MAP_SAND_COLOR);
            }            
        }

        ctx.draw_children(&self.tiles);
        ctx.draw_children(&self.ports);

        for road in self.road_index.iter().map(|(idx0, idx1)| self.roads[*idx0][*idx1].as_ref().unwrap()) {
            ctx.draw_child(road);
        }

        ctx.draw_children(&self.buildings);

        ctx.draw_child(&self.robber);

        for (_, anim) in self.tile_digit_anims.iter() {
            if let Some(animator) = anim.animator.digit0.as_ref() {
                ctx.draw_child(&animator.digit);
            }
            if let Some(animator) = anim.animator.digit1.as_ref() {
                ctx.draw_child(&animator.digit);
            }
        }
    }
}

impl StatefulDrawable for Map {
    type State = ();
    fn stateful_draw(&self, ctx: &mut DrawContext, _: &Self::State) {
        self.draw(ctx);
    }
}

// TODO: this is ridiculous... please find a better approach LMFAO
impl MountableLayout for Map {
    fn relayout(&mut self, ctx: &mut LayoutContext) {
        for (tile, anim) in self.tile_digit_anims.iter_mut() {
            anim.update(&mut self.tiles[*tile]);
        }
        self.tile_digit_anims.retain(|(_, anim)| anim.state.playback == PlaybackState::Playing);
        ctx.relayout_children_of(self);
    }

    fn mount_ref(&self) -> &Mount { &self.mount }
    fn mount_mut(&mut self) -> &mut Mount { &mut self.mount }
    fn child_ref(&self, mut i: usize) -> Option<&dyn MountableLayout> {
        match i {
            0 => Some(self.robber.as_trait_ref()),
            _ => {
                i -= 1;
                if i >= self.road_index.len() {
                    i -= self.road_index.len();
                    if i >= self.buildings.len() {
                        i -= self.buildings.len();
                        if i >= self.tiles.len() {
                            i -= self.tiles.len();
                            if i >= self.ports.len() {
                                None
                            } else {
                                Some(self.ports[i].as_trait_ref())
                            }
                        } else {
                            Some(self.tiles[i].as_trait_ref())
                        }
                    } else {
                        Some(self.buildings[i].as_trait_ref())
                    }
                } else {
                    let (idx0, idx1) = self.road_index[i];
                    Some(self.roads[idx0][idx1].as_ref().unwrap())
                }
            }
        }
    }
    fn child_mut(&mut self, mut i: usize) -> Option<&mut dyn MountableLayout> {
        match i {
            0 => Some(self.robber.as_trait_mut()),
            _ => {
                i -= 1;
                if i >= self.road_index.len() {
                    i -= self.road_index.len();
                    if i >= self.buildings.len() {
                        i -= self.buildings.len();
                        if i >= self.tiles.len() {
                            i -= self.tiles.len();
                            if i >= self.ports.len() {
                                None
                            } else {
                                Some(self.ports[i].as_trait_mut())
                            }
                        } else {
                            Some(self.tiles[i].as_trait_mut())
                        }
                    } else {
                        Some(self.buildings[i].as_trait_mut())
                    }
                } else {
                    let (idx0, idx1) = self.road_index[i];
                    Some(self.roads[idx0][idx1].as_mut().unwrap())
                }
            }
        }
    }
}