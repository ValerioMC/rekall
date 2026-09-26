//! The iteration order of the one `java.util.HashSet` whose order reached the wire: a note's
//! tasks (`Document.tasks`, the inverse side of a many-to-many, which Hibernate loads into a
//! plain `HashSet`). That order is not random: it is the order of the hash table's buckets, and
//! the bucket of a `Task` comes from `UUID.hashCode()` of its id, so the same ids always come
//! out in the same order. Ties within a bucket keep the order the elements went in.

use uuid::Uuid;

const DEFAULT_CAPACITY: usize = 16;

/// `UUID.hashCode()`: the two halves folded together.
pub fn uuid_hash(id: Uuid) -> i32 {
    let (most, least) = id.as_u64_pair();
    let hilo = (most ^ least) as i64;
    ((hilo >> 32) as i32) ^ (hilo as i32)
}

/// `HashMap.hash`: the high bits spread into the low ones.
fn spread(hash: i32) -> u32 {
    let h = hash as u32;
    h ^ (h >> 16)
}

/// The table a `new HashSet<>()` has grown to after `size` insertions: sixteen buckets,
/// doubled whenever the size passes three quarters of them.
fn capacity_for(size: usize) -> usize {
    let mut capacity = DEFAULT_CAPACITY;
    while size > capacity * 3 / 4 {
        capacity *= 2;
    }
    capacity
}

/// `items` in the order a `HashSet` holding them, added in the order given, iterates them.
pub fn hash_set_order<T>(items: Vec<T>, id_of: impl Fn(&T) -> Uuid) -> Vec<T> {
    let mask = capacity_for(items.len()) as u32 - 1;
    let mut keyed: Vec<(u32, usize, T)> =
        items.into_iter().enumerate().map(|(i, item)| (spread(uuid_hash(id_of(&item))) & mask, i, item)).collect();
    keyed.sort_by_key(|(bucket, position, _)| (*bucket, *position));
    keyed.into_iter().map(|(_, _, item)| item).collect()
}

#[cfg(test)]
#[path = "../tests/unit/jcoll_tests.rs"]
mod tests;
