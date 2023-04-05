//! Adds highlighting functionality to `bevy_mod_picking`. Supports highlighting selection state
//! from `bevy_picking_selection`.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

#[allow(unused_imports)]
use bevy::{asset::Asset, prelude::*, render::color::Color};
use bevy_picking_core::PickSet;
#[cfg(feature = "selection")]
use bevy_picking_selection::PickSelection;

/// Adds pick highlighting functionality to your app.
pub struct HighlightingPlugin;
impl Plugin for HighlightingPlugin {
    #[allow(unused_variables)]
    fn build(&self, app: &mut App) {
        app.add_plugin(CustomHighlightPlugin::<StandardMaterial> {
            highlighting_default: |mut assets| DefaultHighlighting {
                hovered: assets.add(Color::rgb(0.35, 0.35, 0.35).into()),
                pressed: assets.add(Color::rgb(0.35, 0.75, 0.35).into()),
                #[cfg(feature = "selection")]
                selected: assets.add(Color::rgb(0.35, 0.35, 0.75).into()),
            },
        });

        #[cfg(feature = "bevy/bevy_sprite")]
        app.add_plugin(CustomHighlightPlugin::<bevy::sprite::ColorMaterial> {
            highlighting_default: |mut assets| DefaultHighlighting {
                hovered: assets.add(Color::rgb(0.35, 0.35, 0.35).into()),
                pressed: assets.add(Color::rgb(0.35, 0.75, 0.35).into()),
                #[cfg(feature = "selection")]
                selected: assets.add(Color::rgb(0.35, 0.35, 0.75).into()),
            },
        });
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
#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct PickHighlight;

/// [`HighlightOverride`] options.
#[derive(Clone)]
pub enum HighlightKind<T: Asset> {
    /// A simple fixed highlight material override for this entity.
    Fixed(Handle<T>),
    /// A function that takes the base material of the entity, and outputs a new material. This can
    /// be used to make "tinted" materials.
    Dynamic {
        /// The function to set.
        function: fn(initial: &T) -> T,
        /// The function will be run when the entity's Handle changes, and the output will be stored
        /// here.
        cache: Option<Handle<T>>,
    },
}

impl<T: Asset> HighlightKind<T> {
    /// Get a handle to the override [`Asset`].
    pub fn get_handle(&self) -> Option<Handle<T>> {
        match self {
            HighlightKind::Fixed(handle) => Some(handle.to_owned()),
            HighlightKind::Dynamic { cache, .. } => cache.to_owned(),
        }
    }

    /// Get the dynamic override function and cache if it exists.
    pub fn get_dynamic(&mut self) -> Option<(&fn(initial: &T) -> T, &mut Option<Handle<T>>)> {
        match self {
            Self::Dynamic { function, cache } => Some((function, cache)),
            _ => None,
        }
    }

    /// Create a [`HighlightKind::Dynamic`] with the supplied function. Useful when you want to
    /// tweak the initial `Asset` when highlighting, e.g. tinting `StandardMaterial` blue.
    pub const fn new_dynamic(function: fn(initial: &T) -> T) -> Self {
        Self::Dynamic {
            function,
            cache: None,
        }
    }
}

impl<T: Asset> std::fmt::Debug for HighlightKind<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixed(arg0) => f.debug_tuple("Fixed").field(arg0).finish(),
            Self::Dynamic { cache, .. } => f.debug_struct("Dynamic").field("cache", cache).finish(),
        }
    }
}

/// Overrides the highlighting appearance of an entity. See [`PickHighlight`].
#[derive(Component, Clone, Debug, Default)]
pub struct HighlightOverride<T: Asset> {
    /// Overrides this asset's global default appearance when hovered
    pub hovered: Option<HighlightKind<T>>,
    /// Overrides this asset's global default appearance when pressed
    pub pressed: Option<HighlightKind<T>>,
    /// Overrides this asset's global default appearance when selected
    #[cfg(feature = "selection")]
    pub selected: Option<HighlightKind<T>>,
}

impl<T: Asset> HighlightOverride<T> {
    /// System that updates the dynamic overrides when the entity's Handle changes.
    fn update_dynamic(
        mut asset_server: ResMut<Assets<T>>,
        mut entities: Query<
            (&mut HighlightOverride<T>, &InitialHighlight<T>),
            Changed<InitialHighlight<T>>,
        >,
    ) {
        for (mut highlight_override, highlight_initial) in entities.iter_mut() {
            let HighlightOverride {
                hovered,
                pressed,
                selected,
            } = highlight_override.as_mut();

            let mut h = hovered.as_mut().and_then(|h| h.get_dynamic());
            let mut p = pressed.as_mut().and_then(|h| h.get_dynamic());

            let iter = h.iter_mut().chain(p.iter_mut());

            #[cfg(feature = "selection")]
            let mut s = selected.as_mut().and_then(|h| h.get_dynamic());
            #[cfg(feature = "selection")]
            let iter = iter.chain(s.iter_mut());

            for (function, cache) in iter {
                if let Some(asset) = asset_server
                    .get(&highlight_initial.initial)
                    .map(|i| function(i))
                {
                    **cache = Some(asset_server.add(asset));
                }
            }
        }
    }
}

