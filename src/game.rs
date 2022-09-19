use crate::render;

pub fn start(enable_logger: bool) -> Result<(), std::io::Error> {
    render::run(enable_logger)
}