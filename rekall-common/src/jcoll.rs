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
mod tests {
    use super::*;

    #[test]
    fn the_hash_is_the_one_java_computes() {
        // UUID.fromString("00000000-0000-0001-0000-000000000002").hashCode() == 3
        assert_eq!(uuid_hash(Uuid::from_u64_pair(1, 2)), 3);
        // new UUID(0x1234_5678_0000_0000L, 0).hashCode() == 0x12345678
        assert_eq!(uuid_hash(Uuid::from_u64_pair(0x1234_5678_0000_0000, 0)), 0x1234_5678);
        // new UUID(-1L, 0).hashCode() == 0 (the two halves cancel)
        assert_eq!(uuid_hash(Uuid::from_u64_pair(u64::MAX, 0)), 0);
    }

    #[test]
    fn items_come_out_by_bucket_then_by_insertion() {
        let ids = [Uuid::from_u64_pair(0, 5), Uuid::from_u64_pair(0, 2), Uuid::from_u64_pair(0, 21), Uuid::from_u64_pair(0, 9)];
        // Buckets (hash & 15): 5, 2, 5 (21 = 16 + 5, after the one with 5), 9.
        let ordered = hash_set_order(ids.to_vec(), |id| *id);
        assert_eq!(ordered, vec![ids[1], ids[0], ids[2], ids[3]]);
    }

    #[test]
    fn the_table_doubles_past_three_quarters() {
        assert_eq!(capacity_for(12), 16);
        assert_eq!(capacity_for(13), 32);
        assert_eq!(capacity_for(25), 64);
    }

    /// Orders printed by a JVM (`new HashSet<>()` then `add` in order) for random ids.
    #[test]
    fn the_order_matches_what_a_jvm_printed() {
        let cases: [(&str, &str); 6] = [
            ("149faf38-a704-45e7-9da2-8966114460ee,d78e3600-9684-466f-ba07-c74cf0b6ae3d", "1 0"),
            ("2b07aa58-2cad-4d37-aded-480fc56c5f06,243e48f5-4d00-408d-80e8-5ed356926e43,c1d0ec34-a7ce-4cb0-a929-599c0f0e11a4", "2 1 0"),
            ("4b6d520e-7e1a-4ff8-9322-5426aaf8409e,ed51d18a-1545-492d-9902-631cb24b8c32,4699d2ec-214f-4b38-8d9c-85027dd5259b,be78ccc3-89dd-44ba-9096-ccc6358a826d,d31363b6-c506-4da5-8451-767683e6d4fd", "2 0 1 4 3"),
            ("cbe9ca68-465a-4681-98cb-2565d3b53d6c,1db727db-e56d-40a4-bfe7-69e613f0d11b,0f6413c1-cdfc-414a-8907-4dbac8802fd2,cc0d8108-1f68-4fb0-aece-cee22e544bc3,9ad9a001-8edd-4183-bdbc-b711d46bce8f,d14a2175-835b-4475-ba18-092bb08be962,03c196c2-d914-4b83-9def-04ed5d055443,9e5db977-a4a5-4871-95af-a35bf36ab797", "6 3 7 5 2 0 1 4"),
            ("1a852dd6-6b4f-46ae-8396-2accc98bd51e,34f2f9aa-61f8-4e2c-bdc5-979498395b23,4576c309-996a-4ce0-aec4-8bf0985a61c0,89d11f92-805b-4068-aa62-eff3c4cd67d0,d50d5deb-9c24-4f65-b382-27715ece1518,1c16fc29-328f-4825-bcf3-a92083c8b0c8,0c0a3c63-49b6-4ad2-8000-09f1e3e245ac,8cad1a08-23d1-47d3-85d0-a54bf154e6e6,e5ec287c-0d28-40fd-ac59-b9d7e43f9015,5a40779c-6c9a-4684-8baa-42d2b8a777e6,faa83168-dc56-4514-b6e8-87d6d8e11386,a5ee862b-63e2-4ede-80a5-f3c71da71dd6,82735594-27e4-40d6-8254-8a782c13cebf", "8 4 5 1 11 7 6 12 2 9 10 3 0"),
            ("27bbc984-924a-40c8-9c76-6f2a84d5c73d,ac78f931-197e-481b-b60d-5a753bb5c51e,6b591ccf-366f-461d-b85e-2539805088ac,5566e369-f67d-4cf1-9757-80298cd703c2,6dd1a81b-d6d7-4adb-92a3-1081b2967a03,22e788cf-77f0-474b-a4d5-952c9ae0db5b,cb06a7e2-c152-4ea5-9126-34914f3d6502,aeb6bdfc-5304-46c6-83e8-3c3667a0b943,5032b623-1c17-4600-b62f-4a999ac11325,f47b459e-9540-4ae1-b6a4-db38951955f1,16cf9bca-64f0-4c40-b12a-c26e5914beff,98c4d00b-548a-4f85-8ca6-7d3bb56f2d0d,d295fa3b-6421-4dc7-8a31-d4e91ef0c6e5,5df466ef-5eed-48bd-8834-67ba75abe915,bd285d72-eb42-43a8-9d87-1a26cd2f89b6,b3048216-d21a-4e26-9f82-7030be7f2a5c,5223f2cb-b03c-4a9a-b639-ae8652bf9c3f,ef87e57c-efbc-4813-8c5e-8800f42d42fa,dc8322b7-5cf3-4617-bbe4-65f40c3c3f99,cf4397a4-751d-4c43-abca-252e83ab0626", "12 18 3 14 0 9 19 4 5 16 8 7 10 6 13 17 1 2 11 15"),
        ];
        for (ids, expected) in cases {
            let ids: Vec<Uuid> = ids.split(',').map(|s| s.parse().unwrap()).collect();
            let order: Vec<usize> = hash_set_order((0..ids.len()).collect(), |i| ids[*i]);
            let printed: Vec<usize> = expected.split(' ').map(|s| s.parse().unwrap()).collect();
            assert_eq!(order, printed);
        }
    }
}
