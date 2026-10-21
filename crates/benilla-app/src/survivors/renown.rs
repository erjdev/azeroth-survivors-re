//! Renown: the class's progression across runs. A level is spent when the run ends. Marks
//! are not. They are earned from the run and spent at the menu on three ranks that are already
//! in place when the next run starts.

use std::path::PathBuf;

use bevy::prelude::*;

use super::data;
use super::Run;

pub(super) const MAX_RANK: u8 = 5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Track {
    Might,
    Fortitude,
    Celerity,
}

impl Track {
    pub fn label(self) -> &'static str {
        match self {
            Track::Might => "Might",
            Track::Fortitude => "Fortitude",
            Track::Celerity => "Quickness",
        }
    }

    fn all() -> [Track; 3] {
        [Track::Might, Track::Fortitude, Track::Celerity]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClassRenown {
    unspent: u32,
    might: u8,
    fortitude: u8,
    celerity: u8,
}

impl ClassRenown {
    fn rank(self, track: Track) -> u8 {
        match track {
            Track::Might => self.might,
            Track::Fortitude => self.fortitude,
            Track::Celerity => self.celerity,
        }
    }

    fn set_rank(&mut self, track: Track, rank: u8) {
        match track {
            Track::Might => self.might = rank,
            Track::Fortitude => self.fortitude = rank,
            Track::Celerity => self.celerity = rank,
        }
    }
}

impl Default for ClassRenown {
    fn default() -> Self {
        Self {
            unspent: 0,
            might: 0,
            fortitude: 0,
            celerity: 0,
        }
    }
}

/// Saved per class in `benilla-config/survivors-renown.txt`. `last_earned` is this session's
/// results line only; it is not written to the file.
#[derive(Resource)]
pub(super) struct Renown {
    classes: Vec<ClassRenown>,
    pub last_earned: u32,
}

impl Default for Renown {
    fn default() -> Self {
        Self::load()
    }
}

impl Renown {
    fn load() -> Self {
        let classes = path()
            .and_then(|p| crate::local_state::read_to_string(&p).ok())
            .map(|text| parse(&text))
            .unwrap_or_else(|| vec![ClassRenown::default(); data::CLASSES.len()]);
        Self {
            classes,
            last_earned: 0,
        }
    }

    fn save(&self) {
        let Some(path) = path() else {
            return;
        };
        let mut body = String::from("# unspent might fortitude quickness, one class per line\n");
        for c in &self.classes {
            body.push_str(&format!(
                "{} {} {} {}\n",
                c.unspent, c.might, c.fortitude, c.celerity
            ));
        }
        if let Err(e) = crate::local_state::write_atomic(&path, &body) {
            warn!("survivors: renown save failed: {e}");
        }
    }

    pub fn ranks(&self, class: usize) -> [u8; 3] {
        self.classes.get(class).map_or([0; 3], |c| {
            [c.might, c.fortitude, c.celerity]
        })
    }

    pub fn unspent(&self, class: usize) -> u32 {
        self.classes.get(class).map_or(0, |c| c.unspent)
    }

    pub fn rank(&self, class: usize, track: Track) -> u8 {
        self.classes.get(class).map_or(0, |c| c.rank(track))
    }

    pub fn can_spend(&self, class: usize, track: Track) -> bool {
        self.classes.get(class).is_some_and(|c| {
            c.rank(track) < MAX_RANK && c.unspent >= rank_cost(c.rank(track))
        })
    }

    /// Spend the next rank's price on `track`. A full rank or a short purse does nothing.
    /// The first rank costs 1 mark and each rank after costs one more, so a track is 15.
    pub fn spend(&mut self, class: usize, track: Track) -> bool {
        let Some(c) = self.classes.get_mut(class) else {
            return false;
        };
        let cost = rank_cost(c.rank(track));
        if c.rank(track) >= MAX_RANK || c.unspent < cost {
            return false;
        }
        c.unspent -= cost;
        c.set_rank(track, c.rank(track) + 1);
        self.save();
        true
    }

    pub fn status_line(&self, class: Option<usize>) -> String {
        let Some(i) = class else {
            return "Renown — pick a class".into();
        };
        let c = self.classes.get(i).copied().unwrap_or_default();
        format!(
            "Renown marks {}    Might {}    Fortitude {}    Quickness {}",
            c.unspent,
            priced(c.might),
            priced(c.fortitude),
            priced(c.celerity),
        )
    }
}

/// Marks to buy the rank after `rank`. Shown in parentheses on the menu.
pub(super) fn rank_cost(rank: u8) -> u32 {
    u32::from(rank) + 1
}

fn priced(rank: u8) -> String {
    if rank >= MAX_RANK {
        format!("{rank}/{MAX_RANK}")
    } else {
        format!("{rank}/{MAX_RANK} ({})", rank_cost(rank))
    }
}

fn path() -> Option<PathBuf> {
    crate::local_state::home().map(|h| h.join("survivors-renown.txt"))
}

/// One data line per class, in [`data::CLASSES`] order. A leading comment does not shift class 0.
fn parse(text: &str) -> Vec<ClassRenown> {
    let mut classes = vec![ClassRenown::default(); data::CLASSES.len()];
    let mut slot = 0;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(c) = classes.get_mut(slot) else {
            break;
        };
        slot += 1;
        let mut n = line.split_whitespace().filter_map(|w| w.parse::<u32>().ok());
        *c = ClassRenown {
            unspent: n.next().unwrap_or(0),
            might: n.next().unwrap_or(0).min(u32::from(MAX_RANK)) as u8,
            fortitude: n.next().unwrap_or(0).min(u32::from(MAX_RANK)) as u8,
            celerity: n.next().unwrap_or(0).min(u32::from(MAX_RANK)) as u8,
        };
    }
    classes
}

