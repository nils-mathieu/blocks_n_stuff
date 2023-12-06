Cool thing I made in Rust.

This project is supposed to be sort of a Minecraft clone to show off my skills in graphics
programming.

A web version of this project is available at https://nils-mathieu.fr/blocks_n_stuff/.

## Technologies

This project uses the following thrid-party Rust libraries:

- `wgpu`, a cross-platform abstraction over Vulkan, Metal, DX12, OpenGL and WebGPU. I would've
  liked to use Vulkan directly, but that would make the project unusable on web, which I really
  want to target.
- `winit`, a cross-platform windowing library.
- `glam`, a linear algebra library that makes use of SIMD instructions to optimize most operations.
- `png`, a PNG image decoder/encoder.

## Keybindings

### Movements

| Key                     | Action        |
| ----------------------- | ------------- |
| <kbd>W</kbd>            | Move forward  |
| <kbd>A</kbd>            | Move left     |
| <kbd>S</kbd>            | Move backward |
| <kbd>D</kbd>            | Move right    |
| <kbd>Space</kbd>        | Fly Up        |
| <kbd>Left shift</kbd>   | Fly Down      |
| <kbd>Left control</kbd> | Sprint        |

### Misc

| Key                   | Action                   |
| --------------------- | ------------------------ |
| <kbd>Escape</kbd>     | Exit game                |
| <kbd>F11</kbd>        | Toggle fullscreen        |
| <kbd>R</kbd>          | Re-create world          |
| <kbd>Arrow up</kbd>   | Increase render distance |
| <kbd>Arrow down</kbd> | Decrease render distance |
| <kbd>F10</kbd>        | Toggle fog               |

### Debug

| Key           | Action               |
| ------------- | -------------------- |
| <kbd>F3</kbd> | Toggle debug overlay |
| <kbd>F4</kbd> | Toggle chunk borders |
