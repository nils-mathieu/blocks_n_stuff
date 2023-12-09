use std::sync::Arc;

use bns_core::{BlockId, TextureId};
use bns_render::{Gpu, Texture, TextureAtlasConfig, TextureFormat};
use bns_rng::{DefaultRng, Rng};

/// Loads the texture atlas from the asset directory.
pub async fn load_texture_atlas() -> TextureAtlasConfig<'static> {
    let mut data = Vec::new();
    let mut count = 0;
    let mut metadata: Option<bns_image::ImageMetadata> = None;

    for texture_id in TextureId::all() {
        let mut image = load_image(texture_id.file_name()).await;
        image.ensure_rgba();

        match &metadata {
            Some(metadata) => {
                if metadata.color_space != image.metadata.color_space {
                    bns_log::warning!(
                        "texture {:?} does not have the same color space: {:?} != {:?}",
                        texture_id,
                        metadata.color_space,
                        image.metadata.color_space,
                    );

                    if image.metadata.width != metadata.width
                        || image.metadata.height != metadata.height
                    {
                        panic!("texture atlas: mismatched texture dimensions");
                    }
                }
            }
            None => metadata = Some(image.metadata),
        }

        data.extend_from_slice(&image.pixels);
        count += 1;
    }

    let metadata = metadata.unwrap();

    TextureAtlasConfig {
        data: data.into(),
        width: metadata.width,
        height: metadata.height,
        count,
        mip_level_count: 1,
        format: match metadata.color_space {
            bns_image::ColorSpace::Srgb => TextureFormat::Rgba8UnormSrgb,
            bns_image::ColorSpace::Unknown => TextureFormat::Rgba8UnormSrgb,
            bns_image::ColorSpace::Linear => TextureFormat::Rgba8Unorm,
        },
    }
}

/// Contains all the loaded assets.
pub struct Assets {
    /// The texture that contains UI elements.
    pub ui: Texture,
}

impl Assets {
    /// Loads the assets.
    pub async fn load(gpu: &Gpu) -> Self {
        Self {
            ui: load_texture(gpu, "ui").await,
        }
    }
}

/// Loads the provided texture.
async fn load_texture(gpu: &Gpu, asset_path: &str) -> Texture {
    let mut image = load_image(asset_path).await;
    image.ensure_rgba();
    Texture::new(
        gpu,
        image.metadata.width,
        image.metadata.height,
        match image.metadata.color_space {
            bns_image::ColorSpace::Srgb => TextureFormat::Rgba8UnormSrgb,
            bns_image::ColorSpace::Unknown => TextureFormat::Rgba8UnormSrgb,
            bns_image::ColorSpace::Linear => TextureFormat::Rgba8Unorm,
        },
        &image.pixels,
    )
}

/// Loads the image from the asset directory.
async fn load_image(asset_path: &str) -> bns_image::Image {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("assets/{}.png", asset_path);
        bns_log::trace!("downloading image from '{url}'...");
        let data = fetch_api::fetch(&url).await;
        bns_image::Image::load_png(std::io::Cursor::new(data)).unwrap()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = format!("assets/{}.png", asset_path);
        bns_log::trace!("loading image from '{path}'...");
        bns_image::Image::load_png(std::fs::File::open(path).unwrap()).unwrap()
    }
}

/// The loaded sounds.
pub struct Sounds {
    pub background_music: Arc<[u8]>,

    pub step_sand1: Arc<[u8]>,
    pub step_sand2: Arc<[u8]>,
    pub step_sand3: Arc<[u8]>,
    pub step_grass1: Arc<[u8]>,
    pub step_grass2: Arc<[u8]>,
    pub step_grass3: Arc<[u8]>,
    pub step_stone1: Arc<[u8]>,
    pub step_stone2: Arc<[u8]>,
    pub step_stone3: Arc<[u8]>,
    pub step_wood1: Arc<[u8]>,
    pub step_wood2: Arc<[u8]>,
    pub step_wood3: Arc<[u8]>,

