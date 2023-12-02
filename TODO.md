# Bugs

- [ ] The line renderer seems to be slightly bugged. This is especially visible when the line is
      huge. This might just be floating point precision errors.
- [ ] Right now, we're leaking memory. The world never unloads the chunks it doesn't need anymore.
- [ ] Chunk do not seem to actually load using the requested priority. Close chunks sometimes
      appear after far chunks.
- [x] The chunk building algorithm seems to be missing chunk corners.

# Other

- [ ] Add profiling options & display in the debug UI.
- [x] Update package to optimize dependencies & not the current package.
- [ ] Add workspace dependencies to avoid having to update every package manually.
- [x] Use `std::thread::available_parallelism` instead `num_cpus`.
- [ ] Add bedrock.

# Features

- [ ] Infinite terrain generation.
  - [ ] Modular biomes.
  - [ ] Base terrain.
  - [ ] Structures (trees, big rocks, etc).
- [ ] Basic graphics.
  - [x] Optimized voxel renderer.
  - [x] Skybox.
  - [x] Reloadable texture atlas.
  - [ ] Ambiant occlusion.
  - [ ] Distance fog.
  - [ ] Underwater fog.
  - [ ] Text rendering.
  - [ ] Debug UI.
  - [ ] Mipmaps.
- [ ] Basic gameplay.
  - [ ] Physics system.
  - [ ] Player movement.
  - [ ] Mining, placing blocks.
- [ ] Advanced graphics.
  - [ ] Dynamic lighting.
  - [ ] Shadows.
  - [ ] Animated water.
  - [ ] Ambiant particles.
- [ ] Misc
  - [ ] Screenshots.
