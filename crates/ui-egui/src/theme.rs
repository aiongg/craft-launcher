//! Colours, type and shapes, matching getartcraft.com: hard edges, hairline rules, Archivo
//! display type, Inter body text and Geist Mono labels. Widgets read [`Tokens::get`]; nothing
//! hard-codes a colour.

// The generated table is written as `EmbeddedFace` literals, which craft-ui's `CraftFace` matches.
use craft_ui::fonts::{CraftFace as EmbeddedFace, Family, I18nScripts, Stack, UiFonts};
use egui::epaint::text::VariationCoords;
use egui::{Color32, CornerRadius, FontData, FontFamily, FontId, FontTweak, Stroke, TextStyle, Visuals};
use serde::{Deserialize, Serialize};

use crate::i18n::Lang;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeChoice {
    /// Follow the system's light or dark setting.
    #[default]
    System,
    Dark,
    Light,
}

impl ThemeChoice {
    pub const ALL: [Self; 3] = [Self::System, Self::Dark, Self::Light];

    /// The English key translated at draw time into the current UI language
    /// (the shell sets it per frame via `i18n::set_current`). Same-crate lookup,
    /// pure and wasm-safe; no new dependency. Call sites stay unchanged.
    pub fn label(self) -> &'static str {
        match self {
            Self::System => crate::i18n::t("Match system"),
            Self::Dark => crate::i18n::t("Dark"),
            Self::Light => crate::i18n::t("Light"),
        }
    }
}

/// Colour tokens for one appearance: the website's (`globals.css`), by the same names.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    pub dark: bool,
    /// Page and card surface.
    pub bg: Color32,
    /// Badges, wells and row hovers.
    pub sunken: Color32,
    /// Body text.
    pub ink: Color32,
    /// Headings.
    pub ink_strong: Color32,
    /// Secondary text.
    pub muted: Color32,
    /// Labels and tertiary text.
    pub faint: Color32,
    /// Hairlines.
    pub line: Color32,
    /// Outlined buttons and stronger rules.
    pub line_strong: Color32,
    /// The brand blue, for surfaces.
    pub accent: Color32,
    /// The brand blue, for text and icons.
    pub accent_ink: Color32,
    /// Primary buttons and hover inversion.
    pub invert_bg: Color32,
    pub invert_fg: Color32,
    pub good: Color32,
    pub warn: Color32,
    pub bad: Color32,
}

const fn ink_alpha(rgb: u32, alpha: u8) -> Color32 {
    let [_, r, g, b] = rgb.to_be_bytes();
    Color32::from_rgba_unmultiplied_const(r, g, b, alpha)
}

const fn hex(rgb: u32) -> Color32 {
    let [_, r, g, b] = rgb.to_be_bytes();
    Color32::from_rgb(r, g, b)
}

impl Tokens {
    pub const DARK: Self = Self {
        dark: true,
        bg: hex(0x121316),
        sunken: hex(0x060607),
        ink: hex(0xf2f1ee),
        ink_strong: hex(0xffffff),
        muted: ink_alpha(0xf2f1ee, 153),
        faint: ink_alpha(0xf2f1ee, 97),
        line: ink_alpha(0xf2f1ee, 38),
        line_strong: ink_alpha(0xf2f1ee, 102),
        accent: hex(0x2d81ff),
        accent_ink: hex(0x74aaff),
        invert_bg: hex(0xffffff),
        invert_fg: hex(0x0b0b0c),
        // PdfCraft's and LightCraft's inks, so status colours come from the site's palette.
        good: hex(0x3fd6b8),
        warn: hex(0xffc14d),
        bad: hex(0xff6259),
    };

