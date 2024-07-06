# 0.20.0

- Updated for Bevy `0.14.0`.

# 0.19.0

## Fixes:

- `HighlightPluginSettings::is_enabled` made public.
- `bevy_ui` nodes with zero sized rects are ignored to fix a false positive.
- Fixed touch pointer inputs being missed.

## Additions:

- Created a new `render_to_texture` example to showcase how to render viewport textures that support
  picking.
- Added support for a range of compatible versions of egui and rapier, instead of being tied to a
  specific version of these crates.

# 0.18.0

## Announcements:

This plugin is now officially planned to be upstreamed into Bevy. This will require the
`bevy_eventlistener` crate is also upstreamed, and a few critical features added or changed based on
feedback from Bevy's BDFL. Many of the sub-crates in this repo are not appropriate for upstreaming,
and will remain here. This includes things like the `egui` or `rapier` backends, or the highlighting
crate.

The intent is to upstream `bevy_eventlistener` and `bevy_picking_core` to be included in Bevy
`0.14`, time permitting.

## Known bugs:

- The `bevy_ui` debug overlay is broken until <https://github.com/bevyengine/bevy/pull/12268> is
  released as part of `bevy 0.13.1`.

## Changes:

### New `bevy_xpbd backend`

- Added: `bevy_xpbd_3d` picking backend, available behind the optional `backend_xpbd` feature.

### Automatic 3D Pointer Rays

- Added: `RayMap` resource that contains a `Ray` for every (camera, pointer) pair. This is useful
  for any users building a raycasting-based picking backend, handling viewports and scale for you. 
- Changed: rapier and bevy_mod_raycast backends use the `RayMap` instead of constructing their own
  rays.
- Fixed: Ray construction now respects DPI / window scale.

### Expanded features

- Added: support for touch inputs across multiple windows.
- Added: support for `bevy_ui` `UiScale`.
- Added: `RaycastBackendSettings::raycast_visibility` to support picking hidden meshes.
- Changed: simplified debug settings and examples. Debug settings can be changed with the
  `DebugPickingMode` resource.

### Bug Fixes

- Fixed: sprite picking depth is now consistent with other picking backends.
- Fixed: entities with identical depth could be dropped due to the use of a BTreeMap to sort
  entities by depth. This has been changed to use a sorted Vec, to allow entities with the same
  depth to coexist.
- Fixed: rapier and bevy_mod_raycast backends use `RenderLayers::default` when a camera is missing
  them.
- Fixed: the bevy ui backend now ignores clipped areas of UI nodes. 
- Fixed: `PickingPluginsSettings::enable_input` is now used as a run condition for `PickSet::ProcessInput`.
- Fixed: replaced uses of `.insert` with `.try_insert`, where they could potentially panic.
- Fixed: replace all `.single` calls with matched `.get_single` calls to avoid crashing in
  environments where there is no window available.

### Documentation and Naming

- Documentation pass over all docs.
- Changed: renamed `Pickable::should_emit_events` to `is_hoverable` and clarified the docs.
- Removed: `PickingPluginsSettings::enable_highlighting`, which was nonfunctional.
- Changed: renamed `PickingPluginsSettings`, `HighlightingPluginSettings` fields to be consistent.
  Renamed `SelectionSettings` to `SelectionPluginSettings`.

# 0.17.0

- Update for bevy 0.12
- Fixed: rapier and raycast backends were not considering all cases when cameras or entities lacked
  the `RenderLayers` component.
- Fixed: crate would not compile when the bevy ui backend feature was disabled.
- Added: backends now re-export their dependency where applicable. `bevy_picking_raycast` re-exports
  `bevy_mod_raycast`, for example.
- Added: `Reflect` implementations for all components and resources.

# 0.16.0

## Highlights

- Faster compile times.
- Sprites now support atlases, scale, rotation, and anchors.
- All `egui` widgets, including side panels, are now supported.
- `bevy_mod_raycast` and `bevy_rapier` backends are now even simpler, no longer requiring any marker
  components to function.
- More flexible picking behavior and `bevy_ui` compatibility with the updated `Pickable` component.
- Better support for cameras settings such as `is_active`, `RenderLayers`, and `show_ui`.

## Dependencies

- Changed: Removed dependencies on `bevy` and instead depend on bevy subcrates (e.g. `bevy_ecs`)
  directly. This reduces total dependency count, but more importantly allows compilation of each
  picking crate to start before `bevy` finishes.
- Changed: `bevy_ui` has been removed as a core dependency, and is now completely optional.

## API Improvements

- Changed: The plugin no longer respects bevy_ui's `FocusPolicy` because it was not flexible enough.
  This has been replaced with new fields on the `Pickable` component. You can use this to override
  the behavior of any entity in picking. 
  - This allows you to decide if that entity will block lower entities (on by default), and if that
  entity should emit events and be hover-able (on by default).
  - To make objects non-pickable, instead of removing the `Pickable` entity, use the new const value
  `Pickable::IGNORE`.
- Changed: The `PointerInteraction` component, which is added to pointers and tracks all entities
  being interacted with has changed internally from a hashmap of entities and their `Interaction` to
  a sorted list of entities and `HitData`, sorted by depth. This better reflects how pointer data
  would be expected to work, and makes it easier to find the topmost entity under each pointer.
