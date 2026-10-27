//! The suite-theme adapter: the same theme gives the launcher the same core colours as every other
//! Craft app, built-ins included, and a fresh profile shows ArtCraft.

use artcraft_launcher_ui_egui::theme::{self, Mode, Selection, ThemeId, Tokens, Variant};
use artcraft_launcher_ui_egui::{AppAppearance, Settings};
use craft_theme::{CoreRoles, Resolution, resolve};
use craft_theme_store::testing::{SAMPLE_THEME_ID, SAMPLE_THEME_JSON, sample_roles};
use craft_theme_store::{SelectionChange, Store, StoreOptions, SuiteChoice, SuiteDir};
use craft_ui::theme::color32;
use egui::Color32;

fn mode_of(variant: Variant) -> Mode {
    match variant {
        Variant::Light => Mode::Light,
        Variant::Dark => Mode::Dark,
    }
}

/// The launcher tokens that carry a core role unchanged: ink, bg, accent, good, warn, bad.
fn core_tokens(t: &Tokens) -> [Color32; 6] {
    [t.ink, t.bg, t.accent, t.good, t.warn, t.bad]
}

fn core_roles(r: &CoreRoles) -> [Color32; 6] {
    [r.text, r.background, r.accent, r.good, r.warn, r.bad].map(color32)
}

/// A folder of its own under the temp dir, removed when the test ends.
struct Scratch(std::path::PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_sample_theme_followed_from_a_suite_folder_gives_its_core_colours() {
    let dir = Scratch(std::env::temp_dir().join(format!("artcraft-launcher-suite-theme-{}", std::process::id())));
    let _ = std::fs::remove_dir_all(&dir.0);
    let mut store = Store::open(&SuiteDir::at(&dir.0), StoreOptions::new("artcraft-launcher test"));
    store.load_selection().unwrap();
    store.import(SAMPLE_THEME_JSON.as_bytes(), false).unwrap();
    let sample = ThemeId::parse(SAMPLE_THEME_ID).unwrap();
    let following = AppAppearance::follow_suite();
    for variant in [Variant::Light, Variant::Dark] {
        let base = store.accepted().clone();
        store.update(&base, SelectionChange::theme(sample.clone()).with_mode(mode_of(variant))).unwrap();
        let sel = following.effective(&store.accepted().suite);
        assert_eq!(sel, Selection { theme: sample.clone(), mode: mode_of(variant) });
        let theme = store.theme(&sample).expect("the imported theme");
        let (light, dark) = theme::slot_tokens(&sel, &|id| (*id == sample).then(|| theme.clone()));
        assert_eq!(light, dark, "a fixed mode puts the same variant in both slots");
        assert_eq!(core_tokens(&dark), core_roles(&sample_roles(variant)), "{variant:?}");
        assert_eq!(dark.dark, variant == Variant::Dark);
    }
}

#[test]
fn every_builtin_theme_gives_its_core_colours() {
    for builtin in craft_theme::builtins() {
        for variant in [Variant::Light, Variant::Dark] {
            let sel = Selection { theme: builtin.id.clone(), mode: mode_of(variant) };
            let Resolution::Theme(resolved) = resolve(&sel, None, theme::FALLBACK_VARIANT, &craft_theme::lookup_builtin) else {
                panic!("{} resolves to the app default", builtin.id.as_str());
            };
            let tokens = Tokens::from_resolved(&resolved);
            assert_eq!(core_tokens(&tokens), core_roles(&resolved.roles), "{} {variant:?}", builtin.id.as_str());
            let (_, dark_slot) = theme::slot_tokens(&sel, &craft_theme::lookup_builtin);
            assert_eq!(dark_slot, tokens);
        }
    }
}

#[test]
fn a_fresh_profile_shows_artcraft_dark_and_the_launcher_default_is_its_own_palette() {
    let fresh = Settings::default().appearance();
    assert!(fresh.follow_suite);
    let sel = fresh.effective(&SuiteChoice::default());
    assert_eq!(sel, Selection { theme: ThemeId::suite_default(), mode: Mode::Dark });
    assert_eq!(sel.theme.as_str(), "craft.artcraft");
    let (light, dark) = theme::slot_tokens(&Selection { theme: ThemeId::app_default(), mode: Mode::System }, &craft_theme::lookup_builtin);
    assert_eq!((light, dark), (Tokens::LIGHT, Tokens::DARK));
}
