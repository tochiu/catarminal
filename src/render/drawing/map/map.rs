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
        iter::CustomIterator
    }
};

use tui::style::Color;

pub const MAP_SAND_COLOR: Color = Color::Rgb(221, 178, 100);
pub const MAP_OCEAN_COLOR: Color = Color::Rgb(9, 103, 166);

#[derive(Debug)]
pub struct Map {
    bkg: &'static StringShape<'static>,
    tiles: Vec<Tile>,
    ports: Vec<Port>,
    layout: DrawLayout,
    mount: Mount
}

impl Map {
    pub fn new(tiles: Vec<Tile>, ports: Vec<Port>) -> Self {
        let mut map = Map { tiles, ports, bkg: &parse::MAP_BKG_SHAPE, layout: DrawLayout::default(), mount: Mount::default() };

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
    fn child_ref(&self, _: usize) -> Option<&dyn MountableLayout> { None }
    fn child_mut(&mut self, _: usize) -> Option<&mut dyn MountableLayout> { None }
}