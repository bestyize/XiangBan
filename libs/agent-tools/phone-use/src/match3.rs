//! Bounded deterministic planner for settled, plain match-three boards. Not a full game simulator.
use crate::package::Result;
use serde_json::{json,Value};
use std::collections::BTreeSet;

fn matches(board:&[Vec<u8>])->BTreeSet<(usize,usize)> {
    let mut result=BTreeSet::new(); let rows=board.len(); let cols=board[0].len();
    for y in 0..rows { for x in 0..cols {
        let color=board[y][x]; if color==0 {continue;}
        if x+2<cols && board[y][x+1]==color && board[y][x+2]==color {
            let mut end=x; while end<cols && board[y][end]==color {result.insert((y,end));end+=1;}
        }
        if y+2<rows && board[y+1][x]==color && board[y+2][x]==color {
            let mut end=y; while end<rows && board[end][x]==color {result.insert((end,x));end+=1;}
        }
    }}
    result
}

pub fn candidates(board:&[Vec<u8>])->Result<Value> {
    if !(3..=12).contains(&board.len()) || !(3..=12).contains(&board[0].len())
        || board.iter().any(|row|row.len()!=board[0].len()||row.iter().any(|v|*v>32)) {
        return Err("board must be a rectangular 3..12 by 3..12 matrix; 0=blocked/unknown, 1..32=plain colors".into());
    }
    if !matches(board).is_empty() { return Err("board contains existing matches; wait for animations and re-observe".into()); }
    let mut grid=board.to_vec(); let mut moves=Vec::new();
    for y in 0..grid.len() {for x in 0..grid[0].len() {for (y2,x2) in [(y,x+1),(y+1,x)] {
        if y2>=grid.len()||x2>=grid[0].len()||grid[y][x]==0||grid[y2][x2]==0||grid[y][x]==grid[y2][x2] {continue;}
        let first=grid[y][x]; let second=grid[y2][x2]; grid[y][x]=second; grid[y2][x2]=first;
        let cleared=matches(&grid); grid[y][x]=first; grid[y2][x2]=second;
        if !cleared.is_empty() {
            moves.push(json!({"from":{"row":y,"column":x},"to":{"row":y2,"column":x2},"score":cleared.len(),"matched_cells":cleared}));
        }
    }}}
    moves.sort_by_key(|m|std::cmp::Reverse(m["score"].as_u64().unwrap_or(0)));
    let total=moves.len(); moves.truncate(5);
    Ok(json!({"candidates":moves,"total":total,"coordinate_system":"zero-based row/column, not screen pixels",
        "limitations":"Plain colors only; no gravity prediction, special tiles, obstacles or level-goal optimization. Execute one move then re-observe."}))
}
