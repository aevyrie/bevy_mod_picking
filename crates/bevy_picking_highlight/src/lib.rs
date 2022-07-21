#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use std::marker::PhantomData;

use bevy::{app::PluginGroupBuilder, asset::Asset, prelude::*, render::color::Color};
use bevy_picking_core::{
    input::PointerPress,
    output::{PointerClick, PointerDown, PointerOut, PointerOver, PointerUp},
    PickStage, PointerId,
};

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

/// Overrides the highlighting appearance of an entity. See
#[derive(Component, Clone, Debug, Default)]
pub struct HighlightOverride<T: Highlightable> {
    /// Overrides this asset's global default appearance when hovered
    pub hovered: Option<Handle<T>>,
    /// Overrides this asset's global default appearance when pressed
    pub pressed: Option<Handle<T>>,
    /// Overrides this asset's global default appearance when selected
    pub selected: Option<Handle<T>>,
}
impl<T: Highlightable> HighlightOverride<T> {
    pub fn hovered(&self, default: &DefaultHighlighting<T>) -> Handle<T> {
        self.hovered.as_ref().unwrap_or(&default.hovered).to_owned()
    }
    pub fn pressed(&self, default: &DefaultHighlighting<T>) -> Handle<T> {
        self.pressed.as_ref().unwrap_or(&default.pressed).to_owned()
    }
    pub fn selected(&self, default: &DefaultHighlighting<T>) -> Handle<T> {
        self.selected
            .as_ref()
            .unwrap_or(&default.selected)
            .to_owned()
    }
}

pub struct HighlightingPlugins;
impl PluginGroup for HighlightingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(CustomHighlightingPlugin::<StandardMaterial>::default());
        group.add(CustomHighlightingPlugin::<ColorMaterial>::default());
    }
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
                CoreStage::First,
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
    pub initial: Handle<T>,
}

/// Resource that defines the default highlighting assets to use. This can be overridden per-entity
/// with the [`HighlightOverride`] component.
pub struct DefaultHighlighting<T: Highlightable + ?Sized> {
    pub hovered: Handle<T>,
    pub pressed: Handle<T>,
    pub selected: Handle<T>,
}

/// This trait makes it possible for highlighting to be generic over any type of asset. You can
/// implement this for any [`Asset`] type.
pub trait Highlightable: Default + Asset {
    /// The asset used to highlight the picked object. For a 3D mesh, this might be [`StandardMaterial`].
    fn highlight_defaults(materials: Mut<Assets<Self>>) -> DefaultHighlighting<Self>;
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

pub fn get_initial_highlight_asset<T: Highlightable>(
    mut commands: Commands,
    entity_asset_query: Query<(Entity, &Handle<T>), Added<PickHighlight>>,
    mut highlighting_query: Query<Option<&mut InitialHighlight<T>>>,
) {
    for (entity, material) in entity_asset_query.iter() {
        match highlighting_query.get_mut(entity) {
            Ok(Some(mut highlighting)) => highlighting.initial = material.to_owned(),
            _ => {
                commands
                    .entity(entity)
                    .insert(InitialHighlight {
                        initial: material.to_owned(),
                    })
                    .insert(HighlightOverride::<T> {
                        hovered: None,
                        pressed: None,
                        selected: None,
                    });
            }
        }
    }
}

pub fn update_highlight_assets<T: 'static + Highlightable + Send + Sync>(
    global_defaults: Res<DefaultHighlighting<T>>,
    mut interaction_query: Query<(
        &mut Handle<T>,
        Option<&bevy_picking_selection::PickSelection>,
        &InitialHighlight<T>,
        &HighlightOverride<T>,
    )>,
    pointer: Query<(&PointerId, &PointerPress)>,
    mut up_events: EventReader<PointerUp>,
    mut down_events: EventReader<PointerDown>,
    mut over_events: EventReader<PointerOver>,
    mut out_events: EventReader<PointerOut>,
    mut click_events: EventReader<PointerClick>,
) {
    for event in up_events.iter() {
        // Reset *all* deselected entities
        for (mut active_asset, pick_selection, h_initial, _) in interaction_query.iter_mut() {
            match pick_selection {
                Some(s) if !s.is_selected => *active_asset = h_initial.initial.to_owned(),
                _ => (),
            }
        }

        // Only update the entity picked in the current interaction event:
        if let Ok((mut active_asset, pick_selection, _, h_override)) =
            interaction_query.get_mut(event.target())
        {
            *active_asset = match pick_selection {
                Some(s) if s.is_selected => h_override.selected(&global_defaults),
                _ => h_override.hovered(&global_defaults),
            };
        }
    }

    for event in down_events.iter() {
        if let Ok((mut active_asset, _, _, h_override)) = interaction_query.get_mut(event.target())
        {
            *active_asset = h_override.pressed(&global_defaults);
        }
    }

    for event in over_events.iter() {
        if let Ok((mut active_asset, _, _, h_override)) = interaction_query.get_mut(event.target())
        {
            if pointer
                .iter()
                .any(|(id, press)| *id == event.id() && press.is_primary_down())
            {
                *active_asset = h_override.pressed(&global_defaults);
            } else {
                *active_asset = h_override.hovered(&global_defaults);
            }
        }
    }

    for event in out_events.iter() {
        if let Ok((mut active_asset, pick_selection, h_initial, h_override)) =
            interaction_query.get_mut(event.target())
        {
            *active_asset = match pick_selection {
                Some(s) if s.is_selected => h_override.selected(&global_defaults),
                _ => h_initial.initial.clone(),
            };
        }
    }

    for event in click_events.iter() {
        if let Ok((mut active_asset, pick_selection, _, h_override)) =
            interaction_query.get_mut(event.target())
        {
            *active_asset = match pick_selection {
                Some(s) if s.is_selected => h_override.selected(&global_defaults),
                _ => h_override.hovered(&global_defaults),
            };
        }
    }
}
