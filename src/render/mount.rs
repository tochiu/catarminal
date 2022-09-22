use super::{
    draw::*, 
    screen::*, 
    space::*,
    iter::*, 
    anim::*
};

pub type MountId = u64;

#[derive(Copy, Clone, Debug)]
pub struct MountFinder {
    id: MountId
}

impl MountFinder {
    const MAX_DEPTH: u32 = MountId::BITS / u8::BITS;

    pub fn new(id: MountId) -> Self {
        MountFinder { id }
    }
    pub fn peek(&self) -> u8 {
        (self.id & u8::MAX as MountId) as u8
    }
    pub fn next(self) -> MountFinder {
        MountFinder { id: self.id >> u8::BITS }
    }
    pub fn push(self, index: u8) -> MountFinder {
        MountFinder { id: self.id | (index as MountId) << u8::BITS*self.depth() }
    }
    pub fn depth(&self) -> u32 {
        MountFinder::MAX_DEPTH - self.id.leading_zeros() / u8::BITS
    }
}

pub trait MountableLayout: Layoutable + std::fmt::Debug + AsTrait + 'static {

    // implement

    fn mount_ref(&self) -> &Mount;
    fn mount_mut(&mut self) -> &mut Mount;
    fn child_ref(&self, i: usize) -> Option<&dyn MountableLayout>;
    fn child_mut(&mut self, i: usize) -> Option<&mut dyn MountableLayout>;

    // event handlers

    #[allow(unused_variables)]
    fn on_mouse_input(&mut self, event: ScreenInputEvent) -> bool { false }
    
    // required / utility

    fn animate_space(&mut self, service: &mut ScreenAnimationService, to: Space, duration: f32, style: EasingStyle, direction: EasingDirection) {
        let mut layout = self.layout_mut();
        if let Some(anim) = layout.anim.as_mut() {
            anim.cancel(service);
        }
        
        layout.anim = Some(Box::new(SpaceAnimation::new(service, layout.space, to, duration, style, direction)));
    }

    fn mount(&mut self, mut mount: Mount) {
        let mut itr = self.child_iter_mut();
        while let Some(child) = itr.next() {
            child.mount(mount.fork());
        }

        *self.mount_mut() = mount;
    }

    fn child_iter(&self) -> MountChildIter {
        MountChildIter { 
            drawing: self.as_trait_ref(), 
            index: 0
        }
    }

    fn child_iter_mut(&mut self) -> MountChildIterMut {
        MountChildIterMut { 
            drawing: self.as_trait_mut(),
            index: 0
        }
    }

    fn find_descendant_ref(&self, finder: MountFinder) -> Option<&dyn MountableLayout> {
        let index = finder.peek();

        if index == 0 {
            Some(self.as_trait_ref())
        } else {
            if let Some(child) = self.child_ref(index as usize - 1) {
                child.find_descendant_ref(finder.next())
            } else {
                None
            }
        }
    }

    fn find_descendant_mut(&mut self, finder: MountFinder) -> Option<&mut dyn MountableLayout> {
        let index = finder.peek();

        if index == 0 {
            Some(self.as_trait_mut())
        } else {
            if let Some(child) = self.child_mut(index as usize - 1) {
                child.find_descendant_mut(finder.next())
            } else {
                None
            }
        }
    }

    fn relayout(&mut self, relayout: &mut ScreenRelayout) {
        relayout.children_of(self.as_trait_mut());
    }
}

pub trait AsTrait {
    fn as_trait_ref(&self) -> &dyn MountableLayout;
    fn as_trait_mut(&mut self) -> &mut dyn MountableLayout;
}

impl<T: MountableLayout + Sized> AsTrait for T {
    fn as_trait_ref(&self) -> &dyn MountableLayout { self }
    fn as_trait_mut(&mut self) -> &mut dyn MountableLayout { self }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Mount {
    pub id: MountId,
    pub children: u8
}

impl Mount {
    fn fork(&mut self) -> Mount {
        self.children += 1;
        Mount { 
            id: MountFinder::new(self.id).push(self.children).id, 
            children: 0
        }
    }
}

pub struct MountChildIter<'a> {
    drawing: &'a dyn MountableLayout,
    index: usize
}

impl<'a> CustomIterator for MountChildIter<'a> {
    type Item = RefFamily<dyn MountableLayout>;

    fn next<'s>(&'s mut self) -> Option<<Self::Item as FamilyLt<'s>>::Out> {
        let maybe_child = self.drawing.child_ref(self.index);
        self.index += 1;
        maybe_child
    }
}

pub struct MountChildIterMut<'a> {
    drawing: &'a mut dyn MountableLayout,
    index: usize
}

impl<'a> CustomIterator for MountChildIterMut<'a> {
    type Item = MutRefFamily<dyn MountableLayout>;

    fn next<'s>(&'s mut self) -> Option<<Self::Item as FamilyLt<'s>>::Out> {
        let maybe_child = self.drawing.child_mut(self.index);
        self.index += 1;
        maybe_child
    }
}