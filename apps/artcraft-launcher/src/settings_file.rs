//! The settings in eframe's own file, read and written while there is no window.
//!
//! eframe keeps its state (window size, egui memory and the launcher's settings) in `app.ron` in
//! its storage folder, but only while a window is open. Where the window closes to the tray
//! (Wayland), the launcher carries on without one, so it saves its settings to the same file
//! itself: same folder, same format, other keys left as they are.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Where the settings file is. In portable mode it is inside `ArtCraftLauncherData`, so a suite
/// carried on a stick keeps its settings when it is plugged into another computer. Without
/// `portable.txt` it is eframe's own folder, which is in the user's profile and cannot travel.
pub fn path(app_id: &str) -> Option<PathBuf> {
    artcraft_launcher_native::paths::portable_root()
        .map(|root| portable_settings(&root))
        .or_else(|| eframe::storage_dir(app_id).map(|dir| dir.join("app.ron")))
}

/// The window-restore store's file: `window.json` beside the settings file, so a portable suite
/// carries its window placement with it.
pub fn window_store_path(settings: &Path) -> PathBuf {
    settings.with_file_name(craft_window::restore::FILE_NAME)
}

/// The window-restore store for the settings file `settings` (an in-memory one without a path).
/// A store with no placement imports eframe's `"window"` entry from `app.ron`, which older builds
/// saved there; that entry stays as it is for them.
pub fn open_window_store(settings: Option<&Path>) -> craft_window::restore::Store {
    use craft_window::restore::{MAIN, Store, legacy};
    let Some(settings) = settings else { return Store::in_memory() };
    let mut store = Store::at(&window_store_path(settings));
    if store.get(MAIN).is_none()
        && let Some(placement) = legacy::eframe_app_ron_file(settings)
    {
        store.set(MAIN, &placement);
    }
    store
}

/// The settings file inside a portable suite: the same `data` folder `Paths` already uses.
fn portable_settings(root: &Path) -> PathBuf {
    root.join("data").join("app.ron")
}

/// The value stored under `key`, if the file has one that still reads.
pub fn load<T: serde::de::DeserializeOwned>(path: &Path, key: &str) -> Option<T> {
    eframe::get_value(&RonFile::read(path), key)
}

/// Stores `value` under `key`, keeping what else the file holds.
pub fn save<T: serde::Serialize>(path: &Path, key: &str, value: &T) -> Result<(), String> {
    let mut file = RonFile::read(path);
    eframe::set_value(&mut file, key, value);
    file.write(path)
}

/// eframe's file: a RON map of keys to RON strings.
#[derive(Default)]
struct RonFile(HashMap<String, String>);

impl RonFile {
    /// What the file holds; empty when it is missing or doesn't read, as eframe takes it.
    fn read(path: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else { return Self::default() };
        match ron::from_str(&text) {
            Ok(kv) => Self(kv),
            Err(e) => {
                log::warn!("{}: {e}", path.display());
                Self::default()
            }
        }
    }

    /// Writes a new file and moves it into place, so a failed write leaves the old one whole.
    fn write(&self, path: &Path) -> Result<(), String> {
        let text = ron::ser::to_string_pretty(&self.0, ron::ser::PrettyConfig::default()).map_err(|e| e.to_string())?;
        artcraft_launcher_native::paths::write_atomic(path, text.as_bytes())
    }
}

impl eframe::Storage for RonFile {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.0.insert(key.to_string(), value);
    }

    fn remove_string(&mut self, key: &str) {
        self.0.remove(key);
    }

    fn flush(&mut self) {}
}

#[cfg(test)]
mod tests {
    use artcraft_launcher_ui_egui::Settings;

    use super::*;

