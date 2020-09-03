# bevy_mod_picking

3D mouse picking prototype for Bevy. 
**Currently broken while I rework the API.**

## Notes

Casts a ray into the scene and checks for intersection for all meshes tagged with the `PickableMesh` component.

### Limitations

Current limitations I'd like to fix:

* Highlighting coupled with ray casting system: highlighting and selection state logic is bundled in the ray casting logic and API. I want to decouple this and make highlighting/selection an opt-in feature. Current plan is to run picking on meshes with the `PickableMesh` component, highlight meshes with the `HighlightableMesh` component, and manage selection state for meshes with the `SelectableMesh` component. Meshes with both `HighlightableMesh` and `SelectableMesh` will highlight when selected.

* No orthographic camera support: this hasn't been tested yet, but is an explicit goal of this plugin.

* Optimizations: the current ray casting implementation is naive, and queries all meshes in the scene. The first optimization I'd like to apply is checking against bounding spheres before checking each triangle in a mesh. This should greatly improve performance in cases where the cursor is hovering over an area with very few objects.

* Single camera and window: eventually I'd like to support picking for an arbitrary number of cameras and windows.

* No "color-picking" implementation: for performance, I'd like to render the scene to an off-screen buffer that renders each pixel as a mesh ID encoded into RGBA. Picking is then as simple as querying this buffer and doing a lookup to return a mesh handle.
