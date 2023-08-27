use crate::helper::{generate_flexdata_from_str};
use crate::{DataPoint, FlexIndex, FlexDataPoint, FlexData, FlexDataVector, FlexDataType, FlexSeries, FlexTable};
use chrono::prelude::*;
use std::convert::TryFrom;
use std::str::FromStr;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::collections::{HashMap, HashSet, BTreeMap};
use std::process;
use itertools::{iproduct};



extern crate csv;

pub fn get_tables(path: &String, seasons: Vec<&str>, divisions: Vec<&str>, tables: bool) -> (BTreeMap::<[String; 2], FlexTable>,
                                                                                            BTreeMap::<String, HashSet<FlexDataType>>) {
    let mut all_headers = BTreeMap::<String, HashSet<FlexDataType>>::new();
    let mut all_tables = BTreeMap::<[String; 2], FlexTable>::new();
    for (season, division) in iproduct!(seasons, divisions) {
        let mut file_path = path.to_owned();
        file_path.push_str(season);
        file_path.push_str(division);
        file_path.push_str(&".csv");

        if !Path::new(&file_path).exists() { continue }

        println!("{:?}", file_path);
        match hdr_types(file_path.to_string()) {
            Err(err) => {
                println!("{}", err);
                process::exit(1);
            },
            Ok((hdr, tbl)) => {
                if tables {
                    all_tables.insert([season.to_string(), division.to_string()], tbl);
                } else {
                    for head in hdr {
                        all_headers.entry(head.0).or_insert(HashSet::new()).insert(head.1);
                    }
                }
            },
        }
    }
    (all_tables, all_headers)
}


fn hdr_types(file_path: String) -> Result<(HashMap<String, FlexDataType>, FlexTable), csv::Error> {

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .escape(Some(b'\\'))
        .from_path(file_path)?;

    let mut headers = Vec::<(String, Vec<String>)>::new();
    for head in rdr.headers()? {
        headers.push((head.to_string(),Vec::<String>::new()));
    }

    let mut idx = 0;

    let mut record = csv::ByteRecord::new();
    while rdr.read_byte_record(&mut record)? {
        let mut temp_head = headers.clone();
        headers = temp_head.into_iter().zip(&record)
                .map(|(field, entry)| (field.0, {let mut newfield = field.1.clone();
                                                let mut dp = match String::from_utf8(entry.to_vec()) {
                                                    Ok(val) => val,
                                                    Err(_) => String::from_utf8(entry.iter().map(|&x| {if x > 31 && x < 128 {x} else {32}}).collect::<Vec<u8>>()).unwrap()
                                                };
                                                if dp == "NA".to_string() {dp = "".to_string()};
                                                newfield.push(dp.trim().to_string());
                                                newfield}))
                                                .filter(|(k, v)| k != "")
                                                .collect::<Vec::<(String, Vec<String>)>>();
        idx += 1;
        if idx > 10 {
//            break
        }
    }

    let datatypes = headers.iter().map(|(k, v)| (k.to_owned(), series_type(v))).collect::<HashMap<String, FlexDataType>>();

    let mut table_series = Vec::<FlexSeries>::new();
    for h in headers {
        let series_vec = h.1.iter().enumerate().map(|(idx, token)|
                            FlexDataPoint::new(FlexIndex::Uint(idx), generate_flexdata_from_str(token, &datatypes[&h.0])))
                            .collect::<Vec::<FlexDataPoint>>();

        table_series.push(FlexSeries::from_vec(&h.0, datatypes[&h.0].clone(), series_vec));
    }

    let table = FlexTable::new(table_series);

    Ok((datatypes, table))
}



