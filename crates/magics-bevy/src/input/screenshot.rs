use bevy::{prelude::*, render::view::screenshot::ScreenshotManager, window::PrimaryWindow};
use bevy_notify::ToastEvent;
use image::ImageFormat;

/// Plugin for taking and saving screenshots
#[derive(Debug, Default)]
pub struct ScreenshotPlugin {
    config: ScreenshotPluginConfig,
}

impl ScreenshotPlugin {
    /// Create a new ScreenshotPlugin with custom configuration
    pub fn new(config: ScreenshotPluginConfig) -> Self {
        Self { config }
    }
}

/// Configuration options for the ScreenshotPlugin
#[derive(Debug, Clone, Copy, Resource)]
pub struct ScreenshotPluginConfig {
    /// Whether to show a notification when a screenshot is taken
    pub show_notification: bool,
    /// Whether to override existing screenshots with the same name
    pub override_if_screenshot_exists: bool,
    /// Whether to include UI elements in the screenshot
    pub with_egui_ui: bool,
}

impl Default for ScreenshotPluginConfig {
    fn default() -> Self {
        Self {
            show_notification: true,
            override_if_screenshot_exists: true,
            with_egui_ui: true,
        }
    }
}

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TakeScreenshot>()
            .insert_resource(self.config)
            .add_event::<TakeScreenshotFinished>()
            .add_systems(
                Update,
                (
                    toast_on_screenshot_finished_event,
                    handle_screenshot_event,
                ),
            );
    }
}

/// Event to trigger taking a screenshot
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

/// Location where screenshots should be saved
#[derive(Debug, Clone, Default)]
pub enum ScreenshotSaveLocation {
    /// Save at a specific path
    At(std::path::PathBuf),
    /// Save in the current working directory
    #[default]
    Cwd,
}

/// Method to name screenshot files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenshotSavePostfix {
    /// Use a sequential number (screenshot_0.png, screenshot_1.png, etc.)
    Number,
    /// Use a Unix timestamp
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

/// Event fired when a screenshot operation is completed
#[derive(Debug, Clone, Event)]
pub enum TakeScreenshotFinished {
    /// Screenshot was taken successfully
    Success(String),
    /// Screenshot operation failed
    Failure(String),
}

/// System to handle screenshot events
fn handle_screenshot_event(
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
    mut screen_shot_event: EventReader<TakeScreenshot>,
    mut screen_shot_finished_event: EventWriter<TakeScreenshotFinished>,
    config: Res<ScreenshotPluginConfig>,
) {
    for event in screen_shot_event.read() {
        info!("Taking screenshot...");
        let Ok(window) = primary_window.get_single() else {
            warn!("Screenshot action was called without a main window!");
            screen_shot_finished_event.send(TakeScreenshotFinished::Failure(
                "No primary window found".to_string(),
            ));
            return;
        };

        let basename_postfix = match event.postfix {
            ScreenshotSavePostfix::Number => {
                if cfg!(target_arch = "wasm32") {
                    warn!("Number postfix not supported in WASM, using timestamp instead");
                    chrono::Utc::now().timestamp().to_string()
                } else {
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

                    let screenshot_id = latest_screenshot_id.map_or(0, |id| id + 1);
                    screenshot_id.to_string()
                }
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
                screen_shot_finished_event.send(TakeScreenshotFinished::Failure(
                    "Cannot take screenshots when running in WASM".to_string(),
                ));
                continue;
            }
            ScreenshotSaveLocation::At(ref path) => path.clone(),
        };

        let path = dirname
            .join(format!("screenshot_{}.{}", basename_postfix, extension))
            .to_string_lossy()
            .to_string();

        if let Err(err) = screenshot_manager.save_screenshot_to_disk(window, &path) {
            let error_msg = format!("Failed to save screenshot to disk: {err}");
            error!("{error_msg}");
            screen_shot_finished_event.send(TakeScreenshotFinished::Failure(error_msg));
            continue;
        };

        info!("Saved screenshot to ./{path}");

        if config.show_notification {
            screen_shot_finished_event.send(TakeScreenshotFinished::Success(path));
        }
    }
}

/// System to send toast notifications when screenshots are taken
fn toast_on_screenshot_finished_event(
    mut screen_shot_finished_event: EventReader<TakeScreenshotFinished>,
    mut toast_event: EventWriter<ToastEvent>,
) {
    for event in screen_shot_finished_event.read() {
        match event {
            TakeScreenshotFinished::Success(path) => {
                toast_event.send(ToastEvent::success(format!(
                    "Saved screenshot to ./{path}"
                )));
            }
            TakeScreenshotFinished::Failure(err) => {
                toast_event.send(ToastEvent::error(format!(
                    "Failed to save screenshot: {err}"
                )));
            }
        }
    }
}
