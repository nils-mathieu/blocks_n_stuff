use std::sync::Arc;
use std::time::Duration;

use bns_app::{Ctx, KeyCode};
use bns_core::ChunkPos;
use bns_render::data::{
    CharacterInstance, CharacterInstanceCursor, Color, LineInstance, LineVertexFlags, RenderData,
    Ui,
};
use bns_render::{DynamicVertexBuffer, Gpu};

use glam::{Vec2, Vec3};

/// Contains some state that's only used for debugging purposes, such
/// as the debug overlay and the buffer to store chunk border lines.
pub struct DebugThings {
    /// Whether the debug overlay is currently enabled.
    overlay: bool,
    /// The current state of the debug chunk display.
    chunk_state: DebugChunkState,

    /// The content of the debug overlay.
    overlay_buffer: DebugOverlayBuffer,
    /// The actual GPU buffer that will be uploaded to.
    overlay_gpu_buffer: DynamicVertexBuffer<CharacterInstance>,

    /// The total amount of time since the last time
    /// the average frame time was computed.
    accumulated_frame_time: Duration,
    /// The total nubmer of frames that have been accumulated since
    /// the last time the average frame time was computed.
    accumulated_frame_count: u32,
    /// The last average frame time computed.
    average_frame_time: Duration,
}

impl DebugThings {
    /// The minimum amount of time that must have passed before the average frame time
    /// is computed.
    pub const FRAME_TIME_THRESHOLD: Duration = Duration::from_millis(500);

    /// Creates a new [`DebugThings`] instance.
    pub fn new(gpu: Arc<Gpu>) -> Self {
        Self {
            overlay: false,
            chunk_state: DebugChunkState::Hidden,
            overlay_buffer: DebugOverlayBuffer::new(),
            overlay_gpu_buffer: DynamicVertexBuffer::new(gpu, 64),
            accumulated_frame_time: Duration::ZERO,
            accumulated_frame_count: 0,
            average_frame_time: Duration::ZERO,
        }
    }

    /// Ticks the debug overlay.
    #[profiling::function]
    pub fn tick(&mut self, ctx: &mut Ctx) {
        // ==================================
        // Inputs
        // ==================================

        if ctx.just_pressed(KeyCode::F3) {
            self.overlay = !self.overlay;
            bns_log::info!(
                "debug overlay: {}",
                if self.overlay { "visible" } else { "hidden" }
            );
        }

        if ctx.just_pressed(KeyCode::F4) {
            self.chunk_state = self.chunk_state.next_state();
            bns_log::info!("debug chunk state: {}", self.chunk_state.name());
        }

        // ==================================
        // Frame time
        // ==================================

        self.accumulated_frame_time += ctx.since_last_tick();
        self.accumulated_frame_count += 1;
        if self.accumulated_frame_time >= Self::FRAME_TIME_THRESHOLD {
            self.average_frame_time = self.accumulated_frame_time / self.accumulated_frame_count;
            self.accumulated_frame_time = Duration::ZERO;
            self.accumulated_frame_count = 0;
        }
    }

    /// Updates the content of the debug overlay buffer.
    pub fn reset_overlay(&mut self) {
        self.overlay_buffer.reset();

        let _ = writeln!(
            self.overlay_buffer,
            "Frame time: {frame_time:?} ({fps:.2} fps)\n",
            frame_time = self.average_frame_time,
            fps = 1.0 / self.average_frame_time.as_secs_f64(),
        );
    }

    /// Returns a mutable reference to the buffer that contains
    /// the debug overlay.
    ///
    /// This function should generally be called after [`reset_overlay`].
    ///
    /// [`reset_overlay`]: Self::reset_overlay
    #[inline]
    pub fn overlay_buffer(&mut self) -> &mut DebugOverlayBuffer {
        &mut self.overlay_buffer
    }

    /// Renders the debug overlay.
    #[profiling::function]
    pub fn render<'res>(&'res mut self, current_chunk: ChunkPos, frame: &mut RenderData<'res>) {
        // If the overlay is enabled, render it.
        if self.overlay {
            // If the overlay buffer has been updated, upload it to the GPU.
            if !self.overlay_buffer.is_empty() {
                self.overlay_gpu_buffer.clear();
                self.overlay_gpu_buffer
                    .extend(self.overlay_buffer.as_slice());
            }

            frame.ui.push(Ui::Text(self.overlay_gpu_buffer.slice()));
        }

