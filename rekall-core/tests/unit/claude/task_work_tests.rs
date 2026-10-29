use chrono::TimeDelta;

use super::*;

fn work(next_opens_session: bool, settled_ago_minutes: Option<i64>) -> TaskWork {
    TaskWork {
        remaining: Remaining::Open,
        settled_steps: 1,
        running_step_ids: Vec::new(),
        last_settled_at: settled_ago_minutes.map(|minutes| Instant::now().minus(TimeDelta::minutes(minutes))),
        next_opens_session,
    }
}

fn started_ago(minutes: i64) -> Instant {
    Instant::now().minus(TimeDelta::minutes(minutes))
}

#[test]
fn a_session_is_spent_once_it_claimed_and_the_next_step_opens_a_session() {
    assert!(work(true, Some(1)).session_is_spent(&started_ago(10)));
}

#[test]
fn a_claim_older_than_the_session_belongs_to_an_earlier_one() {
    assert!(!work(true, Some(30)).session_is_spent(&started_ago(10)));
    assert!(!work(true, None).session_is_spent(&started_ago(10)));
}

#[test]
fn a_session_that_the_next_step_joins_is_not_spent() {
    assert!(!work(false, Some(1)).session_is_spent(&started_ago(10)));
}

#[test]
fn a_step_still_running_or_no_work_left_keeps_the_session_going() {
    let running = TaskWork { running_step_ids: vec![Id::random()], ..work(true, Some(1)) };
    let finished = TaskWork { remaining: Remaining::Claimed, ..work(true, Some(1)) };

    assert!(!running.session_is_spent(&started_ago(10)));
    assert!(!finished.session_is_spent(&started_ago(10)));
}
