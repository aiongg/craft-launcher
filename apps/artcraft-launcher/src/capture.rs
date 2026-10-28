//! One-shot window capture for the theme render matrix.
//!
//! With `ARTCRAFT_LAUNCHER_CAPTURE=<path.png>` set, the window is saved to that path once it has
//! settled (fonts loaded, theme applied), and the launcher then quits. The theme and mode come from
//! the usual settings and suite folder (`CRAFT_SUITE_DIR`, `selection.json`).

use std::path::PathBuf;
use std::time::{Duration, Instant};

use craft_ui::capture::CaptureSlot;

/// Name of the environment variable that turns the capture on.
pub const ENV: &str = "ARTCRAFT_LAUNCHER_CAPTURE";

const MAX_IMAGE_BYTES: usize = 128 * 1024 * 1024;
/// Passes and time to wait before capturing: fonts, theme and the first layout have settled.
const SETTLE_PASSES: u64 = 30;
const SETTLE_TIME: Duration = Duration::from_millis(1500);
/// Give up (and exit with the failure logged) when no image arrives.
const TIMEOUT: Duration = Duration::from_secs(15);

/// What a pass of [`Capture::step`] asks of the shell.
pub enum Step {
    Wait,
    /// The capture finished or failed; quit.
    Done,
}

pub struct Capture {
    path: PathBuf,
    started: Instant,
    first_pass: Option<u64>,
    requested: Option<(CaptureSlot, Instant)>,
}

impl Capture {
    pub fn from_env() -> Option<Self> {
        let path = std::env::var_os(ENV).filter(|v| !v.is_empty())?;
        Some(Self { path: PathBuf::from(path), started: Instant::now(), first_pass: None, requested: None })
    }

    pub fn step(&mut self, ctx: &egui::Context) -> Step {
        ctx.request_repaint_after(Duration::from_millis(50));
        let pass = ctx.cumulative_pass_nr();
        let first = *self.first_pass.get_or_insert(pass);
        if let Some((slot, asked)) = &self.requested {
            return match slot.take() {
                Some(Ok(image)) => {
                    if let Err(e) = save(&self.path, &image) {
                        log::error!("capture to {}: {e}", self.path.display());
                    }
                    Step::Done
                }
                Some(Err(e)) => {
                    log::error!("capture: {e}");
                    Step::Done
                }
                None if asked.elapsed() > TIMEOUT => {
                    log::error!("capture: no image after {}s", TIMEOUT.as_secs());
                    Step::Done
                }
                None => Step::Wait,
            };
        }
        if pass.saturating_sub(first) >= SETTLE_PASSES && self.started.elapsed() >= SETTLE_TIME {
            let (slot, callback) = CaptureSlot::new(ctx, egui::ViewportId::ROOT, MAX_IMAGE_BYTES);
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
            self.requested = Some((slot, Instant::now()));
        }
        Step::Wait
    }
}

fn save(path: &std::path::Path, image: &egui::ColorImage) -> Result<(), image::ImageError> {
    let [w, h] = image.size;
    let rgba: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    image::save_buffer(path, &rgba, w as u32, h as u32, image::ColorType::Rgba8)
}
