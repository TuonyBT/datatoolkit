use std::convert::TryFrom;
use std::collections::{BTreeSet, BTreeMap};
use std::fs::File;

use itertools::Itertools;

use datatoolkit::{FlexIndex, FlexDataVector, FlexSeries, FlexData, FlexDataType, FlexTable};
use datatoolkit::football::{
graphs::{histogram, model_plot, bets_plot},
models::{linear, linear_gd, logistic_gd},
feature_add::{goal_superiority, gs_summ, logistic_gs_summ},
data_check::{checkstats, final_tables},
data_prep::{create_master, get_tables}
};

use ndarray::{Array, Array2, Axis};
use chrono::prelude::*;


fn main() -> std::io::Result<()> {

    let path: String<> = "/home/tony/Projects/football/results/".to_string();

//  Only need to run this section once to create a json file containing the data types for all headers found
    let seasons: Vec<&str> = vec!["9394", "9495", "9596", "9697", "9798", "9899", "9900", "0001",
                                    "0102", "0203", "0304", "0405", "0506", "0607", "0708", "0809",
                                     "0910", "1011", "1112", "1213", "1314", "1415", "1516", "1617",
                                        "1718", "1819", "1920", "2021", "2122"];
//    let divisions: Vec<&str> = vec!["E0", "E1", "E2", "E3", "EC"];
//    let header_file = File::create(path.as_str().to_owned() + "all_headers.json")?;
//    let all_headers = get_tables(&path, seasons, divisions, false);
//    serde_json::to_writer(header_file, &all_headers)?;
//  End of section to pull out field data types

//  ... and only need to run this section once to create clean versions of all the data files
//    let seasons: Vec<&str> = vec!["9394", "9495"];
    let divisions: Vec<&str> = vec!["E0", "E1", "E2", "E3", "EC"];
    let sel_tables = get_tables(&path, seasons, divisions, true);
    let mut table_set = BTreeMap::<String, BTreeMap::<String, BTreeMap::<String, Vec<FlexData>>>>::new();
    let mut division_set = BTreeMap::<String, BTreeMap::<String, Vec<FlexData>>>::new();
    for (key, tbl) in sel_tables.0 {
        println!("{:?}", key);
        let mut game_set = BTreeMap::<String, Vec<FlexData>>::new();
        for game in tbl.get_data() {
            let date_str = String::try_from(&game[1]).unwrap();
//            let date_int = NaiveDate::parse_from_str(&date_str, "%d/%m/%y").unwrap().num_days_from_ce();
            let ht = date_str + "_" + &String::try_from(&game[2]).unwrap();

            let game_data = &game.get_data()[1..];
            game_set.insert(ht, game_data.to_vec());
        }
//        division_set.insert(key[1].to_owned(), game_set);
        table_set.entry(key[0].to_owned()).or_insert(BTreeMap::<String, BTreeMap::<String, Vec<FlexData>>>::new()).insert(key[1].to_owned(), game_set);
    }
    let tables_file = File::create(path.as_str().to_owned() + "all_tables.json")?;
    serde_json::to_writer(tables_file, &table_set)?;
//  End of section to create clean data files

    Ok(())
}

