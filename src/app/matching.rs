use std::cmp::Reverse;

use crate::domain::model::{LiveWindow, WindowHandle, WindowIdentity, WindowLayout};
use crate::domain::ports::{MatchOutcome, WindowMatch, WindowMatcher};

/// Pairs windows that belong to the same executable, preferring those whose
/// title and class also match. Assignment is greedy over all candidate
/// pairs ordered by score, so an exact match is never stolen by a weaker one.
#[derive(Debug, Default)]
pub struct ExecutableMatcher;

impl ExecutableMatcher {
    fn similarity(a: &WindowIdentity, b: &WindowIdentity) -> u8 {
        let title = if a.title == b.title { 2 } else { 0 };
        let class = if a.class_name == b.class_name { 1 } else { 0 };
        title + class
    }
}

impl WindowMatcher for ExecutableMatcher {
    fn pair<'a>(&self, targets: &'a [WindowLayout], live: &[LiveWindow]) -> MatchOutcome<'a> {
        let mut candidates: Vec<(u8, usize, usize)> = targets
            .iter()
            .enumerate()
            .flat_map(|(t, target)| {
                live.iter()
                    .enumerate()
                    .filter(move |(_, window)| window.identity.app == target.identity.app)
                    .map(move |(l, window)| (Self::similarity(&target.identity, &window.identity), t, l))
            })
            .collect();
        candidates.sort_by_key(|&(score, t, l)| (Reverse(score), t, l));

        let mut assigned: Vec<Option<WindowHandle>> = vec![None; targets.len()];
        let mut claimed = vec![false; live.len()];
        for (_, t, l) in candidates {
            if assigned[t].is_none() && !claimed[l] {
                assigned[t] = Some(live[l].handle);
                claimed[l] = true;
            }
        }

        let mut outcome = MatchOutcome::default();
        for (target, window) in targets.iter().zip(assigned) {
            match window {
                Some(window) => outcome.matched.push(WindowMatch { target, window }),
                None => outcome.unmatched.push(target),
            }
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::fakes::*;

    #[test]
    fn matches_by_executable_case_insensitively() {
        let targets = [layout("C:\\Apps\\Editor.exe", "a")];
        let live = [live(7, "c:\\apps\\editor.EXE", "b")];

        let outcome = ExecutableMatcher.pair(&targets, &live);

        assert_eq!(outcome.matched.len(), 1);
        assert_eq!(outcome.matched[0].window, WindowHandle::from_raw(7));
    }

    #[test]
    fn exact_title_wins_even_when_target_comes_later() {
        let targets = [layout("editor.exe", "first"), layout("editor.exe", "second")];
        let live = [live(1, "editor.exe", "second"), live(2, "editor.exe", "other")];

        let outcome = ExecutableMatcher.pair(&targets, &live);

        let second = outcome.matched.iter().find(|m| m.target.identity.title == "second").unwrap();
        assert_eq!(second.window, WindowHandle::from_raw(1));
        assert_eq!(outcome.matched.len(), 2);
    }

    #[test]
    fn surplus_targets_are_reported_unmatched() {
        let targets = [layout("editor.exe", "a"), layout("editor.exe", "b"), layout("other.exe", "c")];
        let live = [live(1, "editor.exe", "a")];

        let outcome = ExecutableMatcher.pair(&targets, &live);

        assert_eq!(outcome.matched.len(), 1);
        assert_eq!(outcome.unmatched.len(), 2);
    }
}
