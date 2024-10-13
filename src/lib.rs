#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::Carelog;

use rusqlite::{params, Connection, Result};

pub const DB_URL: &str = "./carelog.db";

pub fn create_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "create table if not exists patient_info (
            id integer primary key,
            first_name text not null,
            last_name text,
            mobile_number text
        )",
        params![],
    )?;

    conn.execute(
        "create table if not exists patient_relation (
            relation_id integer primary key,
            patient_one integer not null references patient_info(id),
            patient_two integer not null references patient_info(id)
        )",
        params![],
    )?;

    Ok(())
}

pub fn insert_new_patient(
    conn: &Connection,
    first_name: &str,
    last_name: &str,
    mobile_number: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO patient_info (first_name,last_name,mobile_number) VALUES (?1, ?2, ?3)",
    )?;
    stmt.execute([first_name, last_name, mobile_number])?;
    Ok(())
}
