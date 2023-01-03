/*
 * input.rs
 * module of constructs used in capturing input
 */


use super::{space::*, mount::*};

use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};

/* struct that stores the id of the mount that requested input capture at the defined space */
#[derive(Debug, PartialEq, Eq)]
pub struct Input {
    id: MountId,
    space: AbsoluteSpace
}

/* this struct is what stores all requested input capture spaces and calculates and handles input events */
#[derive(Debug, Default)]
pub struct InputService {
    inputs: Vec<Input>,
    capturing_mount_id: Option<MountId>,
    did_mouse_move: bool,
    input_event_queue: Vec<InputEvent>
}

/* this struct details what mount triggered what input and is what is passed onto the input_event_queue */
#[derive(Debug)]
pub struct InputEvent {
    mount_id: MountId,
    pub kind: InputEventKind
}

/* the kind of input events that can occur with specific input information wrapped inside */
#[derive(Debug)]
pub enum InputEventKind {
    Click(Point2D),
    Drag(Point2D),
    Move(Point2D),
    Down(Point2D),
    Up(Point2D)
}

impl InputService {
    /* 
     * invalidating inputs is necessary because drawings can change or be removed between render calls 
     * so drawings must redefine their input space between each time to keep it alive
     */
    pub fn invalidate_all_inputs(&mut self) {
        self.inputs.clear();
    }

    pub fn capture_space(&mut self, id: MountId, space: AbsoluteSpace) {
        self.inputs.push(Input { id, space });
    }

    /* 
     * get the id of the top-most id that is capturing input at the given point 
     * NOTE: inputs in the vec are considered above the ones before them
     */
    fn query(&self, point: Point2D) -> Option<MountId> {
        for input in self.inputs.iter().rev() {
            let id = input.id;
            let space = input.space;
            if space.is_interior_point(point) {
                return Some(id);
            }
        }

        None
    }

    /*
     * handles a crossterm mouse event, returns a boolean indicating if the screen needs to be rendered again
     */
    pub fn handle_mouse_input(&mut self, event: MouseEvent, root: &mut dyn MountableLayout) -> bool {
        /* get the point the mouse event occured... */
        let point = Point2D::new(
            i16::try_from(event.column).unwrap(), 
            i16::try_from(event.row).unwrap()
        );

        /* ...with a query of the id of the mount that would capture it */
        let maybe_id = self.query(point);

        /* how we handle the event depends on our internal state and the event kind */
        match event.kind {
            MouseEventKind::Down(button) => {
                /* do nothing if its middle mouse or the capturing mount is the same */
                if button == MouseButton::Middle || maybe_id == self.capturing_mount_id {
                    return false
                }

                /* if there is a previously capturing mount, queue a mouse up event for it */
                if let Some(old_id) = self.capturing_mount_id {
                    self.input_event_queue.push(InputEvent { 
                        mount_id: old_id,
                        kind: InputEventKind::Up(point)
                    });
                }

                /* if we queried a capturing mount, queue up a mouse down event for it */
                if let Some(id) = maybe_id {
                    self.input_event_queue.push(InputEvent { 
                        mount_id: id, 
                        kind: InputEventKind::Down(point) 
                    });
                }

                self.capturing_mount_id = maybe_id;
                self.did_mouse_move = false; // mouse hasnt moved yet so reset
            },
            MouseEventKind::Drag(button) => {
                /* do nothing if its middle mouse or we dont have a currently capturing mount */
                if button == MouseButton::Middle || self.capturing_mount_id.is_none() {
                    return false
                }

                self.did_mouse_move = true;

                /* push drag event for currently capturing mount */
                self.input_event_queue.push(InputEvent { 
                    mount_id: self.capturing_mount_id.unwrap(), 
                    kind: InputEventKind::Drag(point)
                });
            },
            /* mouse move is mouse drag but without the held down click */
            MouseEventKind::Moved => {
                /* if there is a currently capturing mount then queue up a drag instead */
                if let Some(capturing_mount_id) = self.capturing_mount_id {
                    self.did_mouse_move = true;
                    self.input_event_queue.push(InputEvent {
                        mount_id: capturing_mount_id, 
                        kind: InputEventKind::Drag(point)
                    });
                /* otherwise if we queried a capturing mount, queue up a mouse move event for it */
                } else if let Some(id) = maybe_id {
                    self.input_event_queue.push(InputEvent { 
                        mount_id: id, 
                        kind: InputEventKind::Move(point)
                    });
                }
            }
            MouseEventKind::Up(button) => {
                /* do nothing if its middle mouse or the capturing mount is the same */
                if button == MouseButton::Middle || self.capturing_mount_id.is_none() {
                    return false
                }
                /* queue up a mouse up event for the currently capturing mount */
                self.input_event_queue.push(InputEvent { 
                    mount_id: self.capturing_mount_id.unwrap(), 
                    kind: InputEventKind::Up(point)
                });
                /* if the mouse didnt move at all, we know it was a click so queue that up */
                if self.capturing_mount_id == self.query(point) {
                    self.input_event_queue.push(InputEvent { 
                        mount_id: self.capturing_mount_id.unwrap(), 
                        kind: InputEventKind::Click(point)
                    });
                }

                /* up means we are no longer the capturing mount */
                self.capturing_mount_id = None;
            }
            _ => () // otherwise do nothing
        }

        let mut should_rerender = false;

        /* 
         * for every input event in the queue we want to find the mount with the given id and have it run its input handler 
         * if it returns true then the screen needs to be rendered again
         */
        for input_event in self.input_event_queue.drain(..) {
            if let Some(descendant) = root.find_descendant_mut(MountFinder::new(input_event.mount_id)) {
                // call on separate line because we dont want short-circuiting to prevent descendant mouse input handler from running
                let descendant_requires_rerender = descendant.on_mouse_input(input_event);
                should_rerender = should_rerender || descendant_requires_rerender;
            }
        }

        should_rerender
    }
}