    pub const LIGHT: Self = Self {
        dark: false,
        bg: hex(0xf2f1ee),
        sunken: hex(0xe9e8e4),
        ink: hex(0x101014),
        ink_strong: hex(0x000000),
        muted: ink_alpha(0x101014, 158),
        faint: ink_alpha(0x101014, 107),
        line: ink_alpha(0x101014, 41),
        line_strong: ink_alpha(0x101014, 102),
        accent: hex(0x2d81ff),
        accent_ink: hex(0x1659c4),
        invert_bg: hex(0x101014),
        invert_fg: hex(0xf2f1ee),
        good: hex(0x0a7563),
        warn: hex(0x8a5800),
        bad: hex(0xb3261e),
    };

    pub fn get(ctx: &egui::Context) -> Self {
        if ctx.theme() == egui::Theme::Dark { Self::DARK } else { Self::LIGHT }
    }

    pub fn hairline(&self) -> Stroke {
        Stroke::new(1.0, self.line)
    }

    /// An app's text colour (`--app-ink`): readable on this appearance's background.
    pub fn app_ink(&self, app: &artcraft_launcher_core::CraftApp) -> Color32 {
        rgb(if self.dark { app.ink_dark } else { app.ink_light })
    }
}

/// Mixes `c` into the background at `alpha` (0â€“1): tinted washes.
pub fn tint(c: Color32, alpha: f32) -> Color32 {
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

pub fn rgb(c: [u8; 3]) -> Color32 {
    Color32::from_rgb(c[0], c[1], c[2])
}

/// The shadow under menus, dialogs and toasts.
pub fn shadow(t: &Tokens) -> egui::epaint::Shadow {
    egui::epaint::Shadow { offset: [0, 8], blur: 24, spread: 0, color: Color32::from_black_alpha(if t.dark { 140 } else { 36 }) }
}

/// The dim over the page behind a drawer; `amount` runs from 0 (none) to 1.
pub fn scrim(t: &Tokens, amount: f32) -> Color32 {
    let alpha = if t.dark { 170.0 } else { 105.0 } * amount.clamp(0.0, 1.0);
    Color32::from_black_alpha(alpha.round() as u8)
}

/// Behind an enlarged picture: near-black in both appearances, as pictures are judged on black.
pub fn backdrop() -> Color32 {
    Color32::from_black_alpha(232)
}

/// A frame standing in for a picture, over [`backdrop`].
pub fn backdrop_panel() -> Color32 {
    Color32::from_white_alpha(14)
}

/// Ink on [`backdrop`], at `alpha` out of 255.
pub fn backdrop_ink(alpha: u8) -> Color32 {
    Color32::from_white_alpha(alpha)
}

fn visuals(t: &Tokens) -> Visuals {
    let mut v = if t.dark { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = t.bg;
    v.window_fill = t.bg;
    v.extreme_bg_color = t.sunken;
    v.text_edit_bg_color = Some(t.sunken);
    v.faint_bg_color = t.sunken;
    v.code_bg_color = t.sunken;
    v.window_stroke = Stroke::new(1.0, t.line_strong);
    v.window_corner_radius = CornerRadius::ZERO;
    v.menu_corner_radius = CornerRadius::ZERO;
    v.window_shadow = shadow(t);
    v.popup_shadow = shadow(t);
    v.override_text_color = Some(t.ink);
    v.hyperlink_color = t.accent_ink;
    v.selection.bg_fill = t.accent;
    v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    v.warn_fg_color = t.warn;
    v.error_fg_color = t.bad;
    for (w, fill, stroke, fg) in [
        (&mut v.widgets.noninteractive, t.bg, t.line, t.muted),
        (&mut v.widgets.inactive, t.bg, t.line, t.ink),
        (&mut v.widgets.hovered, t.sunken, t.line_strong, t.ink_strong),
        (&mut v.widgets.active, t.sunken, t.line_strong, t.ink_strong),
        (&mut v.widgets.open, t.sunken, t.line_strong, t.ink_strong),
    ] {
        w.bg_fill = fill;
        w.weak_bg_fill = fill;
        w.bg_stroke = Stroke::new(1.0, stroke);
        w.corner_radius = CornerRadius::ZERO;
        w.fg_stroke = Stroke::new(1.0, fg);
        w.expansion = 0.0;
    }
    v
}

/// Applies both appearances (egui switches between them with the theme preference).
pub fn apply(ctx: &egui::Context, choice: ThemeChoice) {
    ctx.set_visuals_of(egui::Theme::Dark, visuals(&Tokens::DARK));
    ctx.set_visuals_of(egui::Theme::Light, visuals(&Tokens::LIGHT));
    ctx.set_theme(match choice {
        ThemeChoice::System => egui::ThemePreference::System,
        ThemeChoice::Dark => egui::ThemePreference::Dark,
        ThemeChoice::Light => egui::ThemePreference::Light,
    });
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        // Between a checkbox (or radio button) and its label.
        style.spacing.icon_spacing = 10.0;
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.interact_size.y = CONTROL_H;
        style.spacing.menu_margin = egui::Margin::same(4);
        style.spacing.window_margin = egui::Margin::same(4);
        style.text_styles = [
            (TextStyle::Heading, FontId::new(24.0, display())),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(13.5, medium())),
            (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
        ]
        .into();
    });
}

