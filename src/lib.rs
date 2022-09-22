use chrono::{TimeZone, Utc};
use colored::*;
use std::fmt::Display;

use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Password, Select};
use home::home_dir;
use rusqlite::{params, Connection, Result};

pub const TABLE_TASKS: &str = "tasks";
pub const TABLE_BOARDS: &str = "boards";

pub const CHOICES: [&str; 7] = [
    "create db",
    "create board",
    "create task",
    "list tasks",
    "list boards",
    "other",
    "Exit",
];

fn get_connection() -> Connection {
    let conn = Connection::open(&get_database_path()).unwrap();
    conn
}

#[derive(Debug)]
pub struct Task {
    id: u16,
    title: String,
    done: u8,
    board_id: u16,
    created_at: String,
    reminder: String,
}

#[derive(Debug)]
pub struct Board {
    id: u16,
    title: String,
    color: String,
}

///Create database table if not exists
pub fn create_database() -> Result<()> {
    let conn = get_connection();
    conn.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {TABLE_BOARDS} (
                  id              INTEGER PRIMARY KEY,
                  title           VARCHAR(255) NOT NULL,
                  color           VARCHAR(255) NOT NULL
                  );"
        ),
        [],
    )?;
    conn.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {TABLE_TASKS} (
                  id              INTEGER PRIMARY KEY,
                  title           VARCHAR(255) NOT NULL,
                  done              INTEGER NOT NULL,
                  board_id          INTEGER NOT NULL,
                  created_at           VARCHAR(255) NOT NULL,
                  reminder           VARCHAR(255) NOT NULL,
                  FOREIGN KEY(board_id) REFERENCES {TABLE_BOARDS}(id)
                );"
        ),
        [],
    )?;
    Ok(())
}

pub fn get_database_path() -> String {
    format!("{}.db3", env!("CARGO_PKG_NAME"))
}
///title, color, id
fn get_board() -> (String, String, u16) {
    let boards_raw = get_boards().unwrap();

    let mut boards: Vec<String> = boards_raw
        .iter()
        .map(|x| format!("{} - {}", &x.id, &x.title))
        .collect();
    boards.sort();

    let selected_board = get_user_selection(&boards, "Board");
    let selected_board_id: u16 = selected_board
        .0
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap();

    let board: &Board = boards_raw
        .iter()
        .find(|x| x.id == selected_board_id)
        .unwrap();

    (board.title.to_string(), board.color.to_string(), board.id)
}

pub fn list_tasks() -> Result<()> {
    let mut title = "All tasks".to_string();
    let all_boards = get_user_confirmation("All boards");

    let tasks = match all_boards {
        true => get_tasks(None).unwrap(),
        false => {
            let (board_title, _, board_id) = get_board();
            title = format!("Tasks for board {}", board_title);
            get_tasks(Some(board_id)).unwrap()
        }
    };

    let mut tasks: Vec<String> = tasks
        .iter()
        .map(|x| {
            format!("{} - {}", &x.id, &x.title)
                .color("green")
                .to_string()
        })
        .collect();

    tasks.sort();

    get_user_selection(&tasks, &title);
    Ok(())
}

pub fn list_boards() -> Result<()> {
    let boards = get_boards().unwrap();
    let text = |x: &Board| format!("{} - {}", &x.id, &x.title);

    let mut boards: Vec<String> = boards
        .iter()
        .map(|x| text(&x).color("yellow").to_string())
        .collect();

    boards.sort();
    get_user_selection(&boards, "Boards");
    Ok(())
}

pub fn other() -> Result<()> {
    // create_board().unwrap();
    // create_task().unwrap();
    println!("other");
    Ok(())
}

//Get singe response from choices
pub fn get_user_selection<T>(items: &Vec<T>, title: &str) -> (String, usize)
where
    T: Display,
{
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .with_prompt(title)
        .default(0)
        .interact()
        .unwrap();

    (items.get(selection).unwrap().to_string(), selection)
}

pub fn display_message(message_type: &str, message: &str, color: &str) {
    let msg = format!("[{}] {}", message_type.to_uppercase(), message).color(color);
    println!("{msg}");
}

pub fn get_tasks(board_id: Option<u16>) -> Result<Vec<Task>> {
    let conn = get_connection();
    let mut query = format!("SELECT * FROM {TABLE_TASKS}");

    if board_id.is_some() {
        let board_id = board_id.unwrap().to_string();
        query = format!("SELECT * FROM {TABLE_TASKS} WHERE board_id = {board_id}");
    }

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

///Get all database connections from database
pub fn get_boards() -> Result<Vec<Board>> {
    let conn = get_connection();
    let query = format!("SELECT * FROM {TABLE_BOARDS}");
    let mut records: Vec<Board> = Vec::new();

    let mut stmt = conn.prepare(&query)?;

    let result_iter = stmt.query_map([], |row| {
        Ok(Board {
            id: row.get(0)?,
            title: row.get(1)?,
            color: row.get(2)?,
        })
    })?;

    for i in result_iter {
        records.push(i?);
    }
    Ok(records)
}

pub fn create_task() -> Result<()> {
    let conn = get_connection();
    let title = get_user_input("Task", "sample", true).unwrap();
    let board_id = get_board().2;
    let created_at = Utc::now().to_string();
    let reminder = Utc::now().to_string();

    let task = Task {
        id: 0,
        done: 0,
        created_at,
        title,
        board_id,
        reminder,
    };

    conn.execute(
        &format!("INSERT INTO {TABLE_TASKS} (title, done, board_id, created_at, reminder) VALUES (?1, ?2, ?3, ?4, ?5)"),
        params![task.title, task.done, task.board_id, task.created_at, task.reminder],
    )
    .unwrap();
    Ok(())
}

pub fn create_board() -> Result<()> {
    let conn = get_connection();
    let title = get_user_input("title", "sample", true).unwrap();
    let color = get_user_input("color", "red", true).unwrap();

    let board = Board {
        id: 0,
        title,
        color,
    };

    conn.execute(
        &format!("INSERT INTO {TABLE_BOARDS} (title, color) VALUES (?1, ?2)"),
        params![board.title, board.color],
    )
    .unwrap();
    Ok(())
}
///Get text response.
fn get_user_input(text: &str, default_text: &str, allow_spaces: bool) -> Option<String> {
    let res: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(text)
        .default(default_text.into())
        .interact_text()
        .unwrap();

    if allow_spaces {
        return Some(res);
    }

    let text_parts = &res.split_ascii_whitespace().count();
    if text_parts != &1_usize {
        display_message("error", "Spaces are not allowed", "red");
        return None;
    }
    let res = res.split_ascii_whitespace().next().unwrap().to_string();
    Some(res)
}

///Get boolean response
fn get_user_confirmation(question: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .default(true)
        .interact()
        .unwrap()
}

// fn main() {
//     let b = Utc.ymd(2022, 10, 1).and_hms(10, 15, 0);
//     println!("{:?}", b > a);
// }
