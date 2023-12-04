use bns_core::TextureId;
use bns_render::{TextureAtlasConfig, TextureFormat};

/// Loads the texture atlas from the asset directory.
pub async fn load_texture_atlas() -> TextureAtlasConfig<'static> {
    let mut data = Vec::new();
    let mut count = 0;
    let mut metadata = None;

    for texture_id in TextureId::all() {
        let mut image = load_image(texture_id.file_name()).await;
        image.ensure_srgb();
        image.ensure_rgba();

        match &metadata {
            Some(metadata) => assert_eq!(metadata, &image.metadata),
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
            bns_image::ColorSpace::Unknown => TextureFormat::Rgba8Unorm,
            bns_image::ColorSpace::Linear => TextureFormat::Rgba8Unorm,
        },
    }
}

/// Loads the image from the asset directory.
async fn load_image(asset_path: &str) -> bns_image::Image {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("assets/{}.png", asset_path);
        bns_log::trace!("downloading asset from '{url}'...");
        let data = fetch_api::fetch(&url).await;
        bns_image::Image::load_png(std::io::Cursor::new(data)).unwrap()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = format!("assets/{}.png", asset_path);
        bns_log::trace!("loading asset from '{path}'...");
        bns_image::Image::load_png(std::fs::File::open(path).unwrap()).unwrap()
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
