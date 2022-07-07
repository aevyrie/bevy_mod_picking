use std::fmt::Debug;

use crate::PointerId;
use bevy::{prelude::*, render::camera::RenderTarget};

#[derive(Debug, Default, Clone, Component, PartialEq)]
pub struct PointerClick {
    is_pressed: bool,
}
impl PointerClick {
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }
}

#[derive(Debug, Clone)]
pub enum PointerClickEvent {
    Down { id: PointerId },
    Up { id: PointerId },
}
impl std::fmt::Display for PointerClickEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PointerClickEvent::Down { id } => write!(f, "Event::Click::Down::{:?}", id),
            PointerClickEvent::Up { id } => write!(f, "Event::Click::Up::{:?}", id),
        }
    }
}
impl PointerClickEvent {
    pub fn down(id: PointerId) -> Self {
        Self::Down { id }
    }

    pub fn up(id: PointerId) -> Self {
        Self::Up { id }
    }

    pub fn is_just_down(&self, pointer_id: PointerId) -> bool {
        matches!(self, Self::Down{id} if *id == pointer_id)
    }

    pub fn is_just_up(&self, pointer_id: PointerId) -> bool {
        matches!(self, Self::Up{id} if *id == pointer_id)
    }

    pub fn receive(
        mut events: EventReader<Self>,
        mut pointers: Query<(&PointerId, &mut PointerClick)>,
    ) {
        for event_pointer in events.iter() {
            pointers.for_each_mut(|(pointer_id, mut pointer_click)| match event_pointer {
                Self::Down { id } => {
                    if pointer_id == id {
                        pointer_click.is_pressed = true;
                    }
                }
                Self::Up { id } => {
                    if pointer_id == id {
                        pointer_click.is_pressed = false;
                    }
                }
            })
        }
    }
}

#[derive(Debug, Default, Clone, Component, PartialEq)]
pub struct PointerMultiselect {
    pub is_pressed: bool,
}

#[derive(Debug, Clone, Component, PartialEq)]
pub struct Location {
    pub target: RenderTarget,
    pub position: Vec2,
}
impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pos = self.position;
        write!(
            f,
            "({:6.1} {:6.1}), {}",
            pos.x,
            pos.y,
            match self.target {
                RenderTarget::Window(_) => "Window",
                RenderTarget::Image(_) => "Image",
            }
        )
    }
}
impl Location {
    #[inline]
    pub fn is_in_viewport(&self, camera: &Camera) -> bool {
        camera
            .logical_viewport_rect()
            .map(|(min, max)| {
                (self.position - min).min_element() >= 0.0
                    && (self.position - max).max_element() <= 0.0
            })
            .unwrap_or(false)
    }

    #[inline]
    pub fn is_same_target(&self, camera: &Camera) -> bool {
        self.target == camera.target
    }
}

/// Represents an input pointer used for picking.
#[derive(Debug, Default, Clone, Component, PartialEq)]
pub struct PointerLocation {
    location: Option<Location>,
}
impl PointerLocation {
    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct PointerLocationEvent {
    pub id: PointerId,
    pub location: Location,
}
impl std::fmt::Display for PointerLocationEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event::Location::{:?} {}", self.id, self.location)
    }
}
impl PointerLocationEvent {
    pub fn new(id: PointerId, location: Location) -> Self {
        Self { id, location }
    }

    pub fn receive(
        mut events: EventReader<Self>,
        mut pointers: Query<(&PointerId, &mut PointerLocation)>,
    ) {
        for event_pointer in events.iter() {
            pointers.for_each_mut(|(id, mut pointer)| {
                if *id == event_pointer.id {
                    pointer.location = Some(event_pointer.location.to_owned());
                }
            })
        }
    }
}
