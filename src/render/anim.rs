
use super::{
    space::{Space, Lerp}, 
    screen::ScreenAnimationService
};

use std::time::Instant;

pub trait Animatable {
    type Target: ?Sized;
    fn step(&mut self, target: &mut Self::Target, service: &mut ScreenAnimationService);
    fn cancel(&mut self, service: &mut ScreenAnimationService);
}

#[derive(Clone, Debug)]
pub struct SpaceAnimation {
    space0: Space,
    space1: Space,
    duration: f32,
    style: EasingStyle,
    direction: EasingDirection,
    start: Instant,
    stopped: bool
}

impl SpaceAnimation {
    pub fn new(service: &mut ScreenAnimationService, space0: Space, space1: Space, duration: f32, style: EasingStyle, direction: EasingDirection) -> Self {
        service.add();
        SpaceAnimation { space0, space1, duration, style, direction, start: Instant::now(), stopped: false }
    }
}

impl Animatable for SpaceAnimation {
    type Target = Space;

    fn step(&mut self, target: &mut Self::Target, service: &mut ScreenAnimationService) {
        if self.stopped {
            return
        }

        let alpha = (self.start.elapsed().as_secs_f32()/self.duration).min(1.0);
        if alpha == 1.0 {
            self.stopped = true;
            service.sub();
        }

        *target = self.space0.lerp(self.space1, ease(alpha, self.style, self.direction));
    }

    fn cancel(&mut self, service: &mut ScreenAnimationService) {
        if self.stopped {
            return
        }

        self.stopped = true;
        service.sub();
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EasingStyle {
    Linear,
    Cubic
}

#[derive(Copy, Clone, Debug)]
pub enum EasingDirection {
    In,
    Out,
    InOut
}

pub fn ease(alpha: f32, style: EasingStyle, direction: EasingDirection) -> f32 {
    match style {
        EasingStyle::Cubic => {
            match direction {
                EasingDirection::In => {
                    alpha.powi(3)
                },
                EasingDirection::Out => {
                    1.0 - (1.0 - alpha).powi(3)
                },
                EasingDirection::InOut => {
                    if alpha < 0.5 { 4.0*alpha.powi(3) } else { 1.0 - (2.0*(1.0 - alpha)).powi(3) / 2.0 }
                }
            }
        },
        _ => alpha
    }
}