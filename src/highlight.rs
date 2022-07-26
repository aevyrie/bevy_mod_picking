use super::selection::*;
use crate::PausedForBlockers;
use bevy::{asset::Asset, prelude::*, render::color::Color};

/// Marker component to flag an entity as highlightable
#[derive(Component, Clone, Debug, Default)]
pub struct Highlight;

/// Component used to track the initial asset of a highlightable object, as well as for overriding
/// the default highlight materials.
#[derive(Component, Clone, Debug)]
pub struct Highlighting<T: Asset> {
    pub initial: Handle<T>,
    pub hovered: Option<Handle<T>>,
    pub pressed: Option<Handle<T>>,
    pub selected: Option<Handle<T>>,
}

/// Resource that defines the default highlighting assets to use. This can be overridden per-entity
/// with the [`Highlighting`] component.
pub struct DefaultHighlighting<T: Highlightable + ?Sized> {
    pub hovered: Option<Handle<T::HighlightAsset>>,
    pub pressed: Option<Handle<T::HighlightAsset>>,
    pub selected: Option<Handle<T::HighlightAsset>>,
}

/// This trait makes it possible for highlighting to be generic over any type of asset.
pub trait Highlightable: Default {
    /// The asset used to highlight the picked object. For a 3D mesh, this would probably be [`StandardMaterial`].
    type HighlightAsset: Asset;
    fn highlight_defaults(
        materials: Mut<Assets<Self::HighlightAsset>>,
    ) -> DefaultHighlighting<Self>;
    fn materials(world: &mut World) -> Mut<Assets<Self::HighlightAsset>> {
        world
            .get_resource_mut::<Assets<Self::HighlightAsset>>()
            .expect("Failed to get resource")
    }
}

#[derive(Default)]
pub struct StandardMaterialHighlight;
impl Highlightable for StandardMaterialHighlight {
    type HighlightAsset = StandardMaterial;

    fn highlight_defaults(
        mut materials: Mut<Assets<Self::HighlightAsset>>,
    ) -> DefaultHighlighting<Self> {
        DefaultHighlighting {
            hovered: Some(materials.add(Color::rgb(0.35, 0.35, 0.35).into())),
            pressed: Some(materials.add(Color::rgb(0.35, 0.75, 0.35).into())),
            selected: Some(materials.add(Color::rgb(0.35, 0.35, 0.75).into())),
        }
    }
}

#[derive(Default)]
pub struct ColorMaterialHighlight;
impl Highlightable for ColorMaterialHighlight {
    type HighlightAsset = ColorMaterial;

    fn highlight_defaults(
        mut materials: Mut<Assets<Self::HighlightAsset>>,
    ) -> DefaultHighlighting<Self> {
        DefaultHighlighting {
            hovered: Some(materials.add(Color::rgb(0.35, 0.35, 0.35).into())),
            pressed: Some(materials.add(Color::rgb(0.35, 0.75, 0.35).into())),
            selected: Some(materials.add(Color::rgb(0.35, 0.35, 0.75).into())),
        }
    }
}

impl<T: Highlightable> FromWorld for DefaultHighlighting<T> {
    fn from_world(world: &mut World) -> Self {
        T::highlight_defaults(T::materials(world))
    }
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
pub fn mesh_highlighting<T: 'static + Highlightable + Send + Sync>(
    paused: Option<Res<PausedForBlockers>>,
    global_default_highlight: Res<DefaultHighlighting<T>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut Handle<T::HighlightAsset>,
            Option<&Selection>,
            &Highlighting<T::HighlightAsset>,
        ),
        Or<(Changed<Interaction>, Changed<Selection>)>,
    >,
) {
    // Set non-hovered material when picking is paused (e.g. while hovering a picking blocker).
    if let Some(paused) = paused {
        if paused.is_paused() {
            for (_, mut material, selection, highlight) in interaction_query.iter_mut() {
                *material = if selection.filter(|s| s.selected()).is_some() {
                    highlight.selected.as_ref().unwrap_or(
                        global_default_highlight.selected.as_ref().unwrap_or(
                            &highlight.initial
                        )
                    )
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
                highlight.pressed.as_ref().unwrap_or(
                    global_default_highlight.pressed.as_ref().unwrap_or(
                        &highlight.initial
                    )
                )
            }
            Interaction::Hovered => {
                highlight.hovered.as_ref().unwrap_or(
                    global_default_highlight.hovered.as_ref().unwrap_or(
                        &highlight.initial
                    )
                )
            }
            Interaction::None => {
                if selection.filter(|s| s.selected()).is_some() {
                    highlight.selected.as_ref().unwrap_or(
                        global_default_highlight.as_ref().unwrap_or(
                            &highlight.initial
                        )
                    )
                } else {
                    &highlight.initial
                }
            }
        }
        .to_owned();
    }
}
