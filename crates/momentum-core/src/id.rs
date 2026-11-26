use uuid::Uuid;

/// Returns a new time-ordered UUID v7, used for every primary key (sync-ready).
pub fn new_id() -> Uuid {
    Uuid::now_v7()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_v7_and_monotonic() {
        let a = new_id();
        let b = new_id();
        assert_eq!(a.get_version_num(), 7);
        assert!(a < b, "UUID v7 must sort by creation time");
    }
}
