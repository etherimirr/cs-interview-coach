use chrono::{DateTime, Duration, Utc};

use crate::models::{FsrsState, Grade};

// M1: simplified FSRS-style scheduler (SM-2 lite with stability + difficulty).
// M2 TODO: replace with full FSRS-6 (or use fsrs-rs crate) — keep field names
// compatible so historical states upgrade cleanly.

const MIN_STABILITY: f32 = 0.5;
const MAX_DIFFICULTY: f32 = 10.0;
const MIN_DIFFICULTY: f32 = 1.0;

pub fn apply_grade(state: &mut FsrsState, grade: Grade, now: DateTime<Utc>) {
    state.reps += 1;
    state.last_review = now;

    let (stab_mul, diff_delta, is_lapse) = match grade {
        Grade::Again => (0.5_f32,  0.2, true),
        Grade::Hard  => (1.2_f32,  0.05, false),
        Grade::Good  => (2.5_f32,  0.0, false),
        Grade::Easy  => (3.5_f32, -0.1, false),
    };

    if is_lapse { state.lapses += 1; }

    state.stability = (state.stability * stab_mul).max(MIN_STABILITY);
    state.difficulty = (state.difficulty + diff_delta).clamp(MIN_DIFFICULTY, MAX_DIFFICULTY);

    let interval_days = state.stability.max(1.0).round() as i64;
    state.next_review = now + Duration::days(interval_days);
}
