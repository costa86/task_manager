use crate::user_input::{get_user_confirmation, get_user_selection};
use chrono::{TimeZone, Utc};
use colored::*;
// use home::home_dir;
use rusqlite::{Connection, Result};
use tabled::Tabled;

pub mod dao;
pub mod user_input;
pub const TABLE_TASKS: &str = "tasks";
pub const TABLE_BOARDS: &str = "boards";
pub const TABLE_COMMENTS: &str = "comments";

pub const MAIN_MENU_OPTIONS: [&str; 7] = [
    "Create Task",
    "View Tasks [Pending]",
    "View Tasks [Done]",
    "Create Board",
    "View Boards",
    "other",
    "Exit",
];

const TASK_ACTIONS: [&str; 6] = [
    "Delete",
    "Change",
    "Add comment",
    "View comments",
    "Set reminder",
    "Cancel",
];
pub const BOARD_ACTIONS: [&str; 3] = ["Delete", "Change title", "Cancel"];
pub const SAMPLE_TITLE: &str = "sample";
pub const DATETIME_FORMAT: &str = "%a, %b %e %Y %T";
pub const DATE_FORMAT: &str = "%Y%m%d";
pub const TIME_FORMAT: &str = "%H:%M:%S";
pub const ALTERNATIVE_DATETIME_FORMAT: &str = "%Y%m%d %H:%M:%S";

#[derive(Debug, Tabled)]
pub struct Task {
    pub id: u16,
    pub title: String,
    pub done: u8,
    pub board_id: u16,
    pub created_at: String,
    pub reminder: String,
}

#[derive(Debug)]
pub struct Board {
    pub id: u16,
    pub title: String,
}

#[derive(Debug)]
pub struct Comment {
    pub id: u16,
    pub title: String,
    pub created_at: String,
}

#[derive(Debug)]
pub struct Record {
    pub qtd: u16,
}

pub fn get_connection() -> Connection {
    let conn = Connection::open(&get_database_path()).unwrap();
    conn
}

pub fn get_database_path() -> String {
    format!("{}.db3", env!("CARGO_PKG_NAME"))
}

///title, id
fn select_comment(comments_raw: &Vec<Comment>, task_title: &str) -> Option<(String, u16)> {
    let mut comments: Vec<String> = comments_raw
        .iter()
        .map(|x| format!("{} - {} [{}]", &x.id, &x.title, &x.created_at))
        .collect();
    comments.sort();

    let selected_comment = user_input::get_user_selection(
        &comments,
        format!("Comments for task {}", task_title).as_str(),
    );
    let selected_comment_id: u16 = selected_comment
        .0
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap();

    let selected_comment: &Comment = comments_raw
        .iter()
        .find(|x| x.id == selected_comment_id)
        .unwrap();

    let selected_comment = (selected_comment.title.to_string(), selected_comment.id);
    Some(selected_comment)
}

///title, id
pub fn select_board() -> Option<(String, u16)> {
    let records_qtd = dao::get_records_qtd(TABLE_BOARDS).unwrap();

    if records_qtd == 0 {
        display_message("info", "No Boards found in database", Color::Blue);
        return None;
    }

    let boards_raw = dao::get_boards().unwrap();

    let mut boards: Vec<String> = boards_raw
        .iter()
        .map(|x| format!("{} - {}", &x.id, &x.title))
        .collect();
    boards.sort();

    let selected_board = user_input::get_user_selection(&boards, "Board");
    let selected_board_id: u16 = selected_board
        .0
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap();

    let selected_board: &Board = boards_raw
        .iter()
        .find(|x| x.id == selected_board_id)
        .unwrap();

    let selected_board = (selected_board.title.to_string(), selected_board.id);
    Some(selected_board)
}

