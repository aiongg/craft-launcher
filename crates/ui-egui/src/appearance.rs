//! The launcher's theme: its own choice or the Craft App Suite's, resolved and installed when it
//! changes. The suite's selection comes through a [`SuiteClient`] over the [`SuiteWatch`] the
//! desktop shell injects; without one (tests, web, portable installs) the suite lives in memory
//! and "Follow suite" is not offered.

use craft_theme::Selection;
use craft_theme_store::{Op, SelectionChange, SuiteClient, SuiteWatch};
use egui::{RichText, Ui};

use crate::app::LauncherUi;
use crate::i18n::{fmt, t};
use crate::theme::{self, Tokens};
use crate::widgets::{self, Kind};

/// The suite's selection as the launcher sees it.
pub(crate) struct Suite {
    pub(crate) client: SuiteClient,
    /// The suite folder is shared with the other Craft apps, so "Follow suite" is offered.
    pub(crate) shared: bool,
    /// The selection the installed styles were built for.
    applied: Option<Selection>,
    /// The library was re-listed for the open Settings dialog.
    pub(crate) rescanned: bool,
}

impl Suite {
    pub(crate) fn new(watch: SuiteWatch, shared: bool) -> Self {
        Self { client: SuiteClient::new(watch), shared, applied: None, rescanned: false }
    }
}

impl LauncherUi {
    /// Uses `watch` for the suite's selection and library. `shared`: it watches the suite folder
    /// the other Craft apps use (desktop, not portable), so "Follow suite" is offered.
    pub fn set_suite(&mut self, watch: SuiteWatch, shared: bool) {
        self.suite = Suite::new(watch, shared);
    }

    /// The theme and mode the launcher draws with now.
    pub(crate) fn effective_appearance(&self) -> Selection {
        self.settings.appearance().effective(&self.suite.client.shown())
    }

    /// Installs the styles again when the suite or the launcher's choice changed (`force`: in any
    /// case, for a new context). One atomic load per frame otherwise.
    pub(crate) fn sync_appearance(&mut self, ctx: &egui::Context, force: bool) {
        let changed = self.suite.client.sync();
        if changed && self.suite.shared {
            self.keep_last_suite();
        }
        let appearance = self.settings.appearance();
        self.suite.client.set_app_theme(appearance.own_choice().map(|own| own.theme.clone()));
        let mut sel = self.effective_appearance();
        // The launcher's own library theme is still loading: keep the current look.
        if self.suite.client.is_loading(&sel.theme)
            && let Some(applied) = &self.suite.applied
        {
            if !force {
                return;
            }
            sel = applied.clone();
        }
        if !force && !changed && self.suite.applied.as_ref() == Some(&sel) {
            return;
        }
        let client = &self.suite.client;
        theme::install_selection(ctx, &sel, &|id| client.theme(id));
        self.settings.theme = sel.mode.into();
        self.suite.applied = Some(sel);
    }

    /// Kept in the settings, so a damaged or newer suite file at the next start still shows the
    /// last look.
    fn keep_last_suite(&mut self) {
        let accepted = self.suite.client.accepted();
        if let Some(appearance) = self.settings.appearance.as_mut()
            && appearance.last_suite != *accepted
        {
            appearance.last_suite = accepted.clone();
        }
    }

    /// Changes the suite's selection: shown at once, written by the watcher.
    pub(crate) fn change_suite(&mut self, change: SelectionChange) {
        self.suite.client.change(change);
    }

    /// Re-lists the library once each time the Settings dialog opens.
    pub(crate) fn rescan_suite_library(&mut self) {
        if !self.suite.rescanned {
            self.suite.client.watch().request(Op::Rescan);
            self.suite.rescanned = true;
        }
    }

    /// Under the Appearance choices: a suite change another app's choice blocked ("Apply mine"),
    /// one that failed ("Retry"), and the suite files that can't be used.
    pub(crate) fn suite_notices(&mut self, ui: &mut Ui) {
        let tokens = Tokens::get(ui.ctx());
        let client = &self.suite.client;
        let notice = if client.conflict().is_some() {
            Some((t("Changed in another Craft app; your choice wasn't applied").to_string(), t("Apply mine")))
        } else {
            client.failure().map(|(_, e)| (fmt(t("Couldn't change the suite theme: {e}"), &[("e", &e.to_string())]), t("Retry")))
        };
        let problems: Vec<String> = client.problems().iter().map(ToString::to_string).collect();
        if let Some((text, action)) = notice {
            ui.add_space(6.0);
            ui.add(egui::Label::new(RichText::new(text).size(12.0).color(tokens.warn)).wrap());
            ui.horizontal(|ui| {
                if widgets::button(ui, action, Kind::Secondary, 0.0).clicked() {
                    // Only one of the two is set; each does nothing without its notice.
                    self.suite.client.apply_mine();
                    self.suite.client.retry();
                }
                if widgets::button(ui, t("Dismiss"), Kind::Secondary, 0.0).clicked() {
                    self.suite.client.dismiss();
                }
            });
        }
        if !problems.is_empty() {
            ui.add_space(6.0);
            let line = problems.join("; ");
            ui.add(egui::Label::new(RichText::new(&line).size(12.0).color(tokens.muted)).truncate()).on_hover_text(line);
        }
    }
}
