/*
 * anim.rs
 * module of constructs used in animation
 */

use super::{
    space::{Space, Lerp}, 
    screen::ScreenAnimationService
};

use std::time::Instant;

/* animatable structs implement this trait to define a target type to advance using animation state */
pub trait Animatable {
    type Target: ?Sized;
    fn step(&mut self, target: &mut Self::Target, service: &mut ScreenAnimationService);
    fn cancel(&mut self, service: &mut ScreenAnimationService);
}

/* SpaceAnimation is what is used to animate layout spaces */
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

pub struct TweenInfo {
    pub duration: f32,
    pub style: EasingStyle,
    pub direction: EasingDirection,
    pub delay: f32
}

impl Default for TweenInfo {
    fn default() -> Self {
        TweenInfo { 
            duration: 1.0, 
            style: EasingStyle::Linear, 
            direction: EasingDirection::Out, 
            delay: 0.0 
        }
    }
}

impl SpaceAnimation {
    pub fn new(service: &mut ScreenAnimationService, space0: Space, space1: Space, duration: f32, style: EasingStyle, direction: EasingDirection) -> Self {
        service.add();
        SpaceAnimation { space0, space1, duration, style, direction, start: Instant::now(), stopped: false }
    }
}

impl Animatable for SpaceAnimation {
    type Target = Space;

    /* update animation state and target space accordingly */
    fn step(&mut self, target: &mut Self::Target, service: &mut ScreenAnimationService) {
        if self.stopped {
            return
        }

        /* get animation progress as a [0, 1] scalar */
        let alpha = (self.start.elapsed().as_secs_f32()/self.duration).min(1.0);

        /* stop animation logic if we have reached completion */
        if alpha == 1.0 {
            self.stopped = true;
            service.sub();
        }

        /* update target acoordingly (this still runs when alpha == 1 to put target at goal space) */
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

/* the style of easing to use then transitioning from alpha: 0 to alpha: 1 */
#[derive(Copy, Clone, Debug)]
pub enum EasingStyle {
    Linear,
    Cubic
}

/* the "direction of slow down" of the ease operation */
#[derive(Copy, Clone, Debug)]
pub enum EasingDirection {
    In, // this means the alpha will start with no speed and speed up
    Out, // this means the alpha will start with speed and slow down
    InOut // this means the alpha will start with no speed and speed up but then slow down again
}

/* 
 * takes an alpha assumed to be linear and maps it to an alpha of the given easing style and direction
 * read more here: https://easings.net/
 */
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