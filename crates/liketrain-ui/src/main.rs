use gpui::{App, Bounds, UpdateGlobal, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_component::{Root, Theme, ThemeRegistry};
use itertools::Itertools;
use liketrain_core::{
    ControllerConfig, Direction, Route, TrackGeometry, Train, TrainId,
    comm::{SerialControllerHardwareCommunication, SimHardwareCommunication, SimTrain},
    parser::Parser,
};

use crate::{controller::ControllerUiWrapper, layout::Layout, window::ControlsWindow};

mod app_ext;
mod assets;
mod controller;
mod ebula;
mod layout;
mod ui;
mod window;

fn init_logger() {
    #[cfg(debug_assertions)]
    {
        if std::env::var("RUST_LOG").is_err() {
            unsafe { std::env::set_var("RUST_LOG", "debug") }
        }
    }

    env_logger::init();
}

fn main() {
    init_logger();

    let track_dsl = include_str!("../../../resources/track.ltt");
    let track_geo = include_str!("../../../resources/geo.json");

    let track_defs = liketrain_core::parser::parser().parse(track_dsl).unwrap();
    let track_geo: TrackGeometry = serde_json::from_str(track_geo).unwrap();

    let evaluator = liketrain_core::parser::eval::Evaluator::default();

    let mut track = evaluator.evaluate(track_defs).unwrap();
    track.set_geometry(track_geo);

    let layout_json = include_str!("../../../resources/layout.json");
    let layout: Layout = serde_json::from_str(layout_json).unwrap();

    let resolved_layout = layout.resolve(&track);

    log::info!("layout: {:#?}", resolved_layout);

    let test_route = Route::new(
        "RE5",
        [12_usize, 14, 16, 9, 10, 12],
        Direction::Backward,
        &track,
    )
    .unwrap();

    let test_sim_train = SimTrain::from_route(&test_route, &track, 8.0);
    let test_train = Train::from_route("RE5", test_route);

    let controller_config = ControllerConfig {
        track,
        trains: [(TrainId::new(1), test_train.clone())]
            .into_iter()
            .collect(),
    };

    // let hardware_comm = SerialControllerHardwareCommunication::new("/dev/cu.usbmodem11401", 115200);
    let hardware_comm = SimHardwareCommunication::new([test_sim_train]);

    gpui_platform::application()
        .with_assets(assets::Assets)
        .run(move |cx: &mut App| {
            let controller = ControllerUiWrapper::new(cx, controller_config, hardware_comm)
                .with_layout(resolved_layout);

            cx.set_global(controller);

            assets::init(cx).unwrap();
            gpui_component::init(cx);

            let theme_reg = ThemeRegistry::global(cx);
            log::debug!(
                "Found {} themes ({})",
                theme_reg.themes().len(),
                theme_reg.themes().keys().join(", ")
            );

            let selected_theme = "Default Dark";

            let theme = theme_reg
                .themes()
                .iter()
                .find_map(|(name, theme)| {
                    if name == selected_theme {
                        Some(theme)
                    } else {
                        None
                    }
                })
                .cloned();

            let Some(theme) = theme else {
                log::warn!("{} theme not found", selected_theme);
                return;
            };

            Theme::update_global(cx, |active_theme, _| active_theme.apply_config(&theme));

            log::debug!("fonts: {:#?}", cx.text_system().all_font_names());

            let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    window.set_window_title("liketrain - Controls");
                    let view = cx.new(|cx| ControlsWindow::new(window, cx));

                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .unwrap();
        });
}
