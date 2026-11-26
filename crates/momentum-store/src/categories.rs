use crate::{now_ms, Result, Store, StoreError};
use momentum_core::{is_hex_color, new_id, CategoryStatus};
use rusqlite::params;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Category {
    pub id: String,
    pub title: String,
    pub hex_color: String,
    pub status: CategoryStatus,
    pub position: i64,
}

impl Store {
    pub fn count_categories(&self) -> Result<i64> {
        Ok(self.conn().query_row(
            "SELECT COUNT(*) FROM category WHERE deleted_at IS NULL",
            [],
            |r| r.get(0),
        )?)
    }

    pub fn list_categories(&self) -> Result<Vec<Category>> {
        let mut stmt = self.conn().prepare(
            "SELECT id, title, hex_color, status, position FROM category
             WHERE deleted_at IS NULL ORDER BY position, created_at",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })?;
        rows.map(|row| {
            let (id, title, hex_color, status, position) = row?;
            let status = status
                .parse()
                .map_err(|e| StoreError::Invalid(format!("{e}")))?;
            Ok(Category {
                id,
                title,
                hex_color,
                status,
                position,
            })
        })
        .collect()
    }

    pub fn insert_category(&self, title: &str, hex_color: &str) -> Result<Category> {
        let title = title.trim();
        if title.is_empty() {
            return Err(StoreError::Invalid("category title is empty".into()));
        }
        if !is_hex_color(hex_color) {
            return Err(StoreError::Invalid(format!("invalid color {hex_color}")));
        }
        let id = new_id().to_string();
        let now = now_ms();
        let position: i64 = self.conn().query_row(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM category",
            [],
            |r| r.get(0),
        )?;
        self.conn().execute(
            "INSERT INTO category (id, title, hex_color, status, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?5)",
            params![id, title, hex_color, position, now],
        )?;
        Ok(Category {
            id,
            title: title.to_owned(),
            hex_color: hex_color.to_owned(),
            status: CategoryStatus::Active,
            position,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_validates_input_and_orders_by_position() {
        let store = Store::open_in_memory().unwrap();
        assert!(store.insert_category("  ", "#112233").is_err());
        assert!(store.insert_category("Code", "red").is_err());
        store.insert_category("Code", "#3B82F6").unwrap();
        let titles: Vec<_> = store
            .list_categories()
            .unwrap()
            .into_iter()
            .map(|c| c.title)
            .collect();
        assert_eq!(titles, ["Focus", "Code"]);
    }
}
