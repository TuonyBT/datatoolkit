use crate::{FlexTable};
use std::convert::TryFrom;

use std::collections::{BTreeMap};

use ndarray::{Array};
use ndarray_stats::{
    histogram::{Bins, Edges, Grid},
    HistogramExt,
};
use poloto::prelude::*;


pub fn histogram(df: &FlexTable) -> () {

    // Extract series we want to plot
    let obs = &df.extract_series(&["Game_Sup"])[0];
    let raw_obs = obs.get_data().iter().map(|&x| i64::try_from(x).unwrap()).collect::<Vec<i64>>();

    // 1-dimensional observations, as a (n_observations, n_dimension) 2-d matrix
    let observations = Array::from_shape_vec(
        (raw_obs.len(), 1),
        raw_obs,
        ).unwrap();

    let edge_vec = (-26..27).collect::<Vec<i64>>();

    let edges = Edges::from(edge_vec);
    let bins = Bins::new(edges);
    let grid = Grid::from(vec![bins.clone()]);
    let histogram = observations.histogram(grid);

    let histogram_matrix = histogram.counts();
    let his_data: Vec<f64> = histogram_matrix.iter().map(|i| *i as f64 ).collect();

    let trend = his_data;

    let it = (-26..).zip(trend.iter().copied());
    let data = poloto::data(plots!(
        it.cloned_plot().histogram(""),
        poloto::build::markers([], [])
    ));

    let opt = poloto::render::render_opt();
    let (_, by) = poloto::ticks::bounds(&data, &opt);
    let xtick_fmt = poloto::ticks::from_iter((-30..).step_by(6));
    let ytick_fmt = poloto::ticks::from_default(by);

    let pp = poloto::plot_with(
        data,
        opt,
        poloto::plot_fmt(
            "Distribution of games according to match rating",
            "Match Rating",
            "Number of matches",
            xtick_fmt.with_tick_fmt(|w, v| write!(w, "{} adv", v)),
            ytick_fmt,
        ),
    );

    let file = std::fs::File::create("/home/tony/Projects/football/goal_superiorities_hist.svg").unwrap();
    let hist_write = pp.simple_theme(poloto::upgrade_write(file));
    println!("Did histogram write OK? {:?}", hist_write);
}

pub fn model_plot(df: &BTreeMap::<&String, Vec<[f64; 3]>>, parms: &BTreeMap::<&String, (&[f64], &f64)>) -> () {

    for (outcome, plot) in df {
        let mut out_path: String<> = "/home/tony/Projects/football/models/".to_string();
        out_path.push_str(outcome);
        out_path.push_str(&".csv");
        let mut wtr = csv::WriterBuilder::new()
            .from_path(out_path).unwrap();
        match wtr.write_record(&["MatchSup", "Observed", "Model"]) {
            Ok(_v) => {
                for result in plot {
                    let record = *result;
                    match wtr.serialize(record) {
                        Ok(_v) => {
                        },
                        Err(_e) => println!("Record no good for output csv {:?}", record),
                    };
                }
            },
            Err(_e) => println!("Headers no good for output csv {:?}", outcome),
        };
    }
    for (outcome, pms) in parms {

        let mut out_path: String<> = "/home/tony/Projects/football/models/".to_string();
        out_path.push_str(outcome);
        out_path.push_str(&"_parms.csv");
        let mut wtr = csv::WriterBuilder::new()
            .from_path(out_path).unwrap();
        match wtr.write_record(&["parameters"]) {
            Ok(_v) => {
            },
            Err(_e) => println!("Record no good for output csv {:?}", pms),
        };
        let mut pm_out = pms.0.to_vec();
        pm_out.push(*pms.1);
        for pm in pm_out.iter() {
        match wtr.serialize(pm) {
            Ok(_v) => {
            },
            Err(_e) => println!("Record no good for output csv {:?}", pm),
        };

        }
    }
}

pub fn bets_plot(df: &BTreeMap::<(String, i64), Vec<(bool, bool, f64, f64)>>) -> () {

    let out_path: String<> = "/home/tony/Projects/football/models/test_betting.csv".to_string();
    let mut wtr = csv::WriterBuilder::new()
            .from_path(out_path).unwrap();
    match wtr.write_record(&["Outcome", "MatchSup", "Played", "Blind Bets won", "Blind Payouts", "Stakes", "Bets won", "Payouts"]) {
        Ok(_z) => {
            for (k, v) in df {
                let otc = k.0.as_str();
                let ms = k.1;
                let games = v.len();
                let stakes = v.iter().filter(|x| x.0 == true).collect::<Vec<&(bool, bool, f64, f64)>>().len();
                let payouts = v.iter().fold(0.0, |acc, x| acc + x.2);
                let bets_won = v.iter().filter(|x| x.0 == true && x.1 == true).collect::<Vec<&(bool, bool, f64, f64)>>().len();
                let blind_payouts = v.iter().fold(0.0, |acc, x| acc + x.3);
                let blind_bets_won = v.iter().filter(|x| x.1 == true).collect::<Vec<&(bool, bool, f64, f64)>>().len();
                let record = (otc, ms, games, blind_bets_won, blind_payouts, stakes, bets_won, payouts);
                match wtr.serialize(record) {
                    Ok(_z) => {
                    },
                    Err(_e) => println!("Record no good for output csv {:?}", k),
                };
            }
        },
        Err(_e) => println!("Headers no good for output csv"),
    };

}
