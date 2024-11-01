#![warn(clippy::all, rust_2018_idioms)]

mod app;

use std::{
    fs::File,
    io::{self, BufReader, Write},
    path::Path,
};

pub use app::Carelog;

use csv::ReaderBuilder;
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

pub fn download_licenses(file_path: &'static str) {
    let req = Request::get(SHEET_URL);

    // Send the request with a callback
    ehttp::fetch(req, move |response| {
        match response {
            Ok(resp) if resp.ok => {
                // Convert response bytes to a string
                let data = String::from_utf8_lossy(&resp.bytes);

                let _save_op = save_to_file(file_path, &data);
                if _save_op.is_err() {
                    log::error!("Something went wrong during save_to_file within download_licenses")
                }
            }
            Ok(resp) => {
                log::error!("Error fetching data: {}", resp.status);
            }
            Err(err) => {
                log::error!("Request failed: {}", err);
            }
        }
    });
}

fn save_to_file(file_path: &str, data: &str) -> io::Result<()> {
    let path = Path::new(file_path);
    let mut file = File::create(&path)?; // Create or overwrite the file
    file.write_all(data.as_bytes())?; // Write the CSV data to the file
    Ok(())
}

pub fn is_license_valid(machine_uid: String) -> Result<bool, io::Error> {
    // Open the CSV file
    let file = File::open("licenses.csv")?;
    let reader = BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

    // Get the current date
    let current_date = chrono::Local::now().date_naive(); // Get current date in UTC

    // Iterate through the records in the CSV
    for result in csv_reader.records() {
        match result {
            Ok(record) => {
                if let Some(license_uid) = record.get(0) {
                    if license_uid == machine_uid {
                        // Check if the valid date is in the second column
                        if let Some(valid_date_str) = record.get(1) {
                            // Parse the valid date
                            if let Ok(valid_date) =
                                chrono::NaiveDate::parse_from_str(valid_date_str, "%m/%d/%Y")
                            {
                                // Check if the valid date is greater than or equal to the current date
                                if valid_date >= current_date {
                                    return Ok(true); // Machine UID is valid
                                } else {
                                    return Ok(false); // Machine UID found but expired
                                }
                            } else {
                                log::error!("Error parsing valid date: {}", valid_date_str);
                            }
                        }
                    }
                }
            }
            Err(err) => {
                log::error!("Error reading record: {}", err);
            }
        }
    }

    Ok(false) // Machine UID not found
}
