# Bugs

- [ ] The line renderer seems to be slightly bugged. This is especially visible when the line is
      huge. This might just be floating point precision errors.
- [ ] Right now, we're leaking memory. The world never unloads the chunks it doesn't need anymore.
- [ ] Chunk do not seem to actually load using the requested priority. Close chunks sometimes
      appear after far chunks.
- [x] Update package to optimize dependencies & not the current package.
- [x] The chunk building algorithm seems to be missing chunk corners.

# Other

- [ ] Add profiling options & display in the debug UI.

# Features

- [ ] Infinite terrain generation.
  - [ ] Base terrain.
  - [ ] Structures (trees, big rocks, etc).
  - [ ] Caves.
- [ ] Basic graphics.
  - [x] Optimized voxel renderer.
  - [x] Skybox.
  - [x] Reloadable texture atlas.
  - [ ] Ambiant Occlusion
  - [ ] Distance fog.
  - [ ] Text rendering.
  - [ ] Debug UI.
- [ ] Basic gameplay.
  - [ ] Physics system.
  - [ ] Player movement.
  - [ ] Mining, placing blocks.
- [ ] Advanced graphics.
  - [ ] Dynamic lighting.
  - [ ] Shadows.
  - [ ] Animated water.
  - [ ] Ambiant particles.
- [ ] Advanced gameplay.
  - [ ] Multiplayer.
  - [ ] Monsters AI.
- [ ] Misc
  - [ ] Screenshots.
