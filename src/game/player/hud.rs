use std::sync::Arc;

use bns_app::{Ctx, KeyCode};
use bns_core::{BlockAppearance, BlockId, TextureId};
use bns_render::data::{AtlasSprite, Color, RenderData, Sprite, Ui};
use bns_render::{DynamicVertexBuffer, Gpu};

use bytemuck::Contiguous;
use glam::{Mat2, Vec2};

use crate::assets::Assets;

/// The size of the base texture where all UI elements are stored.
const BASE_TEXTURE_SIZE: Vec2 = Vec2::new(182.0, 45.0);

/// The pixel position of the hotbar's background in the base texture.
const HOTBAR_BACKGROUND_PX_POS: Vec2 = Vec2::new(0.0, 23.0);
/// The size of the hotbar's background in pixels in the base texture.
const HOTBAR_BACKGROUND_PX_SIZE: Vec2 = Vec2::new(182.0, 22.0);
/// The pixel position of the hotbar's cursor in the base texture.
const HOTBAR_CURSOR_PX_POS: Vec2 = Vec2::new(0.0, 0.0);
/// The size of the hotbar's cursor in pixels in the base texture.
const HOTBAR_CURSOR_PX_SIZE: Vec2 = Vec2::new(22.0, 23.0);
/// The position of the crosshair in the base texture.
const CROSSHAIR_PX_POS: Vec2 = Vec2::new(22.0, 0.0);
/// The size of the crosshair in pixels in the base texture.
const CROSSHAIR_PX_SIZE: Vec2 = Vec2::new(7.0, 7.0);

/// A constant multiplier that's used to scale the UI.
const UI_SCALE: f32 = 2.0;

/// The size of the hotbar on screen.
const HOTBAR_BACKGROUND_SIZE: Vec2 = Vec2::new(
    HOTBAR_BACKGROUND_PX_SIZE.x * UI_SCALE,
    HOTBAR_BACKGROUND_PX_SIZE.y * UI_SCALE,
);

/// The padding between the bottom of the screen and the hotbar.
const HOTBAR_PADDING_BOTTOM: f32 = 16.0;

/// Based on the hotbar's anchor (the bottom center of the screen), the offset that the hotbar's
/// background has.
const HOTBAR_BACKGROUND_POS: Vec2 = Vec2::new(
    -HOTBAR_BACKGROUND_SIZE.x / 2.0,
    -HOTBAR_BACKGROUND_SIZE.y - HOTBAR_PADDING_BOTTOM,
);

/// The size of the cursor, on screen.
const HOTBAR_CURSOR_SIZE: Vec2 = Vec2::new(
    HOTBAR_CURSOR_PX_SIZE.x * UI_SCALE,
    HOTBAR_CURSOR_PX_SIZE.y * UI_SCALE,
);

/// The offset of the cursor based on the hotbar's anchor.
const HOTBAR_CURSOR_OFFSET: Vec2 = Vec2::new(HOTBAR_BACKGROUND_POS.x, HOTBAR_BACKGROUND_POS.y);

/// The size of icons in the hotbar.
const HOTBAR_ICON_SIZE: f32 = HOTBAR_CURSOR_SIZE.x - 8.0 * UI_SCALE;

/// The offset of icons in the hotbar, based on the hotbar's anchor.
const HOTBAR_ICON_OFFSET: Vec2 = Vec2::new(
    HOTBAR_BACKGROUND_POS.x + HOTBAR_CURSOR_SIZE.x / 2.0 - HOTBAR_ICON_SIZE / 2.0,
    HOTBAR_BACKGROUND_POS.y + HOTBAR_CURSOR_SIZE.y / 2.0 - HOTBAR_ICON_SIZE / 2.0,
);

/// The size of the crosshair on screen.
const CROSSHAIR_SIZE: Vec2 = Vec2::new(
    CROSSHAIR_PX_SIZE.x * UI_SCALE,
    CROSSHAIR_PX_SIZE.y * UI_SCALE,
);
/// The offset of the crosshair from the center of the screen.
const CROSSHAIR_POS: Vec2 = Vec2::new(-CROSSHAIR_SIZE.x / 2.0, -CROSSHAIR_SIZE.y / 2.0);

/// The total number of slots available in the hotbar.
const HOTBAR_SLOT_COUNT: usize = 9;

/// The size of a single slot in the hotbar.
const HOTBAR_CURSOR_STEP: f32 = HOTBAR_CURSOR_SIZE.x - UI_SCALE * 2.0;

