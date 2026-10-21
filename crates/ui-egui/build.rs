//! Embeds craft-fonts' Chinese and Japanese interface faces, cut to the characters the catalogs
//! use, from the checkout `CRAFT_FONTS_DIR` names; an empty table when it is unset. The web build
//! embeds none: it has no CJK languages.

use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=src/i18n");
    let wasm = craft_fonts_build::target_is_wasm();
    let embedded =
        craft_fonts_build::Embed::new().for_ui().scripts(&["Hans", "Jpan"]).subset_to(catalogs()).filter(move |_| !wasm).write("craft_fonts.rs");
    if let Err(e) = embedded {
        println!("cargo::error={e}");
    }
}

/// Every translation catalog the crate embeds (`src/i18n/*.tsv`), and the language registry, which
/// holds each language's name in itself for the picker.
fn catalogs() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir("src/i18n")
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "tsv"))
        .collect();
    paths.sort();
    paths.push(PathBuf::from("src/i18n/lang.rs"));
    paths
}