/// Height of buttons, inputs and filter cells (the site's medium button).
pub const CONTROL_H: f32 = 34.0;

/// The site's `.hud-label`: 11 px Geist Mono 500, upper case, 0.15 em apart.
pub const HUD_SIZE: f32 = 11.0;
pub const HUD_SPACING: f32 = HUD_SIZE * 0.15;
/// Buttons: 11 px Geist Mono 700, upper case, 0.12 em apart.
pub const BUTTON_SPACING: f32 = HUD_SIZE * 0.12;

// ---- layout ------------------------------------------------------------------------------------
// The app page and the large dialogs (Settings, release notes) share these, so they read as one
// family; the toolbar and the footer use the same steps.

/// Padding inside the app page and the dialogs, round their headers, bodies and footers.
pub const PAD: f32 = 24.0;
/// The gap between the two columns of a page or dialog.
pub const COLUMN_GAP: f32 = 32.0;
/// Padding inside a hairline box: a group of settings, a feature, a release fact.
pub const BOX_MARGIN: egui::Margin = egui::Margin::symmetric(14, 12);
/// Below this width the toolbar moves the sort and "Update all" to a second row.
pub const TOOLBAR_COMPACT_BELOW: f32 = 640.0;
/// From this width the large dialogs read in two columns.
pub const DIALOG_COLUMNS_FROM: f32 = 760.0;
/// From this width the app page reads in two columns (its pictures need the room).
pub const PAGE_COLUMNS_FROM: f32 = 900.0;
/// Title sizes, largest first: an app page's name, a large dialog's title, a small prompt's title.
pub const PAGE_TITLE: f32 = 32.0;
pub const SHEET_TITLE: f32 = 26.0;
pub const DIALOG_TITLE: f32 = 22.0;
/// A row of icon links (the footer, Settings): the room round each logo, and the gap between their
/// targets; neighbours' logos sit `2 × ICON_LINK_PAD + ICON_LINK_GAP` apart.
pub const ICON_LINK_PAD: f32 = 6.0;
pub const ICON_LINK_GAP: f32 = 2.0;

/// Inter 500.
pub fn medium() -> FontFamily {
    FontFamily::Name("medium".into())
}

/// Inter 600.
pub fn semibold() -> FontFamily {
    FontFamily::Name("semibold".into())
}

/// Archivo 620 at 118 % width: the site's `.font-display` headings and app wordmarks.
pub fn display() -> FontFamily {
    FontFamily::Name("display".into())
}

/// Geist Mono 500: labels and badges.
pub fn mono() -> FontFamily {
    FontFamily::Name("mono".into())
}

/// Geist Mono 700: buttons and app number tabs.
pub fn mono_bold() -> FontFamily {
    FontFamily::Name("mono-bold".into())
}

/// craft-fonts' Chinese and Japanese interface faces, cut to the catalogs' characters (build.rs);
/// empty unless built with `CRAFT_FONTS_DIR`, and on the web.
static CRAFT_FONTS: &[EmbeddedFace] = include!(concat!(env!("OUT_DIR"), "/craft_fonts.rs"));

