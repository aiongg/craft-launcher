//! MiSans (Xiaomi's free Simplified Chinese face) installed by hand, for a system whose own font
//! fallback offers no Chinese face (#378). Desktop only.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use egui::epaint::text::{FontInsert, InsertFontFamily};
use egui::{FallbackRequest, FontData, FontPriority, FontProvider};
use skrifa::{FontRef, MetadataProvider};

/// The MiSans files a user installs by hand, by weight preference.
const FILES: [&str; 3] = ["MiSans-Regular.ttf", "MiSans-Normal.ttf", "MiSans-Medium.ttf"];

/// Larger files aren't read: a MiSans weight is about 10 MB.
const MAX_FONT_BYTES: u64 = 100 * 1024 * 1024;

/// The system's fonts, then a hand-installed MiSans for the characters they lack.
pub(crate) struct WithMiSans {
    system: Arc<dyn FontProvider>,
    /// Read the first time the system has no face for a character.
    misans: OnceLock<Option<(FontData, Arc<Vec<u8>>)>>,
}

impl WithMiSans {
    pub(crate) fn new(system: Arc<dyn FontProvider>) -> Self {
        Self { system, misans: OnceLock::new() }
    }
}

impl FontProvider for WithMiSans {
    fn fonts_for_family(&self, family: &egui::FontFamily) -> Vec<FontInsert> {
        self.system.fonts_for_family(family)
    }

    fn font_for(&self, request: &FallbackRequest<'_>) -> Option<FontInsert> {
        self.system.font_for(request).or_else(|| {
            let c = request.cluster.chars().next()?;
            let (data, bytes) = self.misans.get_or_init(load).as_ref()?;
            let has_glyph = FontRef::new(bytes).is_ok_and(|font| font.charmap().map(c).is_some());
            has_glyph.then(|| FontInsert {
                name: "MiSans".to_owned(),
                data: data.clone(),
                families: vec![InsertFontFamily { family: request.family.clone(), priority: FontPriority::Lowest }],
            })
        })
    }
}

/// The first MiSans file found where hand-installed fonts go.
fn load() -> Option<(FontData, Arc<Vec<u8>>)> {
    let bytes = FILES.iter().flat_map(|file| font_dirs().into_iter().map(move |dir| dir.join(file))).find_map(|path| read(&path))?;
    log::info!("using a hand-installed MiSans for Chinese");
    let bytes = Arc::new(bytes);
    Some((FontData::from_blob(bytes.clone(), 0), bytes))
}

fn read(path: &Path) -> Option<Vec<u8>> {
    let size = std::fs::metadata(path).ok()?.len();
    if size > MAX_FONT_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    FontRef::new(&bytes).is_ok().then_some(bytes)
}

/// Where fonts a user installs by hand go on this system: the user's font folders, then the
/// system-wide ones, each as the font managers lay them out.
fn font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            dirs.push(PathBuf::from(home).join("Library/Fonts"));
        }
        dirs.push("/Library/Fonts".into());
    }
    #[cfg(target_os = "windows")]
    {
        // Fonts installed "for me" (the default since Windows 10 1809) go under the user's profile.
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            dirs.push(PathBuf::from(local).join(r"Microsoft\Windows\Fonts"));
        }
        let root = std::env::var_os("SystemRoot").map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from);
        dirs.push(root.join("Fonts"));
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            for sub in [".local/share/fonts", ".local/share/fonts/MiSans", ".local/share/fonts/misans", ".fonts"] {
                dirs.push(home.join(sub));
            }
        }
        for dir in [
            "/usr/share/fonts/MiSans",
            "/usr/share/fonts/misans",
            "/usr/share/fonts/truetype/misans",
            "/usr/share/fonts/TTF",
            "/usr/local/share/fonts/MiSans",
        ] {
            dirs.push(dir.into());
        }
    }
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hand-installed MiSans is looked for in absolute font folders on every system.
    #[test]
    fn misans_is_looked_for_where_users_install_fonts() {
        let dirs = font_dirs();
        assert!(!dirs.is_empty());
        assert!(dirs.iter().all(|dir| dir.is_absolute()), "{dirs:?}");
    }

    /// The system's answer wins; MiSans only answers for characters it has.
    #[test]
    fn misans_answers_after_the_system_and_only_for_its_characters() {
        const INTER: &[u8] = include_bytes!("../../../assets/fonts/Inter-Variable.ttf");
        let bytes = Arc::new(INTER.to_vec());
        let none = |_: &FallbackRequest<'_>| None;
        let provider = WithMiSans { system: Arc::new(none), misans: OnceLock::from(Some((FontData::from_blob(bytes.clone(), 0), bytes))) };
        let family = egui::FontFamily::Proportional;
        let found = provider.font_for(&FallbackRequest { cluster: "A", family: &family });
        assert_eq!(found.map(|insert| insert.name).as_deref(), Some("MiSans"));
        assert!(provider.font_for(&FallbackRequest { cluster: "简", family: &family }).is_none());

        let system =
            |_: &FallbackRequest<'_>| Some(FontInsert { name: "system".to_owned(), data: FontData::from_static(INTER), families: Vec::new() });
        let provider = WithMiSans::new(Arc::new(system));
        assert_eq!(provider.font_for(&FallbackRequest { cluster: "简", family: &family }).map(|insert| insert.name).as_deref(), Some("system"));
    }
}
