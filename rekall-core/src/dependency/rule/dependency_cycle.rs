use std::collections::{HashMap, HashSet};

use rekall_kernel::Id;

/// Whether adding "`task_id` depends on `depends_on`" to the `existing` pairs (task, what it
/// depends on) would make a task wait, through any number of others, for itself.
pub fn closes_a_cycle(existing: &[(Id, Id)], task_id: Id, depends_on: Id) -> bool {
    if task_id == depends_on {
        return true;
    }
    let mut waits_for: HashMap<Id, Vec<Id>> = HashMap::new();
    for (task, dependency) in existing {
        waits_for.entry(*task).or_default().push(*dependency);
    }
    let mut seen = HashSet::new();
    let mut pending = vec![depends_on];
    while let Some(current) = pending.pop() {
        if current == task_id {
            return true;
        }
        if seen.insert(current) {
            pending.extend(waits_for.get(&current).into_iter().flatten().copied());
        }
    }
    false
}

#[cfg(test)]
#[path = "../../../tests/unit/dependency/rule/dependency_cycle_tests.rs"]
mod tests;
