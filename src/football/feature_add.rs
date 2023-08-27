use itertools::{Itertools, EitherOrBoth};
use std::collections::{HashMap, BTreeMap};
use std::convert::TryFrom;

use crate::{FlexIndex, FlexData, FlexDataVector, FlexDataType, FlexTable};


pub fn goal_superiority(master_table: &FlexTable) -> FlexTable {

    let mut goal_superiorities: BTreeMap<(u32, String), i64> = BTreeMap::new();

    let ss = FlexTable::group_by(&master_table, "Ssn");

    let mut s_sort: Vec<_> = ss.iter().map(|(k, _v)| k).collect();
    s_sort.sort();

    for s in s_sort {

        let mut team_games: HashMap<String, Vec<FlexTable>> = HashMap::new();
        for (t,t_games) in FlexTable::group_by(&ss[s], "HomeTeam") {
            let gds = FlexTable::new(t_games.extract_series(&["Date_chron", "FTHG", "FTAG"]));
            team_games.entry(t).or_insert(Vec::<FlexTable>::new()).push(gds);
        }
        for (t,t_games) in FlexTable::group_by(&ss[s], "AwayTeam") {
            let gds = FlexTable::new(t_games.extract_series(&["Date_chron", "FTHG", "FTAG"]));
            team_games.entry(t).or_insert(Vec::<FlexTable>::new()).push(gds);
        }

        for team in team_games {
            let mut goal_sup:Vec<(u32, i64)> = Vec::new();
            let mut favour: i64 = 1;
            for h_a in team.1 {
                for g in h_a.get_data() {
                    let gd = favour * (u32::try_from(&g.get_data()[1]).unwrap() as i64 - u32::try_from(&g.get_data()[2]).unwrap() as i64);
                    goal_sup.push((u32::try_from(&g.get_data()[0]).unwrap(), gd));
                }
            favour *= -1;
            }
            goal_sup.sort();
            for (idx, game) in goal_sup.iter().enumerate() {
                if idx >= 6 {
                    let first =  &goal_sup[(idx - 6)..idx];
                    let score = first.iter().fold(0, |acc, (_d, gs)| acc + gs);
                    goal_superiorities.insert((game.0, team.0.to_string()), score);
                }
            }
        }
    }

    let mut core_data: BTreeMap<(u32, String), &FlexDataVector> = BTreeMap::new();

    for game in master_table.get_data() {
        let key = (u32::try_from(&game.get_data()[&game.get_data().len() - 1]).unwrap(), String::try_from(&game.get_data()[2]).unwrap());
        core_data.insert(key, &game);
    }

    let joined_feature = core_data
        .into_iter()
        .merge_join_by(&goal_superiorities, |(key_a, _), (key_b, _)| Ord::cmp(key_a, key_b))
        .filter_map(|elem| match elem {
            EitherOrBoth::Both((k, val_a), (_, val_b)) => Some((k, (val_a, val_b))),
            _ => None,
        })
        .collect::<BTreeMap<(u32, String), (&FlexDataVector, &i64)>>();

    let mut core_data: BTreeMap<(u32, String), (&FlexDataVector, &i64)> = BTreeMap::new();
    for (_k, game) in joined_feature {
        let key = (u32::try_from(&game.0.get_data()[&game.0.get_data().len() - 1]).unwrap(), String::try_from(&game.0.get_data()[3]).unwrap());
        core_data.insert(key, game);
    }

    let joined_feature = core_data
        .into_iter()
        .merge_join_by(&goal_superiorities, |(key_a, _), (key_b, _)| Ord::cmp(key_a, key_b))
        .filter_map(|elem| match elem {
            EitherOrBoth::Both((k, val_a), (_, val_b)) => Some((k, (val_a, val_b))),
            _ => None,
        })
        .collect::<BTreeMap<(u32, String), ((&FlexDataVector, &i64), &i64)>>();


    let mut merged_data = Vec::<FlexDataVector>::new();
    let mut tmp_idx: usize = 0;
    for game in joined_feature {
        let mut game_data = game.1.0.0.get_data().to_vec();
        let hsup = FlexData::Int(*game.1.0.1);
        let asup = FlexData::Int(*game.1.1);
        let gsup = FlexData::Int(*game.1.0.1 - *game.1.1);
        game_data.push(hsup);
        game_data.push(asup);
        game_data.push(gsup);
        merged_data.push(FlexDataVector::new(FlexIndex::Uint(tmp_idx), game_data));
        tmp_idx += 1;
    }

    let mut new_labels = master_table.get_labels().to_vec();
    new_labels.push("Home_Sup".to_string());
    new_labels.push("Away_Sup".to_string());
    new_labels.push("Game_Sup".to_string());

    let mut new_dtypes = master_table.get_datatypes().to_vec();
    new_dtypes.push(FlexDataType::Int);
    new_dtypes.push(FlexDataType::Int);
    new_dtypes.push(FlexDataType::Int);

    let merged_table = FlexTable::from_vecs(new_labels, new_dtypes, merged_data);

    merged_table
}

pub fn gs_summ(table: &FlexTable) -> BTreeMap::<i64, Vec<f64>> {

    // Group by Goal Superiority
    let mut res = BTreeMap::<i64, Vec<(String, usize)>>::new();
    for (k,v) in FlexTable::group_by(table, "Game_Sup") {
        let mut summary = Vec::<(String, usize)>::new();
        for (r, gs) in FlexTable::group_by(&v, "FTR") {
            summary.push((r, gs.num_records()));
        }
        if summary.len() < 3 {
            let ks = summary.iter().map(|x| x.0.to_string()).collect::<Vec<String>>();
            for r in vec!["H", "D", "A"] {
                if !ks.iter().any(|z| z == r) {
                    summary.push((r.to_string(), 0));
                }
            }
        }
        res.insert(k.parse::<i64>().unwrap(), summary);
    }

    let res_full = res.into_iter().map(|rs| {
        let mut rsf = rs.1.clone(); //Vec::<(String, usize)>::new();
        rsf.sort();
        rsf.reverse();
        let total = rs.1.iter().fold(0, |acc, s| acc + s.1);
        let percs = rsf.iter().map(|x| (x.1 as f64) / total as f64).collect::<Vec<f64>>();
        rsf.push(("Total".to_string(), total));
        (rs.0, percs)
    }
    ).collect::<BTreeMap::<i64, Vec<f64>>>();

    res_full
}


pub fn logistic_gs_summ(table: &FlexTable) -> Vec::<(i64, [f64; 3])> {

    let mut res_full = Vec::<(i64, [f64; 3])>::new();
    for game in table.get_data().iter() {
        let m_sup = i64::try_from(game.get_data().last().unwrap()).unwrap();
        let m_res = String::try_from(&game.get_data()[6]).unwrap();
        let mut res_one_hot = [0.0 as f64; 3];
        for (i, r) in vec!["H", "D", "A"].iter().enumerate() {
            if m_res.as_str() == *r {
                res_one_hot[i] = 1.0;
            }
        }
        res_full.push((m_sup, res_one_hot));
    }

    res_full
}

