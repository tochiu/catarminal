pub mod space;
pub mod draw;
pub mod drawing;
pub mod screen;
pub mod shape;

mod run;
pub use run::run;

mod mount;
mod iter;
mod anim;
mod input;

pub mod prelude {
    pub use super::draw::*;
    pub use super::space::*;
    pub use super::screen::*;
    pub use super::mount::*;
    pub use super::anim::*;
    pub use super::shape::*;
    pub use super::input::*;
}