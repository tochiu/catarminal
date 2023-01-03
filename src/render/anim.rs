/*
 * anim.rs
 * module of constructs used in animation
 */

use std::{num::NonZeroU64, collections::HashMap, time::Instant};

/* animatable structs implement this trait to define a target type to advance using animation state */
pub trait Animator {

    type Target: ?Sized;

    fn update(&mut self, state: &AnimationState, target: &mut Self::Target);
    fn play(&mut self) {}
    fn cancel(&mut self) {}
}

impl Animator for () {
    type Target = ();
    fn update(&mut self, _: &AnimationState, _: &mut Self::Target) {}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum PlaybackState {
    #[default]
    Init,
    Playing,
    Completed,
    Cancelled
}

/* the style of easing to use then transitioning from alpha: 0 to alpha: 1 */
#[derive(Copy, Clone, Debug)]
pub enum EasingStyle {
    #[allow(dead_code)]
    Linear,
    Cubic
}

/* the "direction of slow down" of the ease operation */
#[derive(Copy, Clone, Debug)]
pub enum EasingDirection {
    #[allow(dead_code)]
    In,   // this means the alpha will start with no speed and speed up
    Out,  // this means the alpha will start with speed and slow down
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

pub type AnimationId = NonZeroU64; /* apparently NonZeroU64 has optimized memory layout with Options (sizeof Option(NonZeroU64) == sizeof u64) */

/* 
 * struct that keeps a count of animations running inside the screen
 * animations are owned by their drawings but must make themselves known to the screen as a requirement for some rendering optimizations
 * its not the most "sophisticated" but it is good enough
 */
#[derive(Debug)]
pub struct AnimationService {
    available_id: AnimationId,
    animations: HashMap<AnimationId, AnimationState> /* only playing or expired screen animations are in this HashMap */
}

impl Default for AnimationService {
    fn default() -> Self {
        AnimationService { 
            available_id: AnimationId::new(1).unwrap(), 
            animations: Default::default() 
        }
    }
}

impl AnimationService {
    pub fn cancel(&mut self, state: &mut AnimationState) {
        if state.is_cancellable() {
            state.playback = PlaybackState::Cancelled;
            if let Some(id) = state.id {
                self.animations.remove(&id);
            }
        }
    }

    pub fn create(&mut self, state: &mut AnimationState) {
        let id = self.available_id;
        self.available_id = AnimationId::new(self.available_id.get() + 1).unwrap();
        state.id = Some(id);
        state.start = Instant::now();
        state.playback = PlaybackState::Playing;
        self.animations.insert(id, AnimationState { 
            id: Some(id), 
            start: state.start, 
            duration: state.duration, 
            playback: PlaybackState::Playing
        });
    }

    pub fn count(&self) -> usize {
        self.animations.len()
    }

    pub fn cleanup(&mut self) {
        self.animations.retain(|_, anim| anim.get_alpha() < 1.0);
    }
}

#[derive(Debug)]
pub struct Animation<T: Animator + std::fmt::Debug> {
    pub state: AnimationState,
    pub animator: T
}

impl<T: Animator + std::fmt::Debug> Animation<T> {

    pub fn with_duration(duration: f32, animator: T) -> Self {
        Animation { state: AnimationState::with_duration(duration), animator }
    }

    pub fn play(&mut self, anim_service: &mut AnimationService) {
        anim_service.create(&mut self.state);
        self.animator.play();
    }

    pub fn update(&mut self, target: &mut T::Target) {
        if self.state.playback != PlaybackState::Playing {
            return
        }

        /* 
         * one issue is alpha here is < 1, anim runs where alpha == 1 and so will trigger any custom alpha == 1 logic twice 
         * since we check completion using the alpha before the update call
         */
        let alpha = self.state.get_alpha();
        self.animator.update(&self.state, target);

        if alpha == 1.0 {
            self.state.playback = PlaybackState::Completed;
        }
    }

    pub fn cancel(&mut self, anim_service: &mut AnimationService) {
        anim_service.cancel(&mut self.state);
        self.animator.cancel()
    }
}

/*
 * struct that keeps track of animation state relevant to the screen
 * start and duration are tracked so animations do not need to consult the screen to signal completion
 *      knowing when no animations are playing leads to fewer relayout calls
 * id is necessary to in order for animations to signal a cancel
 */
#[derive(Debug)]
pub struct AnimationState {
    id: Option<AnimationId>,
    pub start: Instant,
    pub duration: f32,
    pub playback: PlaybackState
}

impl AnimationState {
    pub fn with_duration(duration: f32) -> Self {
        AnimationState {
            duration,
            ..Default::default()
        }
    }

    pub fn get_alpha(&self) -> f32 {
        // NOTE: alpha is not remembered on cancel
        match self.playback {
            PlaybackState::Playing => if self.duration != 0.0 { self.get_elapsed()/self.duration } else { 1.0 },
            PlaybackState::Completed => 1.0,
            _ => 0.0
        }
    }

    pub fn get_elapsed(&self) -> f32 {
        self.start.elapsed().as_secs_f32().min(self.duration)
    }

    pub fn is_cancellable(&self) -> bool {
        if let PlaybackState::Init | PlaybackState::Playing = self.playback { true } else { false }
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        AnimationState { id: None, start: Instant::now(), duration: 0.0, playback: Default::default() }
    }
}