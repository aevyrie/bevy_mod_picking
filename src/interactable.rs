use super::*;
use bevy::prelude::*;

pub struct InteractableMesh {
    mouse_down: Vec<(MouseButton, PickIntersection)>,
    mouse_just_pressed: Vec<(MouseButton, PickIntersection)>,
    mouse_just_released: Vec<(MouseButton, PickIntersection)>,
    pub mouse_entered: bool,
    pub mouse_exited: bool,
    pub mouse_hover: bool,
    pick_group: PickGroup,
    button_group: Vec<MouseButton>,
}

impl InteractableMesh {
    pub fn new(pick_group: PickGroup, button_group: Vec<MouseButton>) -> Self {
        InteractableMesh {
            pick_group,
            button_group,
            ..Default::default()
        }
    }

    pub fn mouse_down(&self, button: MouseButton) -> Option<&PickIntersection> {
        //Filter for any values where the first tuple matches the mouse button
        let mut filter = self
            .mouse_down
            .iter()
            .filter(|element| -> bool { element.0 == button });

        match filter.next() {
            Some(val) => Some(&val.1),
            None => None,
        }
    }

    pub fn mouse_just_released(&self, button: MouseButton) -> Option<&PickIntersection> {
        //Filter for any values where the first tuple matches the mouse button
        let mut filter = self
            .mouse_just_released
            .iter()
            .filter(|element| -> bool { element.0 == button });

        match filter.next() {
            Some(val) => Some(&val.1),
            None => None,
        }
    }

    pub fn mouse_just_pressed(&self, button: MouseButton) -> Option<&PickIntersection> {
        //Filter for any values where the first tuple matches the mouse button
        let mut filter = self
            .mouse_just_pressed
            .iter()
            .filter(|element| -> bool { element.0 == button });

        match filter.next() {
            Some(val) => Some(&val.1),
            None => None,
        }
    }

    pub fn mouse_entered(&self) -> bool {
        self.mouse_entered
    }

    pub fn mouse_exited(&self) -> bool {
        self.mouse_exited
    }
}

impl Default for InteractableMesh {
    fn default() -> Self {
        InteractableMesh {
            mouse_entered: false,
            mouse_exited: false,
            mouse_hover: false,
            mouse_down: vec![],
            mouse_just_released: vec![],
            mouse_just_pressed: vec![],
            pick_group: PickGroup::default(),
            button_group: vec![MouseButton::Left, MouseButton::Right],
        }
    }
}

// Now the System for Cursor Events make sure this runs before the update stage but after the pickstate / raycasting system runs
pub fn cursor_events(
    pickstate: Res<PickState>,
    mouse_inputs: Res<Input<MouseButton>>,
    mut q_imesh: Query<(&mut InteractableMesh, Entity)>,
) {
    //Go through the pick state and find the
    q_imesh
        .iter_mut()
        .for_each(|mut element| match pickstate.top(element.0.pick_group) {
            Some(v) => process_pick((&mut element.0, element.1), v, &mouse_inputs),

            None => process_inactive_mesh((&mut element.0, element.1)),
        });
}

fn process_pick(
    elem: (&mut InteractableMesh, Entity),
    pick: &PickIntersection,
    mouse_inputs: &Res<Input<MouseButton>>,
) {
    //If entity is the top pick
    let mesh = elem.0;
    let entity = elem.1;

    //Clear Vecs from last frame
    mesh.mouse_just_released.clear();
    mesh.mouse_just_pressed.clear();
    mesh.mouse_down.clear();

    if entity.id() == pick.entity.id() {
        //If it was hovered previously, that means that this is not the first frame the mouse has been over this mesh
        if mesh.mouse_hover {
            mesh.mouse_entered = false;
        } else {
            mesh.mouse_entered = true;
        }

        mesh.mouse_hover = true;
        mesh.mouse_exited = false;

        for button in mesh.button_group.iter() {
            //Map just_released
            if mouse_inputs.just_released(*button) {
                mesh.mouse_just_released.push((*button, *pick));
            }

            //Map just_pressed
            if mouse_inputs.just_pressed(*button) {
                mesh.mouse_just_pressed.push((*button, *pick));
            }

            //Map Pressed
            if mouse_inputs.pressed(*button) {
                mesh.mouse_down.push((*button, *pick));
            }
        }
    } else {
        process_inactive_mesh((mesh, entity));
    }
}

fn process_inactive_mesh(elem: (&mut InteractableMesh, Entity)) {
    if elem.0.mouse_hover {
        elem.0.mouse_hover = false;
        elem.0.mouse_exited = true;
    } else {
        elem.0.mouse_exited = false;
    }

    elem.0.mouse_entered = false;
}