/// The bundled body font (Inter).
const INTER: &[u8] = include_bytes!("../../../assets/fonts/Inter-Variable.ttf");

/// A bundled face: the name egui knows it by, its file, its variation coordinates and its family.
type Face = (&'static str, &'static [u8], &'static [(&'static [u8; 4], f32)], FontFamily);

/// The site's three variable fonts, each at the weights (and width) it uses there.
fn bundled_faces() -> [Face; 7] {
    const ARCHIVO: &[u8] = include_bytes!("../../../assets/fonts/Archivo-Variable.ttf");
    const GEIST_MONO: &[u8] = include_bytes!("../../../assets/fonts/GeistMono-Variable.ttf");
    [
        ("Inter", INTER, &[(b"wght", 400.0), (b"opsz", 14.0)], FontFamily::Proportional),
        ("Inter-Medium", INTER, &[(b"wght", 500.0), (b"opsz", 14.0)], medium()),
        ("Inter-SemiBold", INTER, &[(b"wght", 600.0), (b"opsz", 14.0)], semibold()),
        ("Archivo-Display", ARCHIVO, &[(b"wght", 620.0), (b"wdth", 118.0)], display()),
        ("GeistMono", GEIST_MONO, &[(b"wght", 400.0)], FontFamily::Monospace),
        ("GeistMono-Medium", GEIST_MONO, &[(b"wght", 500.0)], mono()),
        ("GeistMono-Bold", GEIST_MONO, &[(b"wght", 700.0)], mono_bold()),
    ]
}

/// The font stack for drawing `cjk`, or for the languages the bundled faces cover with `None`:
/// the bundled faces, then (desktop only) the system's fonts for whatever they lack, then egui's.
pub fn font_stack(cjk: Option<Lang>) -> Stack {
    let languages: Vec<&str> = Lang::all().map(Lang::code).collect();
    let mut fonts = UiFonts::new(&I18nScripts).languages(&languages).ui_language(cjk.map_or("en", Lang::code));
    for (name, bytes, coords, family) in bundled_faces() {
        // No subpixel binning: these fonts carry no hinting instructions, so a glyph drawn at a
        // fractional x smears each stem across two pixels, leaving small text blurry at 100 %.
        let tweak = FontTweak {
            coords: VariationCoords::new(coords.iter().map(|(tag, value)| (**tag, *value))),
            subpixel_binning: Some(false),
            ..Default::default()
        };
        // The weight installed fonts draw at in this family.
        let weight = coords.iter().find(|(tag, _)| *tag == b"wght").map_or(400, |(_, w)| *w as u16);
        if let FontFamily::Name(family_name) = &family {
            fonts = fonts.family(Family::named(family_name).weight(weight));
        }
        fonts = fonts.app_face(name, FontData::from_static(bytes).tweak(tweak), &[family]);
    }
    fonts = fonts.craft_faces(CRAFT_FONTS);
    #[cfg(not(target_arch = "wasm32"))]
    {
        fonts = fonts.system_provider(system_fonts::provider(cjk));
    }
    fonts.stack()
}

/// Installs [`font_stack`] in `ctx`. `cjk`: the CJK language to draw, whose system font then
/// leads the search for characters the bundled faces lack, or `None`.
pub fn install_fonts(ctx: &egui::Context, cjk: Option<Lang>) {
    font_stack(cjk).install(ctx);
}

/// Does this build embed an interface face for the CJK language `lang`? Desktop builds made with
/// `CRAFT_FONTS_DIR` do; then the system's fonts are never searched for it.
pub fn cjk_font_embedded(lang: Lang) -> bool {
    lang.script().is_some_and(|script| CRAFT_FONTS.iter().any(|face| face.scripts.contains(&script.tag())))
}

/// Has the system font for `lang` already been looked for and found? Never looks.
pub fn cjk_font_loaded(lang: Lang) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        system_fonts::found(lang) == Some(true)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = lang;
        false
    }
}