    pub break_sand1: Arc<[u8]>,
    pub break_sand2: Arc<[u8]>,
    pub break_sand3: Arc<[u8]>,
    pub break_grass1: Arc<[u8]>,
    pub break_grass2: Arc<[u8]>,
    pub break_grass3: Arc<[u8]>,
    pub break_stone1: Arc<[u8]>,
    pub break_stone2: Arc<[u8]>,
    pub break_stone3: Arc<[u8]>,
    pub break_wood1: Arc<[u8]>,
    pub break_wood2: Arc<[u8]>,
    pub break_wood3: Arc<[u8]>,
    pub break_glass1: Arc<[u8]>,
    pub break_glass2: Arc<[u8]>,
    pub break_glass3: Arc<[u8]>,
    pub break_gravel1: Arc<[u8]>,
    pub break_gravel2: Arc<[u8]>,
    pub break_gravel3: Arc<[u8]>,
}

impl Sounds {
    /// Loads the sounds asynchronously.
    pub async fn load() -> Self {
        Self {
            background_music: Arc::from(load_sound("background_music.ogg").await),
            step_grass1: Arc::from(load_sound("step_grass1.ogg").await),
            step_grass2: Arc::from(load_sound("step_grass2.ogg").await),
            step_grass3: Arc::from(load_sound("step_grass3.ogg").await),
            step_sand1: Arc::from(load_sound("step_sand1.ogg").await),
            step_sand2: Arc::from(load_sound("step_sand2.ogg").await),
            step_sand3: Arc::from(load_sound("step_sand3.ogg").await),
            step_stone1: Arc::from(load_sound("step_stone1.ogg").await),
            step_stone2: Arc::from(load_sound("step_stone2.ogg").await),
            step_stone3: Arc::from(load_sound("step_stone3.ogg").await),
            step_wood1: Arc::from(load_sound("step_wood1.ogg").await),
            step_wood2: Arc::from(load_sound("step_wood2.ogg").await),
            step_wood3: Arc::from(load_sound("step_wood3.ogg").await),
            break_grass1: Arc::from(load_sound("break_grass1.ogg").await),
            break_grass2: Arc::from(load_sound("break_grass2.ogg").await),
            break_grass3: Arc::from(load_sound("break_grass3.ogg").await),
            break_sand1: Arc::from(load_sound("break_sand1.ogg").await),
            break_sand2: Arc::from(load_sound("break_sand2.ogg").await),
            break_sand3: Arc::from(load_sound("break_sand3.ogg").await),
            break_stone1: Arc::from(load_sound("break_stone1.ogg").await),
            break_stone2: Arc::from(load_sound("break_stone2.ogg").await),
            break_stone3: Arc::from(load_sound("break_stone3.ogg").await),
            break_wood1: Arc::from(load_sound("break_wood1.ogg").await),
            break_wood2: Arc::from(load_sound("break_wood2.ogg").await),
            break_wood3: Arc::from(load_sound("break_wood3.ogg").await),
            break_glass1: Arc::from(load_sound("break_glass1.ogg").await),
            break_glass2: Arc::from(load_sound("break_glass2.ogg").await),
            break_glass3: Arc::from(load_sound("break_glass3.ogg").await),
            break_gravel1: Arc::from(load_sound("break_gravel1.ogg").await),
            break_gravel2: Arc::from(load_sound("break_gravel2.ogg").await),
            break_gravel3: Arc::from(load_sound("break_gravel3.ogg").await),
        }
    }

