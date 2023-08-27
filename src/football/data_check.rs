use crate::{FlexTable};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;


pub fn checkstats(master_table: &FlexTable) -> () {

    println!("Total games in master table: {} ", master_table.num_records());

    let ss = FlexTable::group_by(&master_table, "Ssn");
    let mut s_sort: Vec<_> = ss.iter().map(|(k, _v)| k).collect();
    s_sort.sort();

    for s in s_sort {
        println!("Season: {} Number of games: {:?}", s, ss[s].num_records());

        let ds = FlexTable::group_by(&ss[s], "Div_n");
        let mut d_sort: Vec<_> = ds.iter().map(|(k, _v)| k).collect();
        d_sort.sort();
        for d in d_sort {
            println!("Division: {} Number of games: {:?}", d, ds[d].num_records());

            let mut home_played: HashMap<usize, Vec<String>> = HashMap::new();
            for (t,t_games) in FlexTable::group_by(&ds[d], "HomeTeam") {
                home_played.entry(t_games.num_records()).or_insert(Vec::<String>::new()).push(t);
            }
//            println!("Division: {} Number of home games played by each team: {:?}", d, home_played.keys());

            let mut away_played: HashMap<usize, Vec<String>> = HashMap::new();
            for (t,t_games) in FlexTable::group_by(&ds[d], "AwayTeam") {
                away_played.entry(t_games.num_records()).or_insert(Vec::<String>::new()).push(t);
            }
            let mut teams: HashSet<String> = HashSet::new();
            for v in home_played.values() {
                for t in v {
                    teams.insert(t.to_string());
                }
            }
            for v in home_played.values() {
                for t in v {
                    teams.insert(t.to_string());
                }
            }
//            println!("Division: {} Number of away games played by each team: {:?}", d, away_played.keys());
            println!("Division: {} Number of teams: {:?}", d, teams.len());
//            println!("Division: {} Teams: {:?}", d, teams);

        }
    println!("");

    }
}

pub fn final_tables(master_table: &FlexTable) -> () {

    let mut teams_campaigns: HashMap<String, HashSet<(u32, u32)>> = HashMap::new();
    let mut teams_points: HashMap<(u32, u32, String), Vec<(u32, i64)>> = HashMap::new(); //prototype for multi-field groupby
    for game in master_table.get_data() {
        let div = u32::try_from(&game[8]).unwrap();
        let ssn = u32::try_from(&game[7]).unwrap();
        let team = String::try_from(&game[2]).unwrap();

        teams_campaigns.entry(team).or_insert(HashSet::<(u32, u32)>::new()).insert((ssn, div));

        let team = String::try_from(&game[2]).unwrap();
        let home_points = u32::try_from(&game[9]).unwrap();
        let hgd: i64 = u32::try_from(&game[4]).unwrap() as i64 - u32::try_from(&game[5]).unwrap() as i64;

        teams_points.entry((ssn, div, team)).or_insert(Vec::new()).push((home_points, hgd));

        let opposition = String::try_from(&game[3]).unwrap();
        let away_points = u32::try_from(&game[10]).unwrap();
        let agd: i64 = u32::try_from(&game[5]).unwrap() as i64 - u32::try_from(&game[4]).unwrap() as i64;

        teams_points.entry((ssn, div, opposition)).or_insert(Vec::new()).push((away_points, agd));
    }
    let mut camp: Vec<_> = teams_campaigns.iter().collect();
    camp.sort_by_key(|a| a.0);

    let mut tot_team_camp: usize = 0;
    for (team, campaigns) in camp.iter() {
        println!("Team: {} participated in {:?} campaigns", team, campaigns.len());
        tot_team_camp += campaigns.len();
    }
    println!("Total campaigns by all participating teams = {}", tot_team_camp);
    println!("");


    let mut final_tables:HashMap<(u32, u32), Vec<(String, Vec<(u32, i64)>)>> = HashMap::new();
    for (cmpgn, tm) in teams_points {
        final_tables.entry((cmpgn.0, cmpgn.1)).or_insert(Vec::new()).push((cmpgn.2, tm));
    }
    let mut fts: Vec<_> = final_tables.iter().collect();
    fts.sort_by_key(|a| a.0);
    for ft in fts {
        println!("Final Table for Season {:?} and Division {:?}:", ft.0.0, ft.0.1);
        let mut tm_pts: Vec<_> = ft.1.iter().map(|x| (&x.0,
        (x.1.iter().map(|&y| y.0).collect::<Vec<u32>>().iter().sum(), x.1.iter().map(|&y| y.1).collect::<Vec<i64>>().iter().sum())
        )).collect::<Vec<(&String, (u32, i64))>>();
        tm_pts.sort_by_key(|a| a.1);
        tm_pts.reverse();
        for tm in tm_pts {
            println!("{:?}:", tm);
        }

        println!("");
    }


}


