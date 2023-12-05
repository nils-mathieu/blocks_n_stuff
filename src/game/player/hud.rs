use std::sync::Arc;

use bns_app::{Ctx, KeyCode};
use bns_core::BlockId;
use bns_render::data::{RenderData, Sprite, Ui};
use bns_render::{DynamicVertexBuffer, Gpu};

use glam::Vec2;

use crate::assets::Assets;

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
    /// The buffer that contains the instances of the UI elements.
    instances: DynamicVertexBuffer<Sprite>,

    /// The index of the currently selected hotbar slot.
    hotbar_slot: usize,

    /// The materials that are currently available in the hotbar.
    materials: [Option<BlockId>; HOTBAR_SLOT_COUNT],
}

impl Hud {
    /// Creates a new [`Hud`] instance.
    pub fn new(gpu: Arc<Gpu>) -> Self {
        Self {
            hotbar_slot: 0,
            instances: DynamicVertexBuffer::new_with_data(gpu, &[Sprite::dummy(); 3]),
            materials: [
                Some(BlockId::Dirt),
                Some(BlockId::Grass),
                Some(BlockId::Cobblestone),
                Some(BlockId::OakLog),
                Some(BlockId::OakLeaves),
                Some(BlockId::PineLog),
                Some(BlockId::PineLeaves),
                None,
                None,
            ],
        }
    }

    /// Returns the material currently selected in the hotbar.
    #[inline]
    pub fn current_material(&self) -> Option<BlockId> {
        self.materials[self.hotbar_slot]
    }

    /// Rebuilds the UI.
    pub fn rebuild_ui(&mut self, width: u32, height: u32) {
        let hotbar_anchor = Vec2::new(width as f32 / 2.0, height as f32);
        let crosshair_anchor = Vec2::new(width as f32 / 2.0, height as f32 / 2.0);

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
                            + Vec2::X * HOTBAR_SLOT_WIDTH * self.hotbar_slot as f32,
                        HOTBAR_CURSOR_SCREEN_SIZE,
                    ),
                Sprite::dummy()
                    .with_uv_rect(CROSSHAIR_POS, CROSSHAIR_SIZE)
                    .with_rect(
                        crosshair_anchor - CROSSHAIR_SCREEN_SIZE / 2.0,
                        CROSSHAIR_SCREEN_SIZE,
                    ),
            ],
        );
    }

    /// Ticks the HUD.
    pub fn tick(&mut self, ctx: &mut Ctx) {
        if ctx.just_resized() {
            self.rebuild_ui(ctx.width(), ctx.height());
        }

        if ctx.mouse_scroll_y() > 0.0 {
            if self.hotbar_slot == 0 {
                self.hotbar_slot = HOTBAR_SLOT_COUNT - 1;
            } else {
                self.hotbar_slot -= 1;
            }

            self.rebuild_ui(ctx.width(), ctx.height());
        } else if ctx.mouse_scroll_y() < 0.0 {
            if self.hotbar_slot == HOTBAR_SLOT_COUNT - 1 {
                self.hotbar_slot = 0;
            } else {
                self.hotbar_slot += 1;
            }

            self.rebuild_ui(ctx.width(), ctx.height());
        }

        if ctx.just_pressed(KeyCode::Digit1) {
            self.hotbar_slot = 0;
            self.rebuild_ui(ctx.width(), ctx.height());
        } else if ctx.just_pressed(KeyCode::Digit2) {
            self.hotbar_slot = 1;
            self.rebuild_ui(ctx.width(), ctx.height());
        } else if ctx.just_pressed(KeyCode::Digit3) {
            self.hotbar_slot = 2;
            self.rebuild_ui(ctx.width(), ctx.height());
        } else if ctx.just_pressed(KeyCode::Digit4) {
            self.hotbar_slot = 3;
            self.rebuild_ui(ctx.width(), ctx.height());
        } else if ctx.just_pressed(KeyCode::Digit5) {
            self.hotbar_slot = 4;
            self.rebuild_ui(ctx.width(), ctx.height());
        } else if ctx.just_pressed(KeyCode::Digit6) {
            self.hotbar_slot = 5;
            self.rebuild_ui(ctx.width(), ctx.height());
        } else if ctx.just_pressed(KeyCode::Digit7) {
            self.hotbar_slot = 6;
            self.rebuild_ui(ctx.width(), ctx.height());
        } else if ctx.just_pressed(KeyCode::Digit8) {
            self.hotbar_slot = 7;
            self.rebuild_ui(ctx.width(), ctx.height());
        } else if ctx.just_pressed(KeyCode::Digit9) {
            self.hotbar_slot = 8;
            self.rebuild_ui(ctx.width(), ctx.height());
        }
    }

    /// Renders the HUD.
    pub fn render<'res>(&'res self, assets: &'res Assets, frame: &mut RenderData<'res>) {
        frame.ui.push(Ui::Sprite {
            instances: self.instances.slice(),
            texture: &assets.ui,
        })
    }
}