pub fn list_boards() -> Result<()> {
    let selected_board = select_board();
    if selected_board.is_none() {
        return Ok(());
    }
    let (board_title, board_id) = selected_board.unwrap();

    let (_, action_index) = get_user_selection(
        &BOARD_ACTIONS.to_vec(),
        format!("Action on Board {}", board_title).as_str(),
    );

    match action_index {
        0 => delete_board(&board_title, board_id)?,
        1 => dao::edit_board(&board_title, board_id)?,
        _ => return Ok(()),
    };

    Ok(())
}

fn delete_board(board_title: &str, board_id: u16) -> Result<()> {
    let deletion_confirmation =
        get_user_confirmation(format!("Are you sure you want to delete {}", &board_title).as_str());

    if deletion_confirmation {
        let deletion_successful = dao::delete_record_by_id(TABLE_BOARDS, board_id);
        match deletion_successful {
            Ok(_) => display_message(
                "ok",
                format!("Board {} has been deleted", &board_title).as_str(),
                Color::Green,
            ),
            Err(_) => display_message(
                "error",
                format!("Could not delete {} ", &board_title).as_str(),
                Color::Red,
            ),
        }
    }
    Ok(())
}

pub fn datetime_str_is_past(reminder: &str) -> bool {
    let datetime = Utc.datetime_from_str(reminder, DATETIME_FORMAT);

    if datetime.is_err() {
        return false;
    }
    datetime.unwrap() < Utc::now()
}

fn delete_task(task_title: &str, task_id: u16) -> Result<()> {
    let deletion_confirmation =
        get_user_confirmation(format!("Are you sure you want to delete {}", &task_title).as_str());

    if deletion_confirmation {
        let deletion_successful = dao::delete_record_by_id(TABLE_TASKS, task_id);
        match deletion_successful {
            Ok(_) => display_message(
                "ok",
                format!("task {} has been deleted", &task_title).as_str(),
                Color::Green,
            ),
            Err(_) => display_message(
                "error",
                format!("Could not delete {} ", &task_title).as_str(),
                Color::Red,
            ),
        }
    }
    Ok(())
}

pub fn list_tasks(done: u8) -> Result<()> {
    let selected_task = dao::select_task(done);
    if selected_task.is_none() {
        return Ok(());
    }

    let (task_title, task_id) = selected_task.unwrap();

    let (_, action_index) = get_user_selection(
        &TASK_ACTIONS.to_vec(),
        format!("Action on Task {}", task_title).as_str(),
    );

    match action_index {
        0 => delete_task(&task_title, task_id)?,
        1 => dao::switch_task_status(task_id)?,
        2 => dao::create_comment(task_id)?,
        3 => list_comments(&task_title, task_id)?,
        4 => dao::set_reminder(task_id)?,
        _ => return Ok(()),
    };

    Ok(())
}

pub fn list_comments(task_title: &str, task_id: u16) -> Result<()> {
    let comments = dao::get_comments_by_task_id(task_id)?;
    if comments.len() == 0 {
        display_message("info", "No comments for this Task", Color::Cyan);
        return Ok(());
    }

    let selected_comment = select_comment(&comments, &task_title).unwrap();
    println!("{:?}", selected_comment);

    Ok(())
}

#[derive(Debug)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

pub fn other() -> Result<()> {
    Ok(())
}

pub fn display_message(message_type: &str, message: &str, color: Color) {
    let color = match color {
        Color::Red => "red",
        Color::Green => "green",
        Color::Yellow => "yellow",
        Color::Blue => "blue",
        Color::Magenta => "magenta",
        Color::Cyan => "cyan",
        Color::White => "white",
    };

    let msg = format!("[{}] {}", message_type.to_uppercase(), message).color(color);
    println!("{msg}");
}

pub fn get_tasks(query: &str) -> Result<Vec<Task>> {
    let conn = get_connection();

    let mut records: Vec<Task> = Vec::new();

    let mut stmt = conn.prepare(&query)?;

    let result_iter = stmt.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            done: row.get(2)?,
            board_id: row.get(3)?,
            created_at: row.get(4)?,
            reminder: row.get(5)?,
        })
    })?;

    for i in result_iter {
        records.push(i?);
    }

    Ok(records)
}