    /// Returns the sound that must be played when the player breaks the given block.
    pub fn get_sound_for_block_break(&self, block: BlockId, rng: &mut DefaultRng) -> Arc<[u8]> {
        match block {
            BlockId::Grass
            | BlockId::Podzol
            | BlockId::Dirt
            | BlockId::OakLeaves
            | BlockId::PineLeaves
            | BlockId::Clay => match rng.next_u32() % 3 {
                0 => self.break_grass1.clone(),
                1 => self.break_grass2.clone(),
                2 => self.break_grass3.clone(),
                _ => unreachable!(),
            },
            BlockId::Sand => match rng.next_u32() % 3 {
                0 => self.break_sand1.clone(),
                1 => self.break_sand2.clone(),
                2 => self.break_sand3.clone(),
                _ => unreachable!(),
            },
            BlockId::OakLog | BlockId::PineLog | BlockId::OakPlanks | BlockId::PinePlanks => {
                match rng.next_u32() % 3 {
                    0 => self.break_wood1.clone(),
                    1 => self.break_wood2.clone(),
                    2 => self.break_wood3.clone(),
                    _ => unreachable!(),
                }
            }
            BlockId::Gravel => match rng.next_u32() % 3 {
                0 => self.break_gravel1.clone(),
                1 => self.break_gravel2.clone(),
                2 => self.break_gravel3.clone(),
                _ => unreachable!(),
            },
            BlockId::Glass => match rng.next_u32() % 3 {
                0 => self.break_glass1.clone(),
                1 => self.break_glass2.clone(),
                2 => self.break_glass3.clone(),
                _ => unreachable!(),
            },
            _ => match rng.next_u32() % 3 {
                0 => self.break_stone1.clone(),
                1 => self.break_stone2.clone(),
                2 => self.break_stone3.clone(),
                _ => unreachable!(),
            },
        }
    }

    /// Returns the sound that must be played when the player steps on the given block.
    pub fn get_sound_for_block_step(&self, block: BlockId, rng: &mut DefaultRng) -> Arc<[u8]> {
        match block {
            BlockId::Grass
            | BlockId::Podzol
            | BlockId::Dirt
            | BlockId::OakLeaves
            | BlockId::Daffodil
            | BlockId::PineLeaves
            | BlockId::Clay => match rng.next_u32() % 3 {
                0 => self.step_grass1.clone(),
                1 => self.step_grass2.clone(),
                2 => self.step_grass3.clone(),
                _ => unreachable!(),
            },
            BlockId::Sand | BlockId::Gravel => match rng.next_u32() % 3 {
                0 => self.step_sand1.clone(),
                1 => self.step_sand2.clone(),
                2 => self.step_sand3.clone(),
                _ => unreachable!(),
            },
            BlockId::OakLog | BlockId::PineLog | BlockId::OakPlanks | BlockId::PinePlanks => {
                match rng.next_u32() % 3 {
                    0 => self.step_wood1.clone(),
                    1 => self.step_wood2.clone(),
                    2 => self.step_wood3.clone(),
                    _ => unreachable!(),
                }
            }
            _ => match rng.next_u32() % 3 {
                0 => self.step_stone1.clone(),
                1 => self.step_stone2.clone(),
                2 => self.step_stone3.clone(),
                _ => unreachable!(),
            },
        }
    }
}

/// Loads the sounds from the asset directory.
async fn load_sound(asset_path: &str) -> Vec<u8> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("assets/{}", asset_path);
        bns_log::trace!("downloading sound from '{url}'...");
        fetch_api::fetch(&url).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = format!("assets/{}", asset_path);
        bns_log::trace!("loading sound from '{path}'...");
        std::fs::read(path).unwrap()
    }
}

#[cfg(target_arch = "wasm32")]
mod fetch_api {
    use wasm_bindgen_futures::js_sys::{ArrayBuffer, Uint8Array};
    use wasm_bindgen_futures::wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Blob, Response};

    /// Fetches the data from the network.
    pub async fn fetch(url: &str) -> Vec<u8> {
        let window = web_sys::window().unwrap();
        let response = JsFuture::from(window.fetch_with_str(url))
            .await
            .unwrap()
            .unchecked_into::<Response>();
        let blob = JsFuture::from(response.blob().unwrap())
            .await
            .unwrap()
            .unchecked_into::<Blob>();
        let array = JsFuture::from(blob.array_buffer())
            .await
            .unwrap()
            .unchecked_into::<ArrayBuffer>();
        let array = Uint8Array::new(&array);
        array.to_vec()
    }
}