fn series_type(series: &Vec<String>) -> FlexDataType {

    if series.iter().map(|s| s.parse::<i64>().is_ok() || *s == "".to_string())
                    .all(|x| x == true) {return FlexDataType::Int}
    else if series.iter().map(|s| s.parse::<f64>().is_ok() || *s == "".to_string())
                        .all(|x| x == true) {return FlexDataType::Dbl}
    else if series.iter().map(|s| NaiveDate::parse_from_str(&s.as_str(), "%d/%m/%y").is_ok()
                                || NaiveDate::parse_from_str(&s.as_str(), "%d/%m/%Y").is_ok()
                                || *s == "".to_string())
                        .all(|x| x == true) {return FlexDataType::Str}
    else {return FlexDataType::Str}
}


pub fn create_master(seasons: Vec<&str>, divisions: Vec<&str>, headers: Vec<&str>, datatypes: Vec<FlexDataType>) -> FlexTable {

    let seed = read_clean(seasons[0], divisions[0], &headers, &datatypes);
    let master_labels: Vec<String> = seed.get_labels().to_vec();
    let master_datatypes: Vec<FlexDataType> = seed.get_datatypes().to_vec();
    let mut master_data: Vec<FlexDataVector> = Vec::new();

    let mut season_series = FlexSeries::new("Ssn", FlexDataType::Uint);
    let mut division_series = FlexSeries::new("Div_n", FlexDataType::Uint);
    let mut date_series = FlexSeries::new("Date_chron", FlexDataType::Uint);
    let mut home_series = FlexSeries::new("Home Points", FlexDataType::Uint);
    let mut away_series = FlexSeries::new("Away Points", FlexDataType::Uint);
    let mut tmp_idx: usize = 0;

    for (ssn, season) in seasons.iter().enumerate() {
        for (div, division) in divisions.iter().enumerate() {
            let table = read_clean(season, division, &headers, &datatypes);

            for game in table.get_data() {
                let mut game_ = game.as_types(&master_datatypes);
                game_.set_index(FlexIndex::Uint(tmp_idx));
                master_data.push(game_);
                let season_point = FlexDataPoint::new(FlexIndex::Uint(tmp_idx), FlexData::Uint(ssn as u32));
                let division_point = FlexDataPoint::new(FlexIndex::Uint(tmp_idx), FlexData::Uint(div as u32));

                let (hp, ap): (u32, u32) = match String::try_from(&game[6]).unwrap().as_str() {
                    "A" => (0, 3),
                    "H" => (3, 0),
                    _ => (1, 1),
                };
                let home_point = FlexDataPoint::new(FlexIndex::Uint(tmp_idx), FlexData::Uint(hp));
                let away_point = FlexDataPoint::new(FlexIndex::Uint(tmp_idx), FlexData::Uint(ap));

                let date_str = String::try_from(&game[1]).unwrap();
                let date_int = NaiveDate::parse_from_str(&date_str, "%d/%m/%y").unwrap().num_days_from_ce();
                let date_point = FlexDataPoint::new(FlexIndex::Uint(tmp_idx), FlexData::Uint(date_int as u32));
                season_series.insert(season_point);
                division_series.insert(division_point);
                home_series.insert(home_point);
                away_series.insert(away_point);
                date_series.insert(date_point);
                tmp_idx += 1;
            }
        }
    }

    let mut master_table = FlexTable::from_vecs(master_labels, master_datatypes, master_data);
    master_table.add_series(season_series);
    master_table.add_series(division_series);
    master_table.add_series(home_series);
    master_table.add_series(away_series);
    master_table.add_series(date_series);
    master_table
}

pub fn read_clean(season: &str, division: &str, headers_: &Vec<&str>, datatypes: &Vec<FlexDataType>) -> FlexTable {

    let headers = headers_.iter().map(|&x| x.to_string()).collect();

    let mut path: String<> = "/home/tony/Projects/football/results/".to_string();
    path.push_str(season);
    path.push_str(division);
    path.push_str(&".csv");

    let raw_table = FlexTable::from_csv(&path, headers, datatypes.to_vec());

    // Filter out records with no data
    let f = |x: &FlexData| x != &FlexData::Str("".to_string());
    let table = raw_table.filter_any(&["Div"], f);

    table
}