/// A highlighting plugin, generic over any asset that might be used for rendering the different
/// highlighting states.
pub struct CustomHighlightPlugin<T: 'static + Asset + Sync + Send> {
    /// A function that is invoked at startup to allow you to generate the default highlighting
    /// states for `T`.
    pub highlighting_default: fn(ResMut<Assets<T>>) -> DefaultHighlighting<T>,
}

impl<T> Plugin for CustomHighlightPlugin<T>
where
    T: 'static + Asset + Sync + Send,
{
    fn build(&self, app: &mut App) {
        let highlighting_default = self.highlighting_default;

        app.add_startup_system(move |mut commands: Commands, assets: ResMut<Assets<T>>| {
            commands.insert_resource(highlighting_default(assets));
        })
        .add_systems(
            (
                get_initial_highlight_asset::<T>,
                HighlightOverride::<T>::update_dynamic,
                update_highlight_assets::<T>,
                #[cfg(feature = "selection")]
                update_selection::<T>,
            )
                .chain()
                .in_set(PickSet::Last),
        );
    }
}

/// Component used to track the initial asset state of a highlightable object. This is needed to
/// return the highlighting asset back to its original state after highlighting it.
#[derive(Component, Clone, Debug, Reflect)]
pub struct InitialHighlight<T: Asset> {
    /// A handle for the initial asset state of the highlightable entity.
    pub initial: Handle<T>,
}

/// Resource that defines the default highlighting assets to use. This can be overridden per-entity
/// with the [`HighlightOverride`] component.
#[derive(Resource)]
pub struct DefaultHighlighting<T: Asset> {
    /// Default asset handle to use for hovered entities without a [`HighlightOverride`].
    pub hovered: Handle<T>,
    /// Default asset handle to use for pressed entities without a [`HighlightOverride`].
    pub pressed: Handle<T>,
    /// Default asset handle to use for selected entities without a [`HighlightOverride`].
    #[cfg(feature = "selection")]
    pub selected: Handle<T>,
}

impl<T: Asset> DefaultHighlighting<T> {
    /// Returns the hovered highlight override if it exists, falling back to the default.
    pub fn hovered(&self, h_override: &Option<&HighlightOverride<T>>) -> Handle<T> {
        h_override
            .and_then(|h| h.hovered.as_ref())
            .and_then(|h| h.get_handle())
            .unwrap_or_else(|| self.hovered.clone())
    }

    /// Returns the pressed highlight override if it exists, falling back to the default.
    pub fn pressed(&self, h_override: &Option<&HighlightOverride<T>>) -> Handle<T> {
        h_override
            .and_then(|h| h.pressed.as_ref())
            .and_then(|h| h.get_handle())
            .unwrap_or_else(|| self.pressed.clone())
    }
    /// Returns the selected highlight override if it exists, falling back to the default.
    #[cfg(feature = "selection")]
    pub fn selected(&self, h_override: &Option<&HighlightOverride<T>>) -> Handle<T> {
        h_override
            .and_then(|h| h.selected.as_ref())
            .and_then(|h| h.get_handle())
            .unwrap_or_else(|| self.selected.clone())
    }
}

/// Automatically records the "initial" state of highlightable entities.
pub fn get_initial_highlight_asset<T: Asset>(
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
pub fn update_highlight_assets<T: Asset>(
    global_defaults: Res<DefaultHighlighting<T>>,
    mut interaction_query: Query<
        (
            &mut Handle<T>,
            &Interaction,
            &InitialHighlight<T>,
            Option<&HighlightOverride<T>>,
        ),
        Changed<Interaction>,
    >,
) {
    for (mut asset, interaction, init_highlight, h_override) in &mut interaction_query {
        *asset = match interaction {
            Interaction::Clicked => global_defaults.pressed(&h_override),
            Interaction::Hovered => global_defaults.hovered(&h_override),
            Interaction::None => init_highlight.initial.to_owned(),
        }
    }
}

#[cfg(feature = "selection")]
/// If the interaction state of a selected entity is `None`, set the highlight color to `selected`.
pub fn update_selection<T: Asset>(
    global_defaults: Res<DefaultHighlighting<T>>,
    mut interaction_query: Query<
        (
            &mut Handle<T>,
            &Interaction,
            &PickSelection,
            &InitialHighlight<T>,
            Option<&HighlightOverride<T>>,
        ),
        Or<(Changed<PickSelection>, Changed<Interaction>)>,
    >,
) {
    for (mut asset, interaction, selection, init_highlight, h_override) in &mut interaction_query {
        if let Interaction::None = interaction {
            *asset = if selection.is_selected {
                global_defaults.selected(&h_override)
            } else {
                init_highlight.initial.to_owned()
            }
        }
    }
}
