use crate::render;

// for now this just starts the render loop
pub fn start(enable_logger: bool) -> Result<(), std::io::Error> {
    // TODO: fill in map struct here
    render::run(enable_logger, None)
}