- Added: `get_nearest_hit` method added to `PointerInteraction`
- Changed: Moved `PointerInteraction` from the `focus` module to `pointer`.

## Backend Improvements

- Fixed: all backends now correctly check if a camera `is_active` before attempting hit tests.
- Changed: `PickLayer`, which is used to order hits from backends that targets the same render
  target, such as multiple render passes on the same window, has been changed from an `isize` to an
  `f32`. This change was made to support `bevy_ui`, which is "special" and can be rendered on any
  camera via a flag, instead of being rendered with its own camera. The `bevy_ui` backend now adds
  `0.5` to the camera order of any events emitted, which was not possible with an integer.

### Sprite Backend

- Added: support for sprite scale, rotation, and custom anchors.
- Added: support for sprite atlases.

### `bevy_mod_raycast` Backend
- Fixed: the backend now checks render layers when filtering entities.
- Changed: `RaycastPickCamera` and `RaycastPickTarget` markers components are not longer required.
  These components have been replaced with a single `RaycastPickable` marker.
- Added: `RaycastBackendSettings` resource added to allow toggling the above requirement for markers
  at runtime. Enable the `require_markers` field to match behavior of the plugin prior to this
  release.

### `bevy_rapier` Backend
- Fixed: the backend now checks render layers when filtering entities.
- Changed: `RapierPickCamera` and `RapierPickTarget` markers components are not longer required.
  These components have been replaced with a single `RapierPickable` marker.
- Added: `RapierBackendSettings` resource added to allow toggling the above requirement for markers
  at runtime. Enable the `require_markers` field to match behavior of the plugin prior to this
  release.

### `bevy_egui` Backend
- Fixed: backend not detecting hits over `SidePanel`s and other widgets. The backend now runs in
  `PostUpdate`, which means egui hit tests will be one frame out of date. This is required because
  users tend to build their `egui` UI in `Update`, and egui rebuilds the entire UI from scratch
  every frame, so the picking backend must be run after users have built their UI.
- Fixed: backend not returning hits when egui is using the pointer to resize windows or drag widgets
  like sliders or windows.

## Miscellaneous

- Fixed: a system order ambiguity meant that sometimes clicks would be applied to `PickSelection`
  one frame late, and sometimes not. This led to unreliable and sometimes broken behavior, so we fix
  their ordering to ensure `PickSelection` is always updated on the frame the pointer is released.
- Fixed: removed unused `PickSet::EventListeners` and fixed (upstream) eventlisteners running in the
  `Update` schedule instead of the `PreUpdate` schedule.
- Fixed: `interaction_should_run` updating the wrong `PickingPluginsSettings` field. 

# 0.15.0

- Update to Bevy 0.11.
- Removed: The `PickingBackend` trait is no longer required and has been removed.
- Fixed: bevy_ui backend incorrectly skipping valid UI cameras.
- Changed: The plugin no longer respects bevy_ui's `FocusPolicy`. This was proving to cause problems
  as mod_picking and bevy_ui have some fundamental differences that cannot be reconciled. This has
  been replaced by added fields on the `Pickable` component. You can use this to override the
  behavior of any entity in picking. This allows you to decide if that entity will block lower
  entities (on by default), and if that entity should emit events and be hover-able (on by default).
    - To make objects non-pickable, instead of removing the `Pickable` entity, use the new const
      value `Pickable::IGNORE`.
- Changed: The `Pickable` component is now optional. Backends can choose to use it for optimization,
  but should not filter out entities that do not have this component. An example of an optimization
  would be to early exit and stop raycasting once an entity is hit that blocks the entities below
  it.
- Changed: To fully remove bevy_ui as a dependency and avoid issues similar to the `FocusPolicy`
  change, bevy_mod_picking no longer updates the bevy_ui `Interaction` state. This has been replaced
  with a `PickingInteraction` component that serves a similar purpose. This component aggregates the
  picking state of an entity (press, hover, none) across *all* pointers.
- Changed: `PickLayer`, used to order data from backends that targets the same render target, such
  as multiple render passes on the same window, has been changed fom an `isize` to an `f32`. This
  change was made to support bevy_ui, which is "special" and can be rendered on any camera via a
  flag, instead of being rendered with its own camera. The bevy_ui backend now sets the order of any
  events emitted to be the camera order plus 0.5, which was not possible with an integer.
- Changed: The `PointerInteraction` component, which is added to pointers and tracks all entities
  being interacted with has changed internally from a hashmap of entities and their `Interaction` to
  a sorted list of entities and `HitData`, sorted by depth. This better reflects how pointer data
  would be expected to work, and makes it easier to find the topmost entity under each pointer.
- Added: `get_nearest_hit` method added to `PointerInteraction`
- Changed: Moved `PointerInteraction` from the `focus` module to `pointer`.
- Added: `split_screen` viewport example.
- Added: `many_events` and `many_buttons` stress test examples.
- Added: constructors for `HitData` and `PointerHits`.
- Changed: selection only considers clicks with the primary pointer button.
