use super::{
    parse,
    tile::Tile,
    port::{self, Port},
    super::super::{
        space::*,
        draw::*,
        screen::*,
        mount::*,
        shape::*,
        anim::*,
        iter::CustomIterator
    }
};

use crate::enums;

use tui::style::{Color, Style};

pub const MAP_SAND_COLOR: Color = Color::Rgb(221, 178, 100);
pub const MAP_OCEAN_COLOR: Color = Color::Rgb(9, 103, 166);

const ROBBER_OFFSET: Point2D = Point2D::new(9, -4); // robber offset from tile offset

lazy_static! {
    static ref ROBBER_BITSHAPE: BitShape128 = BitShape128::new(0b011101111101110111111111111111, Size2D::new(5, 6));
    static ref ROBBER_STYLE: Style = Style::default().bg(Color::Magenta);
}

#[derive(Debug)]
pub struct Map {
    bkg: &'static StringShape<'static>,
    tiles: Vec<Tile>,
    ports: Vec<Port>,
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
        
        let mut map = Map { 
            tiles, 
            ports, 
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
            _ => None
        }
    }
    fn child_mut(&mut self, i: usize) -> Option<&mut dyn MountableLayout> {
        match i {
            0 => Some(self.robber.as_trait_mut()),
            _ => None
        }
    }
}