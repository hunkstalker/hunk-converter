pub const HISTORY_LIMIT: usize = 21;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    pub rgba: [u8; 4],
    pub locked: bool,
}

impl HistoryEntry {
    pub fn new(rgba: [u8; 4], locked: bool) -> Self {
        Self { rgba, locked }
    }
}

pub fn remember_color(history: &mut Vec<HistoryEntry>, rgba: [u8; 4]) -> bool {
    if let Some(index) = history.iter().position(|entry| entry.rgba == rgba) {
        let entry = history.remove(index);
        history.insert(0, entry);
        return true;
    }
    if history.len() >= HISTORY_LIMIT {
        let Some(index) = history.iter().rposition(|entry| !entry.locked) else {
            return false;
        };
        history.remove(index);
    }
    history.insert(0, HistoryEntry::new(rgba, false));
    true
}

pub fn toggle_lock(history: &mut [HistoryEntry], index: usize) -> Option<bool> {
    let entry = history.get_mut(index)?;
    entry.locked = !entry.locked;
    Some(entry.locked)
}

pub fn clear_unlocked(history: &mut Vec<HistoryEntry>) {
    history.retain(|entry| entry.locked);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(value: u8, locked: bool) -> HistoryEntry {
        HistoryEntry::new([value, 0, 0, 255], locked)
    }

    #[test]
    fn remembers_unique_colors_at_the_front() {
        let mut history = vec![entry(1, false), entry(2, false)];
        assert!(remember_color(&mut history, [3, 0, 0, 255]));
        assert_eq!(history[0].rgba[0], 3);
        assert!(remember_color(&mut history, [2, 0, 0, 255]));
        assert_eq!(history[0].rgba[0], 2);
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn replaces_the_oldest_unlocked_color() {
        let mut history = (0..HISTORY_LIMIT)
            .map(|index| entry(index as u8, false))
            .collect::<Vec<_>>();
        history[HISTORY_LIMIT - 1].locked = true;
        assert!(remember_color(&mut history, [99, 0, 0, 255]));
        assert_eq!(history[0].rgba[0], 99);
        assert_eq!(history.len(), HISTORY_LIMIT);
        assert_eq!(
            history[HISTORY_LIMIT - 1].rgba[0],
            (HISTORY_LIMIT - 1) as u8
        );
        assert!(history[HISTORY_LIMIT - 1].locked);
    }

    #[test]
    fn rejects_new_colors_when_everything_is_locked() {
        let mut history = (0..HISTORY_LIMIT)
            .map(|index| entry(index as u8, true))
            .collect::<Vec<_>>();
        assert!(!remember_color(&mut history, [99, 0, 0, 255]));
        assert_eq!(history.len(), HISTORY_LIMIT);
    }

    #[test]
    fn toggle_and_clear_keep_locked_colors() {
        let mut history = vec![entry(1, true), entry(2, false), entry(3, true)];
        assert_eq!(toggle_lock(&mut history, 1), Some(true));
        clear_unlocked(&mut history);
        assert_eq!(history.len(), 3);
        assert_eq!(toggle_lock(&mut history, 0), Some(false));
        clear_unlocked(&mut history);
        assert_eq!(history, vec![entry(2, true), entry(3, true)]);
    }
}
