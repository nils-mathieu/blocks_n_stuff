use std::sync::Arc;

use bns_app::Ctx;
use bns_render::data::{RenderData, Sprite, Ui};
use bns_render::{DynamicVertexBuffer, Gpu, Texture, TextureFormat};

use glam::Vec2;

const BASE_TEXTURE_SIZE: Vec2 = Vec2::new(182.0, 45.0);
const UI_SCALE: Vec2 = Vec2::new(400.0, 400.0 * BASE_TEXTURE_SIZE.y / BASE_TEXTURE_SIZE.x);

const HOTBAR_BACKGROUND_POS: Vec2 =
    Vec2::new(0.0 / BASE_TEXTURE_SIZE.x, 23.0 / BASE_TEXTURE_SIZE.y);
const HOTBAR_BACKGROUND_SIZE: Vec2 =
    Vec2::new(182.0 / BASE_TEXTURE_SIZE.x, 22.0 / BASE_TEXTURE_SIZE.y);
const HOTBAR_CURSOR_POS: Vec2 = Vec2::new(0.0, 0.0);
const HOTBAR_CURSOR_SIZE: Vec2 = Vec2::new(22.0 / BASE_TEXTURE_SIZE.x, 23.0 / BASE_TEXTURE_SIZE.y);
const CROSSHAIR_POS: Vec2 = Vec2::new(22.0 / BASE_TEXTURE_SIZE.x, 0.0 / BASE_TEXTURE_SIZE.y);
const CROSSHAIR_SIZE: Vec2 = Vec2::new(7.0 / BASE_TEXTURE_SIZE.x, 7.0 / BASE_TEXTURE_SIZE.y);

const HOTBAR_SLOT_COUNT: usize = 9;

const HOTBAR_PADDING_BOTTOM: f32 = 16.0;
const HOTBAR_BACKGROUND_SCREEN_SIZE: Vec2 = Vec2::new(
    HOTBAR_BACKGROUND_SIZE.x * UI_SCALE.x,
    HOTBAR_BACKGROUND_SIZE.y * UI_SCALE.y,
);
const HOTBAR_CURSOR_SCREEN_SIZE: Vec2 = Vec2::new(
    HOTBAR_CURSOR_SIZE.x * UI_SCALE.x,
    HOTBAR_CURSOR_SIZE.y * UI_SCALE.y,
);
const CROSSHAIR_SCREEN_SIZE: Vec2 =
    Vec2::new(CROSSHAIR_SIZE.x * UI_SCALE.x, CROSSHAIR_SIZE.y * UI_SCALE.y);
const HOTBAR_SLOT_WIDTH: f32 = HOTBAR_BACKGROUND_SCREEN_SIZE.x / HOTBAR_SLOT_COUNT as f32;

/// The Head-Up Display (HUD) of the player.
pub struct Hud {
    /// The texture that contains all UI elements of the head-up display.
    ui_texture: Texture,
    /// The buffer that contains the instances of the UI elements.
    instances: DynamicVertexBuffer<Sprite>,

    /// The index of the currently selected hotbar slot.
    hotbar_slot: usize,
}

impl Hud {
    /// Creates a new [`Hud`] instance.
    pub fn new(gpu: Arc<Gpu>) -> Self {
        bns_log::trace!("loading texture 'assets/ui.png'...");
        let mut texture =
            bns_image::Image::load_png(std::fs::File::open("assets/ui.png").unwrap()).unwrap();
        texture.ensure_rgba();
        texture.ensure_srgb();
        let ui_texture = Texture::new(
            &gpu,
            texture.metadata.width,
            texture.metadata.height,
            TextureFormat::Rgba8UnormSrgb,
            &texture.pixels,
        );

        Self {
            hotbar_slot: 0,
            ui_texture,
            instances: DynamicVertexBuffer::new_with_data(gpu, &[Sprite::dummy(); 3]),
        }
    }

    /// Ticks the HUD.
    pub fn tick(&mut self, ctx: &mut Ctx) {
        if ctx.just_resized() {
            let (width, height) = ctx.size();
            let hotbar_anchor = Vec2::new(width as f32 / 2.0, height as f32);
            let crosshair_anchor = Vec2::new(width as f32 / 2.0, height as f32 / 2.0);

            // We need to place the hotbar in the center of the screen.
            self.instances.edit(
                0,
                &[
                    Sprite::dummy()
                        .with_uv_rect(HOTBAR_BACKGROUND_POS, HOTBAR_BACKGROUND_SIZE)
                        .with_rect(
                            hotbar_anchor
                                - Vec2::new(
                                    HOTBAR_BACKGROUND_SCREEN_SIZE.x / 2.0,
                                    HOTBAR_BACKGROUND_SCREEN_SIZE.y + HOTBAR_PADDING_BOTTOM,
                                ),
                            HOTBAR_BACKGROUND_SCREEN_SIZE,
                        ),
                    Sprite::dummy()
                        .with_uv_rect(HOTBAR_CURSOR_POS, HOTBAR_CURSOR_SIZE)
                        .with_rect(
                            hotbar_anchor
                                - Vec2::new(
                                    HOTBAR_BACKGROUND_SCREEN_SIZE.x / 2.0,
                                    HOTBAR_BACKGROUND_SCREEN_SIZE.y + HOTBAR_PADDING_BOTTOM,
                                )
                                + HOTBAR_SLOT_WIDTH * self.hotbar_slot as f32,
                            HOTBAR_CURSOR_SCREEN_SIZE,
                        ),
                    Sprite::dummy()
                        .with_uv_rect(CROSSHAIR_POS, CROSSHAIR_SIZE)
                        .with_rect(
                            crosshair_anchor - CROSSHAIR_SCREEN_SIZE / 2.0,
                            CROSSHAIR_SCREEN_SIZE,
                        ),
                ],
            )
        }
    }

    /// Renders the HUD.
    pub fn render<'res>(&'res self, frame: &mut RenderData<'res>) {
        frame.ui.push(Ui::Sprite {
            instances: self.instances.slice(),
            texture: &self.ui_texture,
        })
    }
}