fn main_() {

    let seasons: Vec<&str> = vec!["9394", "9495", "9596", "9697", "9798", "9899", "9900", "0001"];
    let divisions: Vec<&str> = vec!["E0", "E1", "E2", "E3"];
    let headers: Vec<&str> = vec!["Div","Date","HomeTeam","AwayTeam","FTHG","FTAG","FTR"];
    let datatypes: Vec<FlexDataType> = vec![
        FlexDataType::Str,
        FlexDataType::Str,
        FlexDataType::Str,
        FlexDataType::Str,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Str,
    ];

//    let master_table = create_master(seasons, divisions, headers, datatypes);

//    master_table.to_csv("/home/tony/Projects/football/results/master.csv");
//    checkstats(&master_table);
//    final_tables(&master_table);

//    let new_master_table = goal_superiority(&master_table);
//    new_master_table.to_csv("/home/tony/Projects/football/results/master_with_goalsuperiority.csv");

//  re-import new master table to save run time
    let new_headers: Vec<String> = vec!["", "Div","Date","HomeTeam","AwayTeam","FTHG","FTAG","FTR",
                                "Ssn", "Div_n", "Home Points", "Away Points", "Date_chron", "Home_Sup", "Away_Sup", "Game_Sup"]
                                .iter().map(|&x| x.to_string()).collect();
    let new_datatypes: Vec<FlexDataType> = vec![
        FlexDataType::Uint,
        FlexDataType::Str,
        FlexDataType::Str,
        FlexDataType::Str,
        FlexDataType::Str,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Str,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Int,
        FlexDataType::Int,
        FlexDataType::Int,
    ];

    let new_input_table = FlexTable::from_csv("/home/tony/Projects/football/results/master_with_goalsuperiority.csv",
                                                new_headers, new_datatypes.to_vec());

    let labels: Vec<String> = new_input_table.get_labels()[1..].to_vec();
    let datatypes: Vec<FlexDataType> = new_input_table.get_datatypes()[1..].to_vec();
    let mut train: Vec<FlexDataVector> = Vec::new();
    let mut test: Vec<FlexDataVector> = Vec::new();
    for game in new_input_table.get_data() {

        let tmp_idx = u32::try_from(&game.get_data()[0]).unwrap();
        let game_ = FlexDataVector::new(FlexIndex::Uint(tmp_idx as usize), game.get_data()[1..].to_vec());

// controls which records go into training set vs test set
//        if tmp_idx % 140 == 0 {
        if true {
            train.push(game_);
        }
        else {
            test.push(game_);
        }
    }
    let test_labels = labels.clone();
    let test_datatypes = datatypes.clone();
    let new_master_table = FlexTable::from_vecs(labels, datatypes, train);
    let test_set_table = FlexTable::from_vecs(test_labels, test_datatypes, test);


//    histogram(&new_master_table);

    let gs_results = gs_summ(&new_master_table);
    let gs_data = logistic_gs_summ(&new_master_table);

// Define the features we want to model and put them into the X (i.e. features) matrix
    let features_vec = gs_data.iter().map(|(k, _v)| vec![1 as f64, *k as f64, k.pow(2) as f64]).collect::<Vec<Vec<f64>>>();

// Set up the different response variables we want to model
    let outcomes = [String::from("Home Win"), String::from("Draw"), String::from("Away Win")];
    let mut responses = BTreeMap::<&String, Vec<f64>>::new();
    for (idx, outcome) in outcomes.iter().enumerate() {
    //  Extract the observations that correspond to these features for the response we want to model
//        responses.insert(outcome, gs_results.iter().map(|(_k, v)| v[idx] * 100.0).collect::<Vec<f64>>());
        responses.insert(outcome, gs_data.iter().map(|(_k, v)| v[idx]).collect::<Vec<f64>>());
    }

    let (l_fits, stds) = linear_gd(&features_vec, &responses);

//  TESTING LOGISTIC REGRESSION
    let logistic_features_vec = gs_data.iter().map(|(k, _v)| vec![1 as f64, *k as f64, k.pow(2) as f64]).collect::<Vec<Vec<f64>>>();
    let (logistic_fits, logistic_stds) = logistic_gd(&logistic_features_vec, &responses);

    println!("Logistic fits {:?}", logistic_fits);

//  FINISHED TESTING LOGISTIC REGRESSION


    let mut plot_dict = BTreeMap::<&String, Vec<[f64; 3]>>::new();
    let mut parm_dict = BTreeMap::<&String, (&[f64], &f64)>::new();
    let n_obs = gs_data.len();
    println!("Number of observations to plot: {}", n_obs);

    let mut gs_data_grouped = BTreeMap::<i64, Vec<[f64; 3]>>::new();
    for (k, v) in gs_data.iter() {
        gs_data_grouped.entry(*k).or_insert(Vec::<[f64; 3]>::new()).push(*v);
    }
    let gs_hist = gs_data_grouped.iter().map(|(k, v)| (*k, v.len())).collect::<BTreeMap<i64, usize>>();

    for (idx, outcome) in outcomes.iter().enumerate() {
        let mut tmp_dict = BTreeMap::<i64, Vec<f64>>::new();
        let plot_data = &gs_data.iter().map(|(k, v)| (*k as f64, v[idx] * 100.0)).collect::<Vec<(f64, f64)>>();


        let mut plt_d = l_fits[outcome].0.iter().zip(plot_data.iter()).map(|(&y_hat, &(x, y))| [x, y, y_hat]).collect::<Vec<[f64; 3]>>();
        let mut plt_dt = l_fits[outcome].0.iter().zip(plot_data.iter()).map(|(&y_hat, &(x, y))| (x as i64, [y, y_hat])).collect::<Vec<(i64, [f64; 2])>>();

        let mut data_grouped = BTreeMap::<i64, Vec<[f64; 2]>>::new();
        for (k, v) in plt_dt.iter() {
            data_grouped.entry(*k).or_insert(Vec::<[f64; 2]>::new()).push(*v);
        }

        let mut points = Vec::<[f64; 3]>::new();
        for (k, v) in data_grouped.iter() {
            let accs = v.iter().fold(vec![0.0 , 0.0], |acc, x| vec![acc[0] + x[0] / gs_hist[k] as f64, x[1]]);
            points.push([*k as f64, accs[0], accs[1]]);
        }

        plot_dict.insert(outcome, points);


        let parm_d = l_fits[outcome].1.as_slice().unwrap();
        parm_dict.insert(outcome, (parm_d, &l_fits[outcome].2));
    }

    model_plot(&plot_dict, &parm_dict);

//  This function just prints the parameters from a closed-form linear regression
//    linear(&gs_results);

// Repeat the reading of a file with betting odds included so we can back-test performance

    let seasons: Vec<&str> = vec!["0102"];
    let divisions: Vec<&str> = vec!["E0", "E1", "E2", "E3"];
//    let divisions: Vec<&str> = vec!["E2"];
    let headers: Vec<&str> = vec!["Div","Date","HomeTeam","AwayTeam","FTHG","FTAG","FTR",
    "HTHG","HTAG","HTR","Attendance","Referee","HS","AS","HST","AST","HHW","AHW","HC","AC","HF","AF",
    "HO","AO","HY","AY","HR","AR","HBP","ABP","GBH","GBD","GBA","IWH","IWD","IWA","LBH","LBD","LBA",
    "SBH","SBD","SBA", "SYH","SYD","SYA","WHH","WHD","WHA"];
    let datatypes: Vec<FlexDataType> = vec![
        FlexDataType::Str,
        FlexDataType::Str,
        FlexDataType::Str,
        FlexDataType::Str,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Str,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Str,
        FlexDataType::Uint,
        FlexDataType::Str,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Uint,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
        FlexDataType::Dbl,
    ];

    let test_table = create_master(seasons, divisions, headers, datatypes);
    test_table.to_csv("/home/tony/Projects/football/models/initial_results_interim.csv");

    let new_test_table = goal_superiority(&test_table);
    new_test_table.to_csv("/home/tony/Projects/football/models/initial_results_backtest.csv");

    let mut fair_odds = BTreeMap::<i64, BTreeMap<&String, f64>>::new();

    for outcome in l_fits.keys() {
        let mut score_map = Vec::<(i64, (&String, f64))>::new();
        for score in -30..31 {
            let features:Vec<f64> = vec![1 as f64, score as f64, (score as i32).pow(2) as f64];
            let ft_arr = Array::from_shape_vec((3, 1), features).unwrap();
            let mut theta = l_fits[outcome].1.clone();
            let wts = stds.broadcast((1, 3)).unwrap();
            theta.zip_mut_with(&wts.t(), |x, s| *x *= s);
            let y_hat = theta.t().dot(&ft_arr).sum() as f64;
            score_map.push((score as i64, (outcome, 100.0 / y_hat)));
        }
        for (score, f) in score_map {
            fair_odds.entry(score).or_insert(BTreeMap::new()).insert(f.0, f.1);
        }
    }

    let mut betting = BTreeMap::<(String, i64), Vec<(bool, bool, f64, f64)>>::new();
    for (gi, game) in new_test_table.get_data().iter().enumerate() {
        let game_data = game.get_data();
        let gk = (&game_data[game_data.len() - 4], &game_data[2], &game.get_data()[3]);
        let match_sup = &game.get_data()[game_data.len() - 1];
        let score = i64::try_from(match_sup).unwrap();

        let odds = game.get_data()[30..48].into_iter()
                    .map(|z| {
            match f64::try_from(z) {
                Ok(i) => (i * 10000.0) as usize,
                Err(_j) => 0,
            }
            })
            .collect::<Vec<usize>>();
        let odds_array = Array::from_shape_vec((6, 3), odds).unwrap();


        for (idx, outcome) in outcomes.iter().enumerate() {

            let mut outcome_odds = odds_array.column(idx).to_vec();
            outcome_odds.sort();
            let best_odds = *outcome_odds.last().unwrap() as f64 * 0.01;
            let fair = fair_odds[&score][outcome];
            let result = String::try_from(&game_data[6]).unwrap();
            let bet = best_odds > (fair * 100.0);
            let payout = &result.as_str()[..1] == &outcome.as_str()[..1];
            let blind_winnings = match payout {
                true => best_odds,
                false => 0.0,
            };
            let winnings = match bet && payout {
                true => best_odds,
                false => 0.0,
            };
            betting.entry((outcome.to_string(), score)).or_insert(Vec::<(bool, bool, f64, f64)>::new()).push((bet, payout, winnings, blind_winnings));
        }
    }

    bets_plot(&betting);


}


