//! Calcul des horaires d'une journée de tournage.
//!
//! Deux passes sur la liste ordonnée des éléments :
//! - *prévisionnel* : durées prévues uniquement ;
//! - *recalculé* : durées prévues + retards / avances saisis.
//!
//! Règles :
//! - un élément commence quand le précédent finit ;
//! - une pause à heure fixe ne se décale pas d'elle-même : si l'on arrive en avance,
//!   on attend l'heure fixe ; si un retard la rattrape, on **signale un conflit**
//!   (la pause est alors poussée dans le calcul pour garder une suite cohérente) et
//!   l'utilisateur décide (rendre la pause flexible, la déplacer, raccourcir une scène…).

use crate::planning::{DayItem, ItemKind};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotConflict {
    /// Un retard fait empiéter la suite sur une pause à heure fixe.
    FixedBreakPushed,
    /// Le planning prévisionnel lui-même déborde sur la pause à heure fixe.
    FixedBreakOverlap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Slot {
    pub item_id: Uuid,
    pub planned_start: u32,
    pub planned_end: u32,
    pub actual_start: u32,
    pub actual_end: u32,
    /// Décalage du début par rapport au prévisionnel (minutes, négatif = en avance).
    pub shift_min: i32,
    pub conflict: Option<SlotConflict>,
    /// Ampleur du conflit en minutes (0 s'il n'y en a pas).
    pub conflict_min: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub planned_end: u32,
    pub actual_end: u32,
    pub day_end: u32,
    /// Minutes de dépassement de l'heure de fin de journée (0 si tout rentre).
    pub overtime_min: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub slots: Vec<Slot>,
    pub summary: Summary,
}

pub fn compute(day_start: u32, day_end: u32, items: &[DayItem]) -> Schedule {
    let mut planned_cursor = day_start;
    let mut actual_cursor = day_start;
    let mut slots = Vec::with_capacity(items.len());

    for item in items {
        let fixed = match item.kind {
            ItemKind::Break => item.fixed_start_min.map(u32::from),
            ItemKind::Scene => None,
        };
        let actual_duration = (i64::from(item.duration_min) + i64::from(item.delay_min)).max(0) as u32;
        let mut conflict: Option<(SlotConflict, u32)> = None;

        let planned_start = match fixed {
            Some(f) if f >= planned_cursor => f,
            Some(f) => {
                conflict = Some((SlotConflict::FixedBreakOverlap, planned_cursor - f));
                planned_cursor
            }
            None => planned_cursor,
        };
        let actual_start = match fixed {
            Some(f) if f >= actual_cursor => f,
            Some(f) => {
                if conflict.is_none() {
                    conflict = Some((SlotConflict::FixedBreakPushed, actual_cursor - f));
                }
                actual_cursor
            }
            None => actual_cursor,
        };

        let planned_end = planned_start + item.duration_min;
        let actual_end = actual_start + actual_duration;
        planned_cursor = planned_end;
        actual_cursor = actual_end;

        slots.push(Slot {
            item_id: item.id,
            planned_start,
            planned_end,
            actual_start,
            actual_end,
            shift_min: actual_start as i32 - planned_start as i32,
            conflict: conflict.map(|c| c.0),
            conflict_min: conflict.map_or(0, |c| c.1),
        });
    }

    Schedule {
        slots,
        summary: Summary {
            planned_end: planned_cursor,
            actual_end: actual_cursor,
            day_end,
            overtime_min: actual_cursor.saturating_sub(day_end),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const H8: u32 = 8 * 60;

    fn scene(duration: u32, delay: i32) -> DayItem {
        DayItem {
            id: Uuid::new_v4(),
            day_id: Uuid::nil(),
            position: 0,
            kind: ItemKind::Scene,
            scene_id: Some(Uuid::new_v4()),
            label: String::new(),
            duration_min: duration,
            delay_min: delay,
            fixed_start_min: None,
        }
    }

    fn pause(duration: u32, fixed: Option<u16>) -> DayItem {
        DayItem {
            kind: ItemKind::Break,
            scene_id: None,
            label: "Pause".into(),
            fixed_start_min: fixed,
            ..scene(duration, 0)
        }
    }

    /// Exemple du cahier des charges : S1 1h, S2 1h30, pause 15 min, S3 1h15,
    /// déjeuner 1h, S4 1h30, en partant de 08h00.
    fn day(s2_delay: i32, lunch_fixed: Option<u16>) -> Vec<DayItem> {
        vec![
            scene(60, 0),
            scene(90, s2_delay),
            pause(15, None),
            scene(75, 0),
            pause(60, lunch_fixed),
            scene(90, 0),
        ]
    }

    #[test]
    fn without_delay_planned_and_actual_are_identical() {
        let s = compute(H8, 18 * 60, &day(0, Some(720)));
        assert_eq!(s.slots[0].planned_start, 480);
        assert_eq!(s.slots[4].planned_start, 720); // déjeuner 12h00
        assert_eq!(s.slots[5].planned_end, 870); // fin 14h30
        assert!(s.slots.iter().all(|x| x.shift_min == 0 && x.conflict.is_none()));
        assert_eq!(s.summary.overtime_min, 0);
    }

    #[test]
    fn delay_shifts_everything_after_it_when_lunch_is_flexible() {
        let s = compute(H8, 18 * 60, &day(30, None));
        assert_eq!(s.slots[0].shift_min, 0);
        assert_eq!(s.slots[1].shift_min, 0);
        assert_eq!(s.slots[1].actual_end, 660);
        for slot in &s.slots[2..] {
            assert_eq!(slot.shift_min, 30);
            assert_eq!(slot.conflict, None);
        }
        assert_eq!(s.summary.planned_end, 870);
        assert_eq!(s.summary.actual_end, 900);
    }

    #[test]
    fn delay_reaching_a_fixed_break_is_reported_as_a_conflict() {
        let s = compute(H8, 18 * 60, &day(30, Some(720)));
        let lunch = &s.slots[4];
        assert_eq!(lunch.conflict, Some(SlotConflict::FixedBreakPushed));
        assert_eq!(lunch.conflict_min, 30);
        assert_eq!(lunch.planned_start, 720);
        assert_eq!(lunch.actual_start, 750);
        assert_eq!(s.slots[5].actual_start, 810);
    }

    #[test]
    fn being_ahead_never_pulls_a_fixed_break_earlier() {
        let s = compute(H8, 18 * 60, &day(-15, Some(720)));
        assert_eq!(s.slots[1].actual_end, 615);
        assert_eq!(s.slots[3].shift_min, -15);
        let lunch = &s.slots[4];
        assert_eq!(lunch.actual_start, 720);
        assert_eq!(lunch.shift_min, 0);
        assert_eq!(lunch.conflict, None);
        assert_eq!(s.slots[5].shift_min, 0);
    }

    #[test]
    fn overtime_is_reported_against_the_day_end() {
        let s = compute(H8, 14 * 60, &day(30, Some(720)));
        assert_eq!(s.summary.actual_end, 900);
        assert_eq!(s.summary.overtime_min, 60);
    }

    #[test]
    fn planned_overlap_with_a_fixed_break_is_flagged() {
        let items = vec![scene(300, 0), pause(60, Some(720))]; // S1 08h00-13h00, déjeuner fixe 12h00
        let s = compute(H8, 18 * 60, &items);
        assert_eq!(s.slots[1].conflict, Some(SlotConflict::FixedBreakOverlap));
        assert_eq!(s.slots[1].conflict_min, 60);
    }

    #[test]
    fn empty_day_ends_at_start() {
        let s = compute(H8, 18 * 60, &[]);
        assert!(s.slots.is_empty());
        assert_eq!(s.summary.actual_end, H8);
        assert_eq!(s.summary.overtime_min, 0);
    }
}
