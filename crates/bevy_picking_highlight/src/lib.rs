//! Adds highlighting functionality to `bevy_mod_picking`. Supports highlighting selection state
//! from `bevy_picking_selection`.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use std::marker::PhantomData;

use bevy::{app::PluginGroupBuilder, asset::Asset, prelude::*, render::color::Color};
use bevy_picking_core::PickStage;
use bevy_picking_selection::PickSelection;

/// Adds pick highlighting functionality to your app.
pub struct HighlightingPlugins;
impl PluginGroup for HighlightingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(CustomHighlightingPlugin::<StandardMaterial>::default());
        group.add(CustomHighlightingPlugin::<ColorMaterial>::default());
    }
}

/// Makes an entity highlightable with any [`Highlightable`] [`Asset`]. By default, this plugin
/// provides an implementation for [`StandardMaterial`] and [`ColorMaterial`]. If this entity has
/// either of those asset handles, the plugin will automatically update them to match the entity's
/// interaction state. To use another asset type, all you need to do is implement [`Highlightable`]
/// for the asset and add the [`CustomHighlightingPlugin::<T>`] plugin to your app, where `T` is
/// your asset type.
///
/// ### Overriding Highlighting Appearance
///
/// By default, this plugin will use [`DefaultHighlighting<T>`] for assets of type `T`. You can
/// override this global default with the optional fields in the [`HighlightOverride`] component.
#[derive(Component, Clone, Debug, Default)]
pub struct PickHighlight;

/// Overrides the highlighting appearance of an entity. See [`PickHighlight`].
#[derive(Component, Clone, Debug, Default)]
pub struct HighlightOverride<T: Highlightable> {
    /// Overrides this asset's global default appearance when hovered
    pub hovered: Option<Handle<T>>,
    /// Overrides this asset's global default appearance when pressed
    pub pressed: Option<Handle<T>>,
    /// Overrides this asset's global default appearance when selected
    pub selected: Option<Handle<T>>,
}

/// A highlighting plugin, generic over any asset that might be used for rendering the different
/// highlighting states.
#[derive(Default)]
pub struct CustomHighlightingPlugin<T: 'static + Highlightable + Sync + Send>(PhantomData<T>);

impl<T> Plugin for CustomHighlightingPlugin<T>
where
    T: 'static + Highlightable + Sync + Send,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<DefaultHighlighting<T>>()
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .after(PickStage::Focus)
                    .with_system(get_initial_highlight_asset::<T>)
                    .with_system(
                        update_highlight_assets::<T>
                            .after(get_initial_highlight_asset::<T>)
                            .after(bevy_picking_selection::send_selection_events),
                    ),
            );
    }
}

/// Component used to track the initial asset state of a highlightable object. This is needed to
/// return the highlighting asset back to its original state after highlighting it.
#[derive(Component, Clone, Debug)]
pub struct InitialHighlight<T: Asset> {
    /// A handle for the initial asset state of the highlightable entity.
    pub initial: Handle<T>,
}

/// Resource that defines the default highlighting assets to use. This can be overridden per-entity
/// with the [`HighlightOverride`] component.
pub struct DefaultHighlighting<T: Highlightable + ?Sized> {
    /// Default asset handle to use for hovered entities without a [`HighlightOverride`].
    pub hovered: Handle<T>,
    /// Default asset handle to use for pressed entities without a [`HighlightOverride`].
    pub pressed: Handle<T>,
    /// Default asset handle to use for selected entities without a [`HighlightOverride`].
    pub selected: Handle<T>,
}

impl<T: Highlightable> DefaultHighlighting<T> {
    /// Returns the hovered highlight override if it exists, falling back to the default.
    pub fn hovered(&self, h_override: &Option<&HighlightOverride<T>>) -> Handle<T> {
        if let Some(h_override) = h_override.and_then(|h| h.hovered.as_ref()) {
            h_override.to_owned()
        } else {
            self.hovered.clone()
        }
    }

