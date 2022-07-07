use crate::PointerId;
use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct PointerOverEvent {
    pub id: PointerId,
    pub over_list: Vec<PointerOverMetadata>,
}
impl std::fmt::Display for PointerOverEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event::Over::{:?} {:?}", self.id, self.over_list)
    }
}

/// A component that assigns an entity to a picking layer. When computing picking focus, entities
/// are sorted in order from the highest to lowest layer, and by depth within each layer.
#[derive(Debug, Clone, Copy, Component, PartialEq, Eq, PartialOrd, Ord)]
pub struct PickLayer(u8);
impl PickLayer {
    pub fn above_all() -> Self {
        PickLayer(0)
    }
    pub fn ui() -> Self {
        PickLayer(10)
    }
    pub fn above_world() -> Self {
        PickLayer(20)
    }
    pub fn world() -> Self {
        PickLayer(30)
    }
    pub fn below_world() -> Self {
        PickLayer(40)
    }
    pub fn below_all() -> Self {
        PickLayer(50)
    }
    pub fn custom(layer: u8) -> Self {
        PickLayer(layer)
    }
    pub fn layer(&self) -> u8 {
        self.0
    }
}
impl Default for PickLayer {
    fn default() -> Self {
        PickLayer::world()
    }
}
impl std::fmt::Display for PickLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                0 => "Above All".into(),
                10 => "UI".into(),
                20 => "Above World".into(),
                30 => "World".into(),
                40 => "Below World".into(),
                50 => "Below All".into(),
                n => format!("Custom {n}"),
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct PointerOverMetadata {
    pub entity: Entity,
    pub depth: f32,
}