        const CHUNK_SIZE: f32 = bns_core::Chunk::SIDE as f32;
        match self.chunk_state {
            DebugChunkState::Hidden => (),
            DebugChunkState::ShowCurrentChunk => {
                push_aabb_lines(
                    &mut frame.lines,
                    current_chunk.as_vec3() * CHUNK_SIZE,
                    current_chunk.as_vec3() * CHUNK_SIZE + Vec3::splat(CHUNK_SIZE),
                    Color::RED,
                    3.0,
                    LineVertexFlags::ABOVE,
                );
            }
            DebugChunkState::ShowAllChunks => {
                const BOUND: i32 = 4;

                for z in -BOUND..=BOUND {
                    for y in -BOUND..=BOUND {
                        for x in -BOUND..=BOUND {
                            let pos = ChunkPos::new(x, y, z);
                            push_aabb_lines(
                                &mut frame.lines,
                                (current_chunk + pos).as_vec3() * CHUNK_SIZE,
                                (current_chunk + pos).as_vec3() * CHUNK_SIZE
                                    + Vec3::splat(CHUNK_SIZE),
                                if pos == ChunkPos::ZERO {
                                    Color::RED
                                } else {
                                    Color::YELLOW
                                },
                                if pos == ChunkPos::ZERO { 3.0 } else { 2.0 },
                                if pos == ChunkPos::ZERO {
                                    LineVertexFlags::ABOVE
                                } else {
                                    LineVertexFlags::empty()
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}

/// A buffer that implements [`std::fmt::Write`] and writes to a
/// collection of [`CharacterInstance`]s.
pub struct DebugOverlayBuffer {
    cursor: CharacterInstanceCursor,
    buffer: Vec<CharacterInstance>,
}

impl DebugOverlayBuffer {
    /// The initial cursor position.
    const INITIAL_CURSOR: CharacterInstanceCursor =
        CharacterInstanceCursor::new(Vec2::new(5.0, 5.0), Vec2::new(16.0, 32.0), Vec2::ZERO);

    /// Creates a new [`DebugOverlayBuffer`] instance.
    fn new() -> Self {
        Self {
            cursor: Self::INITIAL_CURSOR,
            buffer: Vec::new(),
        }
    }

    /// Resets the buffer.
    #[inline]
    fn reset(&mut self) -> &mut Self {
        self.cursor = Self::INITIAL_CURSOR;
        self.buffer.clear();
        self
    }

    /// Returns whether the buffer is empty.
    #[inline]
    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the content of the buffer.
    #[inline]
    fn as_slice(&self) -> &[CharacterInstance] {
        self.buffer.as_slice()
    }

    /// Writes the content of the buffer to the GPU buffer.
    #[inline]
    pub fn write_fmt(&mut self, args: std::fmt::Arguments) -> std::fmt::Result {
        std::fmt::write(self, args)
    }
}

impl std::fmt::Write for DebugOverlayBuffer {
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.buffer.push(self.cursor.advance(c));
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buffer
            .extend(s.chars().map(|c| self.cursor.advance(c)));
        Ok(())
    }
}

/// The state of the debug chunk display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DebugChunkState {
    /// No debug information are displayed.
    Hidden,
    /// Draw the chunk that the camera is currently in.
    ShowCurrentChunk,
    /// Draw the chunk grid.
    ShowAllChunks,
}

impl DebugChunkState {
    /// Returns a string representation of the debug chunk state.
    pub fn name(self) -> &'static str {
        match self {
            Self::Hidden => "hidden",
            Self::ShowCurrentChunk => "show current chunk",
            Self::ShowAllChunks => "show all chunks",
        }
    }

    /// Returns the next state in the cycle.
    pub fn next_state(self) -> Self {
        match self {
            Self::Hidden => Self::ShowCurrentChunk,
            Self::ShowCurrentChunk => Self::ShowAllChunks,
            Self::ShowAllChunks => Self::Hidden,
        }
    }
}

/// Adds a new axis-aligned bounding box to the gizmos list.
pub fn push_aabb_lines(
    lines: &mut Vec<LineInstance>,
    min: Vec3,
    max: Vec3,
    color: Color,
    width: f32,
    flags: LineVertexFlags,
) {
    use glam::vec3;

    let base = LineInstance {
        width,
        flags,
        color,
        start: Vec3::ZERO,
        end: Vec3::ZERO,
    };

    lines.extend_from_slice(&[
        // Lower face
        LineInstance {
            start: vec3(min.x, min.y, min.z),
            end: vec3(max.x, min.y, min.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, min.y, min.z),
            end: vec3(max.x, min.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, min.y, max.z),
            end: vec3(min.x, min.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(min.x, min.y, max.z),
            end: vec3(min.x, min.y, min.z),
            ..base
        },
        // Upper face
        LineInstance {
            start: vec3(min.x, max.y, min.z),
            end: vec3(max.x, max.y, min.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, max.y, min.z),
            end: vec3(max.x, max.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, max.y, max.z),
            end: vec3(min.x, max.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(min.x, max.y, max.z),
            end: vec3(min.x, max.y, min.z),
            ..base
        },
        // Vertical edges
        LineInstance {
            start: vec3(min.x, min.y, min.z),
            end: vec3(min.x, max.y, min.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, min.y, min.z),
            end: vec3(max.x, max.y, min.z),
            ..base
        },
        LineInstance {
            start: vec3(max.x, min.y, max.z),
            end: vec3(max.x, max.y, max.z),
            ..base
        },
        LineInstance {
            start: vec3(min.x, min.y, max.z),
            end: vec3(min.x, max.y, max.z),
            ..base
        },
    ]);
}