/// Can the CJK language `lang` be drawn? With no embedded face for it, looks through the system's
/// fonts on the first call. The web build has neither, so CJK text can't be drawn there. A font
/// for another CJK language doesn't count: it leaves this one's own characters as tofu, so the
/// language stays in English.
pub fn cjk_font_available(lang: Lang) -> bool {
    if cjk_font_embedded(lang) {
        return true;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        system_fonts::available(lang)
    }
    #[cfg(target_arch = "wasm32")]
    {
        false
    }
}

/// The system's fonts, through egui's system font provider: one per UI language, as its locale
/// picks between the Chinese and Japanese forms of the same characters.
#[cfg(not(target_arch = "wasm32"))]
mod system_fonts {
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex, PoisonError};

    use egui::{FallbackRequest, FontFamily, FontProvider};
    use egui_system_font_provider::SystemFontProvider;

    use crate::i18n::Lang;

    /// The providers made so far, by UI language code ("" for the bundled faces' languages).
    static PROVIDERS: Mutex<BTreeMap<&'static str, Arc<SystemFontProvider>>> = Mutex::new(BTreeMap::new());
    /// Whether each CJK language's characters were found, by language code.
    static FOUND: Mutex<BTreeMap<&'static str, bool>> = Mutex::new(BTreeMap::new());

    pub(super) fn provider(cjk: Option<Lang>) -> Arc<dyn FontProvider> {
        let key = cjk.map_or("", Lang::code);
        let mut providers = PROVIDERS.lock().unwrap_or_else(PoisonError::into_inner);
        providers.entry(key).or_insert_with(|| Arc::new(SystemFontProvider::new().with_locale(cjk.map(Lang::code)))).clone()
    }

    pub(super) fn found(lang: Lang) -> Option<bool> {
        FOUND.lock().unwrap_or_else(PoisonError::into_inner).get(lang.code()).copied()
    }

    /// Does the system have a face for every character of the language's own name? That name
    /// uses characters only its own script has (简 is Simplified Chinese, 語 is Japanese).
    pub(super) fn available(lang: Lang) -> bool {
        if let Some(found) = found(lang) {
            return found;
        }
        let provider = provider(Some(lang));
        let family = FontFamily::Proportional;
        let mut buf = [0; 4];
        let found = lang.name().chars().all(|c| provider.font_for(&FallbackRequest { cluster: c.encode_utf8(&mut buf), family: &family }).is_some());
        if !found {
            log::warn!("no font for {} found on this system", lang.english_name());
        }
        FOUND.lock().unwrap_or_else(PoisonError::into_inner).insert(lang.code(), found);
        found
    }
}

#[cfg(test)]
mod tests {
    use craft_ui::fonts::{Sample, assert_covers};
    use egui::{Color32, FontFamily, FontId};

    use super::*;
    use crate::i18n::LANGUAGES;

    /// Catalog entries no code looks up: the view switch and the search field draw their icons
    /// themselves. egui's own faces had these symbols until 0.37.
    const UNUSED: [&str; 3] = ["⊞  Grid", "☰  Table", "🔍  Search apps"];

    /// The bundled and embedded faces draw every string of every catalog in a language they cover,
    /// and that language's own name in the picker, without any system font. CJK catalogs count when
    /// the build embeds their interface face (built with `CRAFT_FONTS_DIR`).
    #[test]
    fn bundled_faces_cover_every_shipped_string() {
        let embedded = |code| Lang::from_code(code).is_some_and(cjk_font_embedded);
        let samples = LANGUAGES.iter().filter(|info| info.script.is_none() || embedded(info.code)).flat_map(|info| {
            let strings = info.source.lines().filter(|line| !line.starts_with('#')).filter_map(|line| {
                let mut columns = line.split('\t');
                let source = columns.nth(1).filter(|source| !UNUSED.contains(source))?;
                Some([(source, source), (source, columns.next()?)])
            });
            let name = [(info.code, info.name)];
            strings.flatten().chain(name).map(|(key, text)| Sample { language: info.code, key, text })
        });
        assert_covers(&font_stack(None), samples);
    }

