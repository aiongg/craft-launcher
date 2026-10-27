//! The launcher's theme: its own choice or the Craft App Suite's, resolved and installed when it
//! changes. The suite's selection comes from a [`SuiteWatch`] the desktop shell injects; without
//! one (tests, web, portable installs) the suite lives in memory and "Follow suite" is not offered.

use craft_theme::{Selection, ThemeId};
use craft_theme_store::{Accepted, Op, SelectionChange, Snapshot, StoreError, SuiteChoice, SuiteWatch};

use crate::app::LauncherUi;
use crate::i18n::fmt;
use crate::theme;

/// The suite's selection as the launcher last saw it, and what it is waiting for.
pub(crate) struct Suite {
    pub(crate) watch: SuiteWatch,
    /// The suite folder is shared with the other Craft apps, so "Follow suite" is offered.
    pub(crate) shared: bool,
    /// The generation `accepted` was taken from.
    seen: Option<u64>,
    pub(crate) accepted: Accepted,
    /// A suite change sent to the store and the choice shown until its result arrives.
    pending: Option<(u64, SuiteChoice)>,
    /// The library theme last named with `Op::AppTheme`; `None` until one is sent.
    app_theme_sent: Option<Option<ThemeId>>,
    /// The suite generation and selection the installed styles were built for.
    applied: Option<(u64, Selection)>,
    /// The library was re-listed for the open Settings dialog.
    pub(crate) rescanned: bool,
}

impl Suite {
    pub(crate) fn new(watch: SuiteWatch, shared: bool) -> Self {
        Self { watch, shared, seen: None, accepted: Accepted::default(), pending: None, app_theme_sent: None, applied: None, rescanned: false }
    }

    /// The suite's choice as shown: a change still on its way, else the accepted one.
    pub(crate) fn choice(&self) -> &SuiteChoice {
        self.pending.as_ref().map_or(&self.accepted.suite, |(_, choice)| choice)
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
        self.settings.appearance().effective(self.suite.choice())
    }

    /// Installs the styles again when the suite or the launcher's choice changed (`force`: in any
    /// case, for a new context). One atomic load per frame otherwise.
    pub(crate) fn sync_appearance(&mut self, ctx: &egui::Context, force: bool) {
        if self.suite.seen != Some(self.suite.watch.generation()) {
            let snapshot = self.suite.watch.snapshot();
            self.take_snapshot(&snapshot);
        }
        self.send_app_theme();
        let sel = self.effective_appearance();
        let key = (self.suite.watch.generation(), sel);
        if !force && self.suite.applied.as_ref() == Some(&key) {
            return;
        }
        let snapshot = self.suite.watch.snapshot();
        theme::install_selection(ctx, &key.1, &|id| snapshot.theme(id));
        self.settings.theme = key.1.mode.into();
        self.suite.applied = Some(key);
    }

    fn take_snapshot(&mut self, snapshot: &Snapshot) {
        self.suite.seen = Some(snapshot.generation);
        self.suite.accepted = snapshot.accepted.clone();
        if let Some((id, _)) = &self.suite.pending
            && let Some(result) = snapshot.result(*id)
        {
            if let Err(e) = result {
                let text = match e {
                    StoreError::Conflict { .. } => crate::i18n::t("Changed in another Craft app; your choice wasn't applied").to_string(),
                    e => fmt(crate::i18n::t("Couldn't change the suite theme: {e}"), &[("e", &e.to_string())]),
                };
                self.toast(text, true);
            }
            self.suite.pending = None;
        }
        // Kept in the settings, so a damaged or newer suite file at the next start still shows
        // the last look.
        if self.suite.shared
            && let Some(appearance) = self.settings.appearance.as_mut()
            && appearance.last_suite != snapshot.accepted
        {
            appearance.last_suite = snapshot.accepted.clone();
        }
    }

    /// Names the library theme of the launcher's own choice, so the watcher keeps it parsed.
    fn send_app_theme(&mut self) {
        let appearance = self.settings.appearance();
        let wanted = appearance.own_choice().map(|own| own.theme.clone()).filter(|id| !id.is_builtin());
        if self.suite.app_theme_sent.as_ref() != Some(&wanted) {
            self.suite.watch.request(Op::AppTheme(wanted.clone()));
            self.suite.app_theme_sent = Some(wanted);
        }
    }

    /// Changes the suite's selection: shown at once, written by the watcher.
    pub(crate) fn change_suite(&mut self, change: SelectionChange) {
        let base = self.suite.accepted.clone();
        let shown = change.applied_to(self.suite.choice());
        let id = self.suite.watch.request(Op::Update { base, change });
        self.suite.pending = Some((id, shown));
    }

    /// Re-lists the library once each time the Settings dialog opens.
    pub(crate) fn rescan_suite_library(&mut self) {
        if !self.suite.rescanned {
            self.suite.watch.request(Op::Rescan);
            self.suite.rescanned = true;
        }
    }
}
