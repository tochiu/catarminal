use super::{
    PlayerFrame,
    PlayerFrameState,
    PLAYER_FRAME_SIZE,
    super::super::{
        draw::*,
        space::*,
        screen::*
    }
};

#[derive(Debug)]
pub struct PlayerList {
    pub frames: Vec<PlayerFrame>,
    pub layout: DrawLayout
}

impl PlayerList {
    pub fn new(mut frames: Vec<PlayerFrame>, layout: DrawLayout) -> Self {
        for (i, frame) in frames.iter_mut().enumerate() {
            frame.layout
                .set_position(UDim2::new(0.5, 0, 1.0, -(i as i16)*(PLAYER_FRAME_SIZE.y.offset as i16)))
                .set_anchor(Scale2D::new(0.5, 1.0));
        }
        PlayerList { frames, layout }
    }
}

impl Layoutable for PlayerList {
    fn layout_ref(&self) -> &DrawLayout { &self.layout }
    fn layout_mut(&mut self) -> &mut DrawLayout { &mut self.layout }
}

impl StatefulDrawable for PlayerList {
    type State = [PlayerFrameState];

    fn stateful_draw(&self, mut area: ScreenArea, state: &Self::State) {
        area.draw_stateful_children(self.frames.as_slice(), state);
    }
}