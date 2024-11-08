#![warn(clippy::all, rust_2018_idioms)]

mod app;

use std::{
    collections::HashMap,
    io::{BufReader, Read},
};

pub use app::Carelog;

use ehttp::Request;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};

use rusqlite::{params, Connection, Result};

const SHEET_URL: &str = "https://docs.google.com/spreadsheets/d/1vUyrTyEbVLQVf41y4KEpPJ_ukBDMUD4WrEB4nUQqSXI/gviz/tq?tqx=out:csv";

pub fn setup_file_logging() -> Result<(), Box<dyn std::error::Error>> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}\n")))
        .build("carelog.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    log::info!("Logging initialized to file");
    Ok(())
}

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
            patient_id integer references patient_info(id),
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

pub fn extract_number_from_brackets(input: &str) -> Option<u32> {
    input
        .split('(')
        .nth(1)
        .and_then(|s| s.split(')').next())
        .and_then(|s| s.parse().ok())
}

pub fn insert_patient_data(
    conn: &Connection,
    id: &u32,
    diagnosis_overview: &str,
    detailed_notes: &str,
    medical_tests: &str,
) -> Result<(), rusqlite::Error> {
    let visit_date = chrono::Local::now().date_naive().to_string();

    let mut stmt = conn.prepare_cached(
        "INSERT OR REPLACE INTO patient_data 
        (diagnosis_overview,detailed_notes,medical_test_info,patient_id,visit_date) 
        VALUES (?1, ?2, ?3,?4,?5)",
    )?;

    stmt.execute(params![
        diagnosis_overview,
        detailed_notes,
        medical_tests,
        id,
        visit_date
    ])?;

    Ok(())
}

#[derive(Debug)]
pub struct LicenseEntry {
    pub machine_uid: String,
    pub valid_till: String,
    pub licensee: String,
}

pub fn download_licenses_data() -> String {
    let req = Request::get(SHEET_URL);
    let res = ehttp::fetch_blocking(&req);

    let data: String = match res {
        Ok(res) => String::from_utf8_lossy(&res.bytes).into_owned(),
        Err(_) => {
            log::error!("Something went wrong while fetching the license data");
            String::from("")
        }
    };

    return data;
}

pub fn is_license_valid(machine_uid: &str, license_data: String) -> bool {
    // Parse the CSV data
    let mut reader = BufReader::new(license_data.as_bytes());
    let mut csv_data = String::new();
    reader.read_to_string(&mut csv_data).unwrap();

    // Store the expiration dates by machine UID
    let mut expiration_dates: HashMap<&str, chrono::NaiveDate> = HashMap::new();

    // Iterate through the CSV data, skipping the header row
    for line in csv_data.lines().skip(1) {
        let mut parts = line.split(",");

        // Extract the machine UID and expiration date
        let machine_uid_from_csv = parts.next().unwrap().trim_matches('"');
        let expiration_date = parts.next().unwrap().trim_matches('"');
        let _client = parts.next().unwrap().trim_matches('"');

        // Store the expiration date for the current machine UID
        let expiration_date =
            chrono::NaiveDate::parse_from_str(expiration_date, "%m/%d/%Y").unwrap();

        expiration_dates.insert(machine_uid_from_csv, expiration_date);
    }

    // Check if the given machine UID has a valid expiration date
    if let Some(expiration_date) = expiration_dates.get(machine_uid) {
        let now = chrono::Utc::now().date_naive();
        return *expiration_date >= now; // returns true if expiration_date > today
    }

    return false;
}
