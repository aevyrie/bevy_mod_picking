# UNRELEASED

## Documentation

- Added: feature gated modules should now be visible on docs.rs, along with a note about the
  features required to use them.

## Cleanup

- Fixed: removed unused `PickSet::EventListeners` and fixed (upstream) eventlisteners running in the
  `Update` schedule instead of the `PreUpdate` schedule.
- Changed: Removed dependencies on `bevy` and instead depend on bevy subcrates (e.g. `bevy_ecs`)
  directly. This reduces total dependency count, but more impactful is that this allows compilation
  of each picking crate to start earlier in the build process, before `bevy` finishes.

## API Improvements

- Changed: the bevy_mod_raycast backend no longer requires markers on the camera
  (`RaycastPickCamera`) and targets (`RaycastPickTarget`).
- Added: `RaycastBackendSettings` resource added to allow toggling the requirement for markers with
  the bevy_mod_raycast backend at runtime. Enable the `require_markers` field to match behavior of
  the plugin prior to this release.
- Changed: The plugin no longer respects bevy_ui's `FocusPolicy`. This was proving to cause problems
  as mod_picking and bevy_ui have some fundamental differences that cannot be reconciled. This has
  been replaced by added fields on the `Pickable` component. You can use this to override the
  behavior of any entity in picking. This allows you to decide if that entity will block lower
  entities (on by default), and if that entity should emit events and be hover-able (on by default).
- To make objects non-pickable, instead of removing the `Pickable` entity, use the new const value
  `Pickable::IGNORE`.
- Changed: The `PointerInteraction` component, which is added to pointers and tracks all entities
  being interacted with has changed internally from a hashmap of entities and their `Interaction` to
  a sorted list of entities and `HitData`, sorted by depth. This better reflects how pointer data
  would be expected to work, and makes it easier to find the topmost entity under each pointer.
- Added: `get_nearest_hit` method added to `PointerInteraction`
- Changed: Moved `PointerInteraction` from the `focus` module to `pointer`.

## Backend Improvements

- Added: `bevy_mod_raycast` backend now checks render layers when filtering entities.
- Changed: `PickLayer`, used to order data from backends that targets the same render target, such
  as multiple render passes on the same window, has been changed from an `isize` to an `f32`. This
  change was made to support bevy_ui, which is "special" and can be rendered on any camera via a
  flag, instead of being rendered with its own camera. The bevy_ui backend now sets the order of any
  events emitted to be the camera order plus 0.5, which was not possible with an integer.
- Added: the sprite backend now supports sprite scale, rotation, and custom anchors.

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
