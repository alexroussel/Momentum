use crate::{now_ms, Result, Store};
use rusqlite::{params, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};

impl Store {
    /// Reads a JSON setting; `None` when unset.
    pub fn get_setting<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let raw: Option<String> = self
            .conn()
            .query_row(
                "SELECT value_json FROM setting WHERE key = ?1",
                [key],
                |r| r.get(0),
            )
            .optional()?;
        raw.map(|s| serde_json::from_str(&s).map_err(Into::into))
            .transpose()
    }

    /// Writes (upserts) a JSON setting.
    pub fn set_setting<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let json = serde_json::to_string(value)?;
        self.conn().execute(
            "INSERT INTO setting (key, value_json, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json,
                                            updated_at = excluded.updated_at",
            params![key, json, now_ms()],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Store;

    #[test]
    fn settings_round_trip() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.get_setting::<String>("theme").unwrap(), None);
        store.set_setting("theme", &"light").unwrap();
        store.set_setting("theme", &"dark").unwrap();
        assert_eq!(
            store.get_setting::<String>("theme").unwrap().as_deref(),
            Some("dark")
        );
    }
}
