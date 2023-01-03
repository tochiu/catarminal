/*
 * mount.rs
 * a module of constructs that define a mountable layout
 * a mountable layout is a layout that can be found by its screen
 * this property allows them to communicate with the screen to capture input or trigger a rerender
 * mountable layouts are the only layouts that execute a relayout because relayouts can mutate screen state
 */
use super::prelude::*;
use super::iter::*;

/* mount identifier */
pub type MountId = u64;

/* a mount defines its id and number of children (this is required to create child mounts) */
#[derive(Copy, Clone, Default, Debug)]
pub struct Mount {
    pub id: MountId,
    pub children: u8
}

impl Mount {
    /* create the mount for a child mountable */
    fn fork(&mut self) -> Mount {
        self.children += 1;
        Mount { 
            id: MountFinder::new(self.id).push(self.children).id, 
            children: 0
        }
    }
}

/* 
 * mount ids encode how to get to them from the root drawing of the screen 
 * by storing the index of the drawing to query at each level in a byte 
 * since MountId is a typdef'd u64, a maximum of 8 ancestors is allowed for a mount
 * 
 * MountFinder is a utility struct used with methods useful for finding mounts using their ids
 */
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

    /* creating a mountfinder from a mountid and pushing an index into the id is the way child mount ids are made (seek Mount::fork) */
    pub fn push(self, index: u8) -> MountFinder {
        MountFinder { id: self.id | (index as MountId) << u8::BITS*self.depth() }
    }
    pub fn depth(&self) -> u32 {
        MountFinder::MAX_DEPTH - self.id.leading_zeros() / u8::BITS
    }
}

/* MountableLayout's must be static because the CustomIterator trait the child iterator implement require that the item type is static */
pub trait MountableLayout: Layoutable + std::fmt::Debug + AsMountableLayout + 'static {

    // implement

    fn mount_ref(&self) -> &Mount;
    fn mount_mut(&mut self) -> &mut Mount;
    fn child_ref(&self, i: usize) -> Option<&dyn MountableLayout>;
    fn child_mut(&mut self, i: usize) -> Option<&mut dyn MountableLayout>;

    // event handlers

    #[allow(unused_variables)]
    fn on_mouse_input(&mut self, event: InputEvent) -> bool { false }
    
    // required / utility

    /* animate the layout of the mountable */
    fn animate_space_from(&mut self, anim_service: &mut AnimationService, from: Space, to: Space, duration: f32, style: EasingStyle, direction: EasingDirection) {
        let mut layout = self.layout_mut();
        if let Some(anim) = layout.anim.as_mut() {
            anim.cancel(anim_service);
        }

        let mut anim = Box::new(SpaceAnimation::new(from, to, duration, style, direction));
        anim.play(anim_service);
        
        layout.anim = Some(anim);
    }

    fn animate_space(&mut self, anim_service: &mut AnimationService, to: Space, duration: f32, style: EasingStyle, direction: EasingDirection) {
        self.animate_space_from(anim_service, self.layout_ref().space, to, duration, style, direction)
    }

    /* set the mount of the mountable and mount its descendants */
    fn mount(&mut self, mut mount: Mount) {
        let mut itr = self.child_iter_mut();
        while let Some(child) = itr.next() {
            child.mount(mount.fork());
        }

        *self.mount_mut() = mount;
    }

    /* get iterator for mounts children */
    fn child_iter(&self) -> MountChildIter {
        MountChildIter { 
            mountable: self.as_trait_ref(), 
            index: 0
        }
    }

    /* same as child_iter but mutable */
    fn child_iter_mut(&mut self) -> MountChildIterMut {
        MountChildIterMut { 
            mountable: self.as_trait_mut(),
            index: 0
        }
    }

    /* get a reference to a descendant using the given MountFinder */
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

    /* same as find_descendant_mut but mutable */
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

    fn relayout(&mut self, ctx: &mut LayoutContext) {
        ctx.relayout_children_of(self.as_trait_mut());
    }
}

/* this trait exists because concrete types implementing MountableLayout need to be used in methods that require the trait object */
pub trait AsMountableLayout {
    fn as_trait_ref(&self) -> &dyn MountableLayout;
    fn as_trait_mut(&mut self) -> &mut dyn MountableLayout;
}

impl<T: MountableLayout + Sized> AsMountableLayout for T {
    fn as_trait_ref(&self) -> &dyn MountableLayout { self }
    fn as_trait_mut(&mut self) -> &mut dyn MountableLayout { self }
}

/* iterators for iterating over the children of the given mountable mutably or immutably */

pub struct MountChildIter<'a> {
    mountable: &'a dyn MountableLayout,
    index: usize
}

impl<'a> CustomIterator for MountChildIter<'a> {
    type Item = RefFamily<dyn MountableLayout>;

    fn next<'s>(&'s mut self) -> Option<<Self::Item as FamilyLt<'s>>::Out> {
        let maybe_child = self.mountable.child_ref(self.index);
        self.index += 1;
        maybe_child
    }
}

pub struct MountChildIterMut<'a> {
    mountable: &'a mut dyn MountableLayout,
    index: usize
}

impl<'a> CustomIterator for MountChildIterMut<'a> {
    type Item = MutRefFamily<dyn MountableLayout>;

    fn next<'s>(&'s mut self) -> Option<<Self::Item as FamilyLt<'s>>::Out> {
        let maybe_child = self.mountable.child_mut(self.index);
        self.index += 1;
        maybe_child
    }
}