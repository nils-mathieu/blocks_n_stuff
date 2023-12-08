//! This module contains the game-related logic, aggregating all the other modules.

use bns_app::{App, KeyCode, MouseButton};
use bns_render::data::RenderData;
use bns_render::{Renderer, RendererConfig, Surface};

use crate::game::Game;

/// Runs the application until completion.
pub fn run() {
    // On web, we need everything to be executed by the browser's executor (because of some
    // internals of WebGPU).
    //
    // Depending on the target platform, the `run_async` function will either be executed
    // by the browser's executor, or by a dummy runtime.
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(run_async());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run_async());
    }
}

async fn run_async() {
    let app = App::new(bns_app::Config {
        title: "Blocks 'n Stuff",
        min_size: (300, 300),
        fullscreen: false,

        // On platform other than web, we require a specific window size that's decently
        // large. But on web, we leave the css do that for us.
        #[cfg(not(target_arch = "wasm32"))]
        size: Some((1280, 720)),
        #[cfg(target_arch = "wasm32")]
        size: None,
    });

    let mut surface = Surface::new(app.opaque_window()).await;
    let assets = crate::assets::Assets::load(surface.gpu()).await;
    let sounds = crate::assets::Sounds::load().await;
    let mut renderer = Renderer::new(
        surface.gpu().clone(),
        RendererConfig {
            output_format: surface.info().format,
        },
    );
    renderer
        .gpu()
        .set_texture_atlas(&crate::assets::load_texture_atlas().await);
    let mut render_data = Some(RenderData::new(surface.gpu()));

    let mut game = Game::new(surface.gpu().clone(), &sounds);

    app.run(|ctx| {
        // ==============================================
        // Update
        // ==============================================

        if ctx.just_resized() {
            surface.config_mut().width = ctx.width();
            surface.config_mut().height = ctx.height();
            renderer.gpu().notify_resized(ctx.width(), ctx.height());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if ctx.just_pressed(KeyCode::Escape) {
                ctx.close();
                return;
            }
        }

        if ctx.just_pressed(KeyCode::F11) {
            ctx.set_fullscreen(!ctx.fullscreen());
        }

        if ctx.just_pressed(MouseButton::Left) {
            ctx.grab_cursor();
        }

        if ctx.focus_just_changed() && ctx.focused() {
            ctx.release_cursor();
        }

        game.tick(ctx, &sounds);

        // ==============================================
        // Rendering
        // ==============================================

        let Some(frame) = surface.acquire_image() else {
            return;
        };

        let mut data = render_data.take().unwrap();
        game.render(ctx, &assets, &mut data);
        renderer.render(frame.target(), &mut data);
        frame.present();
        render_data = Some(data.reset());

        profiling::finish_frame!();
    });
}
