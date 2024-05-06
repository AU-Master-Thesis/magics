use bevy::{prelude::*, render::view::screenshot::ScreenshotManager, window::PrimaryWindow};
use bevy_notify::ToastEvent;
use image::ImageFormat;

use crate::bevy_utils::run_conditions::event_exists;

#[derive(Debug, Default)]
pub struct ScreenshotPlugin {
    config: ScreenshotPluginConfig,
}

#[derive(Debug, Clone, Copy, Resource)]
pub struct ScreenshotPluginConfig {
    pub show_notification: bool,
    pub override_if_screenshot_exists: bool,
    pub with_egui_ui: bool,
    // OPTIONAL in wasm32
    // pub screenshot_save_location: ScreenShotSaveLocation,
}

impl Default for ScreenshotPluginConfig {
    fn default() -> Self {
        Self {
            show_notification: false,
            override_if_screenshot_exists: true,
            with_egui_ui: true,
            // screenshot_save_location: ScreenShotSaveLocation::default(),
        }
    }
}

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeScreenshot>()
            .insert_resource(self.config)
            .add_event::<TakeScreenshot>()
            .add_event::<TakeScreenshotFinished>()
            .add_systems(
                Update,
                (
                    toast_on_screenshot_finished_event.run_if(
                        event_exists::<ToastEvent>.and_then(on_event::<TakeScreenshotFinished>()),
                    ),
                    handle_screenshot_event.run_if(on_event::<TakeScreenshot>()),
                ),
            );
    }
}

#[derive(Debug, Event, Clone)]
pub struct TakeScreenshot {
    pub save_at_location: ScreenshotSaveLocation,
    pub postfix: ScreenshotSavePostfix,
    pub image_format: ImageFormat,
}

impl Default for TakeScreenshot {
    fn default() -> Self {
        Self {
            save_at_location: ScreenshotSaveLocation::default(),
            postfix: ScreenshotSavePostfix::default(),
            image_format: ImageFormat::Png,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum ScreenshotSaveLocation {
    At(std::path::PathBuf),
    #[default]
    Cwd,
    // Clipboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenshotSavePostfix {
    Number,
    UnixTimestamp,
}

impl Default for ScreenshotSavePostfix {
    fn default() -> Self {
        if cfg!(target_arch = "wasm32") {
            Self::UnixTimestamp
        } else {
            Self::Number
        }
    }
}

#[derive(Debug, Clone, Event)]
pub enum TakeScreenshotFinished {
    Success(String),
    Failure(String),
}

fn handle_screenshot_event(
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
    mut screen_shot_event: EventReader<TakeScreenshot>,
    mut screen_shot_finished_event: EventWriter<TakeScreenshotFinished>,
    // mut toast_event: EventWriter<ToastEvent>,
    config: Res<ScreenshotPluginConfig>,
) {
    // TODO: filter out ui panels
    for event in screen_shot_event.read() {
        info!("Read TakeScreenshot event. Taking screenshot...");
        let Ok(window) = primary_window.get_single() else {
            warn!("screenshot action was called without a main window!");
            return;
        };

        let basename_postfix = match event.postfix {
            ScreenshotSavePostfix::Number => {
                let existing_screenshots =
                    glob::glob("./screenshot_*.png").expect("valid glob pattern");
                let latest_screenshot_id = existing_screenshots
                    .filter_map(std::result::Result::ok)
                    .filter_map(|path| {
                        path.file_name().and_then(|file_name| {
                            file_name.to_str().map(std::string::ToString::to_string)
                        })
                    })
                    .filter_map(|basename| {
                        basename["screenshot_".len()..basename.len() - 4]
                            .parse::<usize>()
                            .ok()
                    })
                    .max();

                // TODO: handle wasm32 constraints

                let screenshot_id = latest_screenshot_id.map_or(0, |id| id + 1);
                screenshot_id.to_string()
            }
            ScreenshotSavePostfix::UnixTimestamp => chrono::Utc::now().timestamp().to_string(),
        };

        let extension = event
            .image_format
            .extensions_str()
            .first()
            .expect("every format has at least one extension");

        let dirname = match event.save_at_location {
            ScreenshotSaveLocation::Cwd if cfg!(not(target_arch = "wasm32")) => {
                std::env::current_dir().expect("current directory exists")
            }
            ScreenshotSaveLocation::Cwd => {
                panic!("cannot take screenshots when running in wasm32")
            }
            ScreenshotSaveLocation::At(ref path) => path.clone(),
        };

        let path = dirname
            .join(format!("screenshot_{}.{}", basename_postfix, extension))
            .to_string_lossy()
            .to_string();

        if let Err(err) = screenshot_manager.save_screenshot_to_disk(window, &path) {
            let error_msg = format!("failed to save screenshot to disk: {}", err);
            error!("failed to write screenshot to disk, error: {}", err);
            // toast_event.send(ToastEvent::error(error_msg));
            screen_shot_finished_event.send(TakeScreenshotFinished::Failure(error_msg));
            continue;
        };

        info!("saved screenshot to ./{}", path);

        if config.show_notification {
            // toast_event.send(ToastEvent::success(format!(
            //     "saved screenshot to ./{}",
            //     path
            // )));
            screen_shot_finished_event.send(TakeScreenshotFinished::Success(path));
        }
    }
}

fn toast_on_screenshot_finished_event(
    mut screen_shot_finished_event: EventReader<TakeScreenshotFinished>,
    mut toast_event: EventWriter<ToastEvent>,
) {
    for event in screen_shot_finished_event.read() {
        match event {
            TakeScreenshotFinished::Success(path) => {
                toast_event.send(ToastEvent::success(format!(
                    "saved screenshot to ./{}",
                    path
                )));
            }
            TakeScreenshotFinished::Failure(err) => {
                toast_event.send(ToastEvent::error(format!(
                    "failed to save screenshot: {}",
                    err
                )));
            }
        }
    }
}
