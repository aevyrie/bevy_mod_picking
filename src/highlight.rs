use super::selection::*;
use crate::PausedForBlockers;
use bevy::{asset::Asset, prelude::*};

/// Marker component to flag an entity as highlightable
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Highlight;

/// Component used to track the initial asset of a highlightable object, as well as for overriding
/// the default highlight materials.
#[derive(Component, Clone, Debug, Reflect)]
pub struct Highlighting<T: Asset> {
    pub initial: Handle<T>,
    pub hovered: Option<Handle<T>>,
    pub pressed: Option<Handle<T>>,
    pub selected: Option<Handle<T>>,
}

/// Resource that defines the default highlighting assets to use. This can be overridden per-entity
/// with the [`Highlighting`] component.
#[derive(Clone, Debug, Resource)]
pub struct DefaultHighlighting<T: Asset> {
    pub hovered: Handle<T>,
    pub pressed: Handle<T>,
    pub selected: Handle<T>,
}

#[allow(clippy::type_complexity)]
pub fn get_initial_mesh_highlight_asset<T: Asset>(
    mut commands: Commands,
    entity_asset_query: Query<(Entity, &Handle<T>), Added<Highlight>>,
    mut highlighting_query: Query<Option<&mut Highlighting<T>>>,
) {
    for (entity, material) in entity_asset_query.iter() {
        match highlighting_query.get_mut(entity) {
            Ok(Some(mut highlighting)) => highlighting.initial = material.to_owned(),
            _ => {
                let init_component = Highlighting {
                    initial: material.to_owned(),
                    hovered: None,
                    pressed: None,
                    selected: None,
                };
                commands.entity(entity).insert(init_component);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn mesh_highlighting<T: 'static + Asset + Send + Sync>(
    paused: Option<Res<PausedForBlockers>>,
    global_default_highlight: Res<DefaultHighlighting<T>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut Handle<T>,
            Option<&Selection>,
            &Highlighting<T>,
        ),
        Or<(Changed<Interaction>, Changed<Selection>)>,
    >,
) {
    // Set non-hovered material when picking is paused (e.g. while hovering a picking blocker).
    if let Some(paused) = paused {
        if paused.is_paused() {
            for (_, mut material, selection, highlight) in interaction_query.iter_mut() {
                *material = if selection.filter(|s| s.selected()).is_some() {
                    if let Some(highlight_asset) = &highlight.selected {
                        highlight_asset
                    } else {
                        &global_default_highlight.selected
                    }
                } else {
                    &highlight.initial
                }
                .to_owned();
            }
            return;
        }
    }
    for (interaction, mut material, selection, highlight) in interaction_query.iter_mut() {
        *material = match *interaction {
            Interaction::Clicked => {
                if let Some(highlight_asset) = &highlight.pressed {
                    highlight_asset
                } else {
                    &global_default_highlight.pressed
                }
            }
            Interaction::Hovered => {
                if let Some(highlight_asset) = &highlight.hovered {
                    highlight_asset
                } else {
                    &global_default_highlight.hovered
                }
            }
            Interaction::None => {
                if selection.filter(|s| s.selected()).is_some() {
                    if let Some(highlight_asset) = &highlight.selected {
                        highlight_asset
                    } else {
                        &global_default_highlight.selected
                    }
                } else {
                    &highlight.initial
                }
            }
        }
        .to_owned();
    }
}
