use super::space::*;

use std::collections::HashSet;

pub type InputId = u32;

#[derive(Debug, PartialEq, Eq)]
pub struct Input(InputId, AbsoluteSpace);

pub struct InputSpace {
    input_ids: HashSet<InputId>,
    unused_input_id: InputId,
    inputs: Vec<Input>,
    updated_input_count: usize,
}

impl InputSpace {
    pub fn new() -> Self {
        InputSpace { 
            input_ids: HashSet::new(), 
            unused_input_id: 0, 
            inputs: Vec::new(), 
            updated_input_count: 0 
        }
    }

    pub fn create_input(&mut self) -> InputId {
        let id = self.unused_input_id;
        self.unused_input_id += 1;
        id
    }

    pub fn invalidate_all_inputs(&mut self) {
        self.updated_input_count = 0;
    }

    pub fn clear_invalid_inputs(&mut self) {
        self.inputs.truncate(self.updated_input_count);
    }

    pub fn update_input_space(&mut self, input_id: InputId, space: AbsoluteSpace) {
        let input_index = self.updated_input_count;
        let index_exists = input_index < self.inputs.len();
        let new_input = Input(input_id, space);

        if !(index_exists && self.inputs[input_index] == new_input) {
            if !self.input_ids.contains(&input_id) {
                self.input_ids.insert(input_id);
            }

            if index_exists {
                self.inputs[input_index] = new_input;
            } else {
                self.inputs.push(new_input);
            }
        }

        self.updated_input_count += 1;
    }

    pub fn query(&self, point: Point2D) -> Option<InputId> {
        for Input(input_id, space) in self.inputs.iter().rev() {
            if point.x >= space.left() && point.x < space.right() && point.y >= space.top() && point.y < space.bottom() {
                return Some(*input_id);
            }
        }
        None
    }
}