    /// Text no catalog has, typed into a field: a rare Han character, a Vietnamese letter and the
    /// Command key. Drawing it never panics, Inter draws the letter and the key, and every
    /// character a bundled, embedded or installed face has is drawn from that face, not as a box.
    #[test]
    fn a_field_draws_characters_outside_the_catalogs() {
        use craft_ui::fonts::missing_glyphs;
        use egui_kittest::Harness;
        use egui_kittest::kittest::Queryable;

        const TYPED: &str = "龘 ẫ ⌘";
        let stack = font_stack(None);
        assert_covers(&stack, [Sample { language: "und", key: "typed", text: "ẫ⌘" }]);

        let mut harness = Harness::builder().allow_missing_glyphs().build_ui_state(
            |ui, (installed, text): &mut (bool, String)| {
                if !*installed {
                    install_fonts(ui.ctx(), None);
                    *installed = true;
                }
                ui.add(egui::TextEdit::singleline(text).hint_text("Search apps"));
            },
            (false, String::new()),
        );
        harness.get_by_role(egui::accesskit::Role::TextInput).focus();
        harness.run();
        harness.get_by_role(egui::accesskit::Role::TextInput).type_text(TYPED);
        harness.run();
        assert_eq!(harness.state().1, TYPED);

        let font_id = FontId::proportional(14.0);
        for c in TYPED.chars().filter(|c| !c.is_whitespace()) {
            let text = c.to_string();
            let some_face_has_it =
                missing_glyphs(&stack, &FontFamily::Proportional, [Sample { language: "und", key: "typed", text: &text }], true).is_empty();
            if some_face_has_it {
                assert!(harness.ctx.fonts_mut(|f| f.has_glyph(&font_id, c)), "{c:?} has a face but is drawn as a box");
            }
        }
    }

    /// A CJK face, embedded or found on the system, sits on the Latin baseline. craft-ui fits the
    /// face into the Latin row with a vertical offset, so the glyph's position says nothing about
    /// where it is drawn: the check is on the ink. Without a CJK font (CI, the web) there is
    /// nothing to check.
    #[test]
    fn cjk_sits_on_the_latin_baseline() {
        for info in LANGUAGES.iter().filter(|l| l.script.is_some()) {
            let Some(lang) = Lang::from_code(info.code) else { continue };
            if !cjk_font_available(lang) {
                continue;
            }
            // A character the language's own catalog uses, so the sample fits its script: the
            // face must draw it, on the Latin baseline.
            let Some(sample) =
                info.source.lines().filter_map(|line| line.split('\t').nth(2)).flat_map(str::chars).find(|c| !c.is_ascii() && c.is_alphabetic())
            else {
                continue;
            };
            let ctx = egui::Context::default();
            install_fonts(&ctx, Some(lang));
            let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
            output.textures_delta.clear();
            let size = 20.0;
            let font_id = FontId::new(size, FontFamily::Proportional);
            assert!(ctx.fonts_mut(|f| f.has_glyphs(&font_id, &sample.to_string())), "{}: no glyph for {sample:?}", lang.code());
            let layout = |text: String| ctx.fonts_mut(|f| f.layout_no_wrap(text, font_id.clone(), Color32::WHITE));
            let latin = layout("A".to_owned());
            let baseline = latin.rows[0].row.glyphs[0].pos.y;
            let (latin, cjk) = (latin.mesh_bounds, layout(sample.to_string()).mesh_bounds);
            // CJK ink reaches a little below the baseline and about as high as a capital.
            let below = cjk.max.y - baseline;
            assert!((0.0..=0.2 * size).contains(&below), "{}: {sample:?} ends {below} below the baseline", lang.code());
            assert!((cjk.min.y - latin.min.y).abs() <= 0.1 * size, "{}: {sample:?} top {}, capital top {}", lang.code(), cjk.min.y, latin.min.y);
        }
    }
}