/// The Head-Up Display (HUD) of the player.
pub struct Hud {
    instances: DynamicVertexBuffer<Sprite>,
    icons: DynamicVertexBuffer<AtlasSprite>,

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
            instances: DynamicVertexBuffer::new_with_data(gpu.clone(), &[Sprite::dummy(); 3]),
            icons: DynamicVertexBuffer::new_with_data(gpu, &[AtlasSprite::dummy(); 9 * 3]),
            materials: [
                Some(BlockId::Dirt),
                Some(BlockId::Grass),
                Some(BlockId::Cobblestone),
                Some(BlockId::OakLog),
                Some(BlockId::OakLeaves),
                Some(BlockId::PineLog),
                Some(BlockId::PineLeaves),
                Some(BlockId::StructureBlock),
                Some(BlockId::StructureOriginBlock),
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
                    .with_uv_rect(
                        HOTBAR_BACKGROUND_PX_POS / BASE_TEXTURE_SIZE,
                        HOTBAR_BACKGROUND_PX_SIZE / BASE_TEXTURE_SIZE,
                    )
                    .with_rect(
                        hotbar_anchor + HOTBAR_BACKGROUND_POS,
                        HOTBAR_BACKGROUND_SIZE,
                    ),
                Sprite::dummy()
                    .with_uv_rect(
                        HOTBAR_CURSOR_PX_POS / BASE_TEXTURE_SIZE,
                        HOTBAR_CURSOR_PX_SIZE / BASE_TEXTURE_SIZE,
                    )
                    .with_rect(
                        hotbar_anchor
                            + HOTBAR_CURSOR_OFFSET
                            + Vec2::X * self.hotbar_slot as f32 * HOTBAR_CURSOR_STEP,
                        HOTBAR_CURSOR_SIZE,
                    ),
                Sprite::dummy()
                    .with_uv_rect(
                        CROSSHAIR_PX_POS / BASE_TEXTURE_SIZE,
                        CROSSHAIR_PX_SIZE / BASE_TEXTURE_SIZE,
                    )
                    .with_rect(crosshair_anchor + CROSSHAIR_POS, CROSSHAIR_SIZE),
            ],
        );

        let mut icons = [AtlasSprite::dummy(); HOTBAR_SLOT_COUNT * 3];
        let mut cur = 0;
        for i in 0..HOTBAR_SLOT_COUNT {
            if let Some(material) = self.materials[i] {
                let (top, _bottom, side) = get_icon_textures(material);

                icons[cur] = AtlasSprite {
                    color: Color::WHITE,
                    position: hotbar_anchor
                        + HOTBAR_ICON_OFFSET
                        + Vec2::X * i as f32 * HOTBAR_CURSOR_STEP
                        + Vec2::Y * HOTBAR_ICON_SIZE * 0.25,
                    texture_id: top as u32,
                    transform: Mat2::from_cols_array(&[
                        HOTBAR_ICON_SIZE * 0.5,
                        -0.25 * HOTBAR_ICON_SIZE,
                        HOTBAR_ICON_SIZE * 0.5,
                        HOTBAR_ICON_SIZE * 0.25,
                    ]),
                };
                cur += 1;
                icons[cur] = AtlasSprite {
                    color: Color::rgb(50, 50, 50),
                    position: hotbar_anchor
                        + HOTBAR_ICON_OFFSET
                        + Vec2::X * i as f32 * HOTBAR_CURSOR_STEP
                        + Vec2::splat(HOTBAR_ICON_SIZE * 0.5),
                    texture_id: side as u32,
                    transform: Mat2::from_cols_array(&[
                        HOTBAR_ICON_SIZE * 0.5,
                        -0.25 * HOTBAR_ICON_SIZE,
                        0.0,
                        HOTBAR_ICON_SIZE * 0.5,
                    ]),
                };
                cur += 1;
                icons[cur] = AtlasSprite {
                    color: Color::rgb(100, 100, 100),
                    position: hotbar_anchor
                        + HOTBAR_ICON_OFFSET
                        + Vec2::X * i as f32 * HOTBAR_CURSOR_STEP
                        + Vec2::new(0.0, HOTBAR_ICON_SIZE * 0.25),
                    texture_id: side as u32,
                    transform: Mat2::from_cols_array(&[
                        HOTBAR_ICON_SIZE * 0.5,
                        0.25 * HOTBAR_ICON_SIZE,
                        0.0,
                        HOTBAR_ICON_SIZE * 0.5,
                    ]),
                };
                cur += 1;
            }
        }

        while cur < icons.len() {
            icons[cur] = AtlasSprite::dummy();
            cur += 1;
        }

        self.icons.edit(0, &icons);
    }

    /// Ticks the HUD.
    pub fn tick(&mut self, ctx: &mut Ctx) {
        if ctx.just_resized() {
            self.rebuild_ui(ctx.width(), ctx.height());
        }

        if ctx.just_pressed(KeyCode::KeyV) {
            self.materials[self.hotbar_slot] = Some(next_material(
                self.materials[self.hotbar_slot].unwrap_or(BlockId::Air),
            ));
            self.rebuild_ui(ctx.width(), ctx.height());
        }

        if ctx.just_pressed(KeyCode::KeyC) {
            self.materials[self.hotbar_slot] = Some(previous_material(
                self.materials[self.hotbar_slot].unwrap_or(BlockId::Air),
            ));
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
        });
        frame.ui.push(Ui::AtlasSprite(self.icons.slice()));
    }
}

fn get_icon_textures(block: BlockId) -> (TextureId, TextureId, TextureId) {
    match block.info().appearance {
        BlockAppearance::Flat(tex) => (tex, tex, tex),
        BlockAppearance::Liquid(liq) => (liq, liq, liq),
        BlockAppearance::Regular { top, bottom, side } => (top, bottom, side),
        BlockAppearance::Invisible => (TextureId::Bedrock, TextureId::Bedrock, TextureId::Bedrock),
    }
}

fn next_material(block: BlockId) -> BlockId {
    let mut new_id = block as <BlockId as Contiguous>::Int + 1;
    if new_id > BlockId::MAX_VALUE {
        new_id = BlockId::MIN_VALUE + 1; // + 1 because we're skipping air
    }
    BlockId::from_integer(new_id).unwrap()
}

fn previous_material(block: BlockId) -> BlockId {
    let id = block as <BlockId as Contiguous>::Int;

    if id == 0 {
        return BlockId::from_integer(BlockId::MAX_VALUE).unwrap();
    }

    let mut new_id = block as <BlockId as Contiguous>::Int - 1;
    if new_id == BlockId::MIN_VALUE {
        new_id = BlockId::MAX_VALUE;
    }
    BlockId::from_integer(new_id).unwrap()
}
