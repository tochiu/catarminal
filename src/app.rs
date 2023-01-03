use crate::logic::{Map, Port, Tile};
use crate::render::{self, drawing};

// for now this just starts the render loop
pub fn start(enable_logger: bool) -> Result<(), std::io::Error> {
    // TODO: fill in map struct here
    let map = Map::generate_map();
    let tile_drawings = map
        .tiles
        .iter()
        .map(|tile| drawing::map::Tile::new(tile.roll, tile.resource))
        .collect::<Vec<_>>();
    let port_drawings = map
        .ports
        .iter()
        .map(|port| drawing::map::Port::new(port.id as usize, port.resource))
        .collect::<Vec<_>>();
    render::run(
        enable_logger,
        Some(drawing::map::Map::new(tile_drawings, port_drawings)),
    )
}