    /// A file in a folder of its own, removed when the test ends.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("artcraft-launcher-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            Self(dir)
        }
        fn file(&self) -> PathBuf {
            self.0.join("app.ron")
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// eframe 0.37's `"window"` entry as it writes it to `app.ron`.
    const EFRAME_WINDOW: &str =
        "(inner_position_pixels:Some((x:210.0,y:130.0)),inner_size_points:Some((x:900.0,y:640.0)),maximized:false,fullscreen:false)";

    fn write_window(dir: &Scratch, window: &str) {
        assert!(std::fs::create_dir_all(&dir.0).is_ok());
        assert!(std::fs::write(dir.file(), format!("{{\n    \"window\": {window:?},\n}}")).is_ok());
    }

    #[test]
    fn the_window_store_sits_beside_the_settings() {
        let settings = PathBuf::from("data").join("app.ron");
        assert_eq!(window_store_path(&settings), PathBuf::from("data").join("window.json"));
    }

    #[test]
    fn the_window_comes_from_eframes_app_ron_once() {
        use craft_window::restore::MAIN;
        let dir = Scratch::new("window-import");
        write_window(&dir, EFRAME_WINDOW);
        let store = open_window_store(Some(&dir.file()));
        let placement = store.get(MAIN).expect("imported");
        assert_eq!(placement.normal.size, [900.0, 640.0]);
        assert_eq!(placement.position_px, Some([210, 130]));
        assert!(!placement.maximized);
        // The old entry stays for older builds.
        assert_eq!(RonFile::read(&dir.file()).0.get("window").map(String::as_str), Some(EFRAME_WINDOW));
    }

    #[test]
    fn a_saved_placement_is_not_overwritten_by_the_import() {
        use craft_window::restore::{MAIN, Placement};
        let dir = Scratch::new("window-no-import");
        write_window(&dir, EFRAME_WINDOW);
        let mut store = craft_window::restore::Store::at(&window_store_path(&dir.file()));
        let mine = Placement::new([1300.0, 800.0], 1.0);
        store.set(MAIN, &mine);
        assert!(matches!(store.save(), Ok(true)));
        assert_eq!(open_window_store(Some(&dir.file())).get(MAIN), Some(mine));
    }

    #[test]
    fn a_saved_window_reads_back_after_a_restart() {
        use craft_window::restore::{MAIN, Placement};
        let dir = Scratch::new("window-round-trip");
        let mut first = open_window_store(Some(&dir.file()));
        assert_eq!(first.get(MAIN), None);
        let mut placement = Placement::new([1000.0, 700.0], 2.0);
        placement.maximized = true;
        first.set(MAIN, &placement);
        assert!(matches!(first.save(), Ok(true)));
        assert_eq!(open_window_store(Some(&dir.file())).get(MAIN), Some(placement));
    }

    #[test]
    fn a_portable_suite_keeps_its_settings_inside_itself() {
        // Built with `join` so the test reads the same on Windows and on Linux CI.
        let root = PathBuf::from("Suite").join("ArtCraftLauncherData");
        assert_eq!(portable_settings(&root), root.join("data").join("app.ron"));
    }

    #[test]
    fn settings_saved_with_no_file_read_back() {
        let dir = Scratch::new("settings-round-trip");
        let settings = Settings { last_check: Some(1_791_500_000), notify: false, ..Settings::default() };
        assert_eq!(save(&dir.file(), "settings", &settings), Ok(()));
        assert_eq!(load::<Settings>(&dir.file(), "settings"), Some(settings));
    }

    #[test]
    fn saving_keeps_what_eframe_stored() {
        let dir = Scratch::new("settings-keeps-window");
        assert!(std::fs::create_dir_all(&dir.0).is_ok());
        // As eframe writes it: a map of keys to RON strings.
        let window = r#"(inner_position_pixels:Some((x:10.0,y:20.0)),fullscreen:false)"#;
        let eframe_file = format!("{{\n    \"window\": {window:?},\n    \"settings\": \"(notify:true)\",\n}}");
        assert!(std::fs::write(dir.file(), eframe_file).is_ok());
        assert_eq!(load::<Settings>(&dir.file(), "settings").map(|s| s.notify), Some(true));

        let settings = Settings { notify: false, ..Settings::default() };
        assert_eq!(save(&dir.file(), "settings", &settings), Ok(()));
        assert_eq!(load::<Settings>(&dir.file(), "settings"), Some(settings));
        assert_eq!(RonFile::read(&dir.file()).0.get("window").map(String::as_str), Some(window));
    }

    #[test]
    fn saving_settings_leaves_an_unrelated_temporary_file_alone() {
        let dir = Scratch::new("settings-temp-collision");
        std::fs::create_dir_all(&dir.0).unwrap();
        let tmp = dir.file().with_extension("ron.tmp");
        std::fs::write(&tmp, b"another writer's data").unwrap();
        let settings = Settings { notify: false, ..Settings::default() };
        save(&dir.file(), "settings", &settings).unwrap();
        assert_eq!(load::<Settings>(&dir.file(), "settings"), Some(settings));
        assert_eq!(std::fs::read(&tmp).unwrap(), b"another writer's data");
    }

    #[test]
    fn a_missing_or_broken_file_has_no_settings() {
        let dir = Scratch::new("settings-broken");
        assert_eq!(load::<Settings>(&dir.file(), "settings"), None);
        assert!(std::fs::create_dir_all(&dir.0).is_ok());
        assert!(std::fs::write(dir.file(), "not ron {").is_ok());
        assert_eq!(load::<Settings>(&dir.file(), "settings"), None);
    }

    /// Writes eframe's file with `settings` as the saved RON under the settings key.
    fn write_settings(dir: &Scratch, settings: &str) {
        assert!(std::fs::create_dir_all(&dir.0).is_ok());
        assert!(std::fs::write(dir.file(), format!("{{\n    \"settings\": {settings:?},\n}}")).is_ok());
    }

    /// What the shell shows after loading: saved settings migrated, or the defaults when there
    /// are none (as `Launcher::start` does).
    fn restored(path: &Path) -> Settings {
        let mut settings = load::<Settings>(path, "settings").unwrap_or_default();
        settings.migrate();
        settings
    }

    /// Fresh installs follow the suite; settings saved before suite themes keep the launcher's own
    /// palette in their saved mode; settings with an appearance keep it.
    #[test]
    fn the_appearance_migrates_from_fresh_legacy_and_new_settings() {
        use artcraft_launcher_ui_egui::AppAppearance;
        use artcraft_launcher_ui_egui::theme::{Mode, Selection, ThemeChoice, ThemeId};

        let fresh = Scratch::new("appearance-fresh");
        assert_eq!(restored(&fresh.file()).appearance, Some(AppAppearance::follow_suite()));

        let legacy = Scratch::new("appearance-legacy");
        write_settings(&legacy, "(theme:Light,notify:false)");
        let settings = restored(&legacy.file());
        let own = Selection { theme: ThemeId::app_default(), mode: Mode::Light };
        assert_eq!(settings.appearance, Some(AppAppearance::own(own)));
        assert!(!settings.notify);
        write_settings(&legacy, "(notify:false)");
        assert_eq!(restored(&legacy.file()).appearance, Some(AppAppearance::launcher_default(ThemeChoice::System)));

        let new = Scratch::new("appearance-new");
        let mut appearance = AppAppearance::follow_suite();
        appearance.own_pick = Some(Selection { theme: ThemeId::parse("craft.graphite").unwrap(), mode: Mode::Light });
        appearance.last_suite =
            craft_theme_store::Accepted::from_selection(Selection { theme: ThemeId::parse("craft.high-contrast").unwrap(), mode: Mode::System });
        appearance.last_suite.revision = 7;
        let settings = Settings { appearance: Some(appearance), theme: ThemeChoice::Dark, ..Settings::default() };
        assert_eq!(save(&new.file(), "settings", &settings), Ok(()));
        assert_eq!(restored(&new.file()), settings, "a restart keeps the appearance as saved");
    }

    /// Settings from an earlier pilot build keep the own choice under `own`, which this build
    /// doesn't read: everything else still loads, and the launcher follows the suite.
    #[test]
    fn an_earlier_pilot_appearance_keeps_the_other_settings() {
        use artcraft_launcher_ui_egui::settings::SortKey;
        let dir = Scratch::new("appearance-pilot");
        write_settings(
            &dir,
            r#"(language:"de",sort:Name,hidden:["photocraft"],appearance:Some((follow_suite:false,own:(theme:"craft.graphite",mode:light),last_suite:(suite:(theme:None,mode:None),revision:0,library_revision:0))))"#,
        );
        let settings = restored(&dir.file());
        assert_eq!(settings.language, "de");
        assert_eq!(settings.sort, SortKey::Name);
        assert_eq!(settings.hidden.iter().map(String::as_str).collect::<Vec<_>>(), ["photocraft"]);
        let appearance = settings.appearance();
        assert_eq!((appearance.follow_suite, appearance.own_pick), (true, None));
    }
}
