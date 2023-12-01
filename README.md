Awesome project, show-off project for my application at [*Hypixel Studos*][hs].

[hs]: https://hypixelstudios.com/

This project is supposed to be sort of a Minecraft clone to show off my skills in graphics
programming.

## Technologies

This project is written in Rust, because I absolutely love the language. It's fast and allows
me to write safe abstractions over fast unsafe code. I know Hypixel Studios uses C++ for their
engine, but I'd rather use Rust :P.

This project uses the following thrid-party libraries:

- `wgpu`, a cross-platform abstraction over Vulkan, Metal, DX12, OpenGL and WebGPU. I would've
  liked to use Vulkan directly, but that would make the project unusable on web, which I really
  want to target.
- `winit`, a cross-platform windowing library.
- `glam`, a linear algebra library that makes use of SIMD instructions to optimize most operations.
- `png`, a PNG image decoder/encoder.

## Todo

Here is a list of the things I want to implement:

- [ ] Infinite terrain generation.
  - [ ] Biomes.
  - [ ] Structures (trees, dungeons).
  - [ ] Caves.
- [ ] Basic graphics.
  - [x] Optimized voxel renderer.
  - [x] Skybox.
  - [x] Reloadable texture atlas.
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
- [ ] Advanced gameplay.
  - [ ] Multiplayer.
  - [ ] Monsters AI.
- [ ] Misc
  - [ ] Screenshots.