    /// Returns the pressed highlight override if it exists, falling back to the default.
    pub fn pressed(&self, h_override: &Option<&HighlightOverride<T>>) -> Handle<T> {
        if let Some(h_override) = h_override.and_then(|h| h.pressed.as_ref()) {
            h_override.to_owned()
        } else {
            self.pressed.clone()
        }
    }
    /// Returns the selected highlight override if it exists, falling back to the default.
    pub fn selected(&self, h_override: &Option<&HighlightOverride<T>>) -> Handle<T> {
        if let Some(h_override) = h_override.and_then(|h| h.selected.as_ref()) {
            h_override.to_owned()
        } else {
            self.selected.clone()
        }
    }
}

/// This trait makes it possible for highlighting to be generic over any type of asset. This can be
/// implement this for any [`Asset`] type.
pub trait Highlightable: Default + Asset {
    /// The asset used to highlight the picked object. For a 3D mesh, this might be [`StandardMaterial`].
    fn highlight_defaults(materials: Mut<Assets<Self>>) -> DefaultHighlighting<Self>;

    /// Retrieves the asset storage, [`Assets`], for the highlighting asset type. This allows
    /// handles for assets used for highlighting to be dereferenced.
    fn materials(world: &mut World) -> Mut<Assets<Self>> {
        world.resource_mut::<Assets<Self>>()
    }
}

impl Highlightable for StandardMaterial {
    fn highlight_defaults(mut materials: Mut<Assets<Self>>) -> DefaultHighlighting<Self> {
        DefaultHighlighting {
            hovered: materials.add(Color::rgb(0.35, 0.35, 0.35).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            selected: materials.add(Color::rgb(0.35, 0.35, 0.75).into()),
        }
    }
}

impl Highlightable for ColorMaterial {
    fn highlight_defaults(mut materials: Mut<Assets<Self>>) -> DefaultHighlighting<Self> {
        DefaultHighlighting {
            hovered: materials.add(Color::rgb(0.35, 0.35, 0.35).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            selected: materials.add(Color::rgb(0.35, 0.35, 0.75).into()),
        }
    }
}

impl<T: Highlightable> FromWorld for DefaultHighlighting<T> {
    fn from_world(world: &mut World) -> Self {
        T::highlight_defaults(T::materials(world))
    }
}

/// Automatically records the "initial" state of highlightable entities.
pub fn get_initial_highlight_asset<T: Highlightable>(
    mut commands: Commands,
    entity_asset_query: Query<(Entity, &Handle<T>), Added<PickHighlight>>,
    mut highlighting_query: Query<Option<&mut InitialHighlight<T>>>,
) {
    for (entity, material) in entity_asset_query.iter() {
        match highlighting_query.get_mut(entity) {
            Ok(Some(mut highlighting)) => highlighting.initial = material.to_owned(),
            _ => {
                commands.entity(entity).insert(InitialHighlight {
                    initial: material.to_owned(),
                });
            }
        }
    }
}

/// Apply highlighting assets to entities based on their state.
pub fn update_highlight_assets<T: 'static + Highlightable + Send + Sync>(
    global_defaults: Res<DefaultHighlighting<T>>,
    mut interaction_query: Query<
        (
            &mut Handle<T>,
            &Interaction,
            Option<&PickSelection>,
            &InitialHighlight<T>,
            Option<&HighlightOverride<T>>,
        ),
        Or<(Changed<Interaction>, Changed<PickSelection>)>,
    >,
) {
    for (mut asset, interaction, selection, init_highlight, h_override) in &mut interaction_query {
        match interaction {
            Interaction::Clicked => *asset = global_defaults.pressed(&h_override),
            Interaction::Hovered => *asset = global_defaults.hovered(&h_override),
            Interaction::None => {
                *asset = match selection {
                    Some(PickSelection { is_selected: true }) => {
                        global_defaults.selected(&h_override)
                    }
                    _ => init_highlight.initial.to_owned(),
                }
            }
        }
    }
}
