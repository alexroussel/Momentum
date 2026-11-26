use rusqlite_migration::{Migrations, M};

/// Ordered list of schema migrations. Never edit a released migration: append a new one.
pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(include_str!("../migrations/0001_initial.sql"))])
}