pub(super) fn class_index(class: &data::ClassDef) -> usize {
    data::CLASSES
        .iter()
        .position(|c| c.id == class.id)
        .unwrap_or(0)
}

/// Marks a finished run is worth. A short death still leaves one. A full clear caps at 8, and
/// later ranks cost more than one mark, so one run cannot finish a track.
pub(super) fn marks_for(run: &Run) -> u32 {
    let time = (run.elapsed / 120.0) as u32;
    let kills = run.kills / 120;
    let bosses = run.bosses;
    let win = u32::from(run.won) * 2;
    (time + kills + bosses + win).clamp(1, 8)
}

/// Once, when the run ends in death or victory. Abandoning from the pause menu pays nothing.
pub(super) fn grant(mut renown: ResMut<Renown>, mut run: ResMut<Run>) {
    if run.renown_paid {
        return;
    }
    run.renown_paid = true;
    let n = marks_for(&run);
    let i = class_index(run.class);
    if let Some(c) = renown.classes.get_mut(i) {
        c.unspent = c.unspent.saturating_add(n);
    }
    renown.last_earned = n;
    renown.save();
}

pub(super) fn tracks() -> [Track; 3] {
    Track::all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finished_run_always_pays_and_never_more_than_eight() {
        let mut run = Run::default();
        assert_eq!(marks_for(&run), 1);
        run.elapsed = 120.0 * 30.0;
        run.kills = 120 * 30;
        run.bosses = 20;
        run.won = true;
        assert_eq!(marks_for(&run), 8);
    }

    #[test]
    fn later_ranks_cost_more_and_five_is_the_last() {
        let mut renown = Renown {
            classes: vec![ClassRenown {
                unspent: 15,
                might: 0,
                fortitude: 0,
                celerity: 0,
            }],
            last_earned: 0,
        };
        for want in 1..=5 {
            assert!(renown.spend(0, Track::Might));
            assert_eq!(renown.rank(0, Track::Might), want);
        }
        assert_eq!(renown.unspent(0), 0);
        assert!(!renown.spend(0, Track::Might));
        assert!(!renown.can_spend(0, Track::Fortitude));
    }

    #[test]
    fn a_comment_header_does_not_shift_the_first_class() {
        let classes = parse("# unspent might fortitude quickness\n3 2 1 0\n");
        assert_eq!(classes[0].unspent, 3);
        assert_eq!(classes[0].might, 2);
        assert_eq!(classes[0].fortitude, 1);
        assert_eq!(classes[0].celerity, 0);
        assert_eq!(classes[1].unspent, 0);
    }
}
