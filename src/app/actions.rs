use std::time::Duration;

#[derive(Debug)]
pub(crate) enum AppAction {
    RequestEncode(Option<Duration>),
    PersistSettings,
    Export,
    OpenFileDialog,
    Reset,
    RememberColor([u8; 4]),
    ToggleHistoryLock(usize),
    ClearHistory,
    CopyColor { label: String, text: String },
    RequestRepaint,
}
