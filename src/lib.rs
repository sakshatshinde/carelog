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
            mobile_number text,
            date_of_birth text,
            address text
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

    conn.execute(
        "create table if not exists patient_data (
            patient_id integer primary key references patient_info(id),
            visit_date text,
            diagnosis_overview text,
            detailed_notes text,
            medical_test_info text
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
    date_of_birth: &chrono::NaiveDate,
    address: &str,
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO patient_info (first_name,last_name,mobile_number,date_of_birth,address) VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    stmt.execute([
        first_name,
        last_name,
        mobile_number,
        &date_of_birth.to_string(),
        address,
    ])?;
    Ok(())
}

pub fn helper_avaliable_patients_in_db(conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare_cached("SELECT first_name, last_name, id FROM patient_info")?;

    // Execute the query and map each row to a formatted String, collecting them into a Vec.
    let result = stmt
        .query_map([], |row| {
            let first_name: String = row.get(0)?;
            let last_name: String = row.get(1)?;
            let id: i32 = row.get(2)?;

            // Format as "FirstName LastName (ID)"
            Ok(format!("{} {} ({})", first_name, last_name, id))
        })?
        .collect::<Result<Vec<String>, _>>()?;

    Ok(result)
}
