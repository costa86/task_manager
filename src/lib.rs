use crate::user_input::{get_user_confirmation, get_user_selection};
use chrono::{TimeZone, Utc};
use colored::*;
use home::home_dir;
use rusqlite::{params, Connection, Result};

pub mod user_input;

pub const TABLE_TASKS: &str = "tasks";
pub const TABLE_BOARDS: &str = "boards";
pub const TABLE_COMMENTS: &str = "comments";

pub const MAIN_MENU_OPTIONS: [&str; 7] = [
    "Create Board",
    "Create Task",
    "View Tasks [Done]",
    "View Tasks [Pending]",
    "View Boards",
    "other",
    "Exit",
];

const TASK_ACTIONS: [&str; 5] = ["Delete", "Change", "Add comment", "View comments", "Cancel"];
const BOARD_ACTIONS: [&str; 3] = ["Delete", "Change title", "Cancel"];
const SAMPLE_TITLE: &str = "sample";

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
}

#[derive(Debug)]
pub struct Comment {
    id: u16,
    title: String,
    task_id: u16,
    created_at: String,
}

#[derive(Debug)]
struct Record {
    qtd: u16,
}

fn get_connection() -> Connection {
    let conn = Connection::open(&get_database_path()).unwrap();
    conn
}

///Create database table if not exists
pub fn create_database() -> Result<()> {
    let conn = get_connection();

    //boards
    conn.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {TABLE_BOARDS} (
                  id              INTEGER PRIMARY KEY,
                  title           VARCHAR(255) NOT NULL
                  );"
        ),
        [],
    )?;

    //tasks
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

    //comments
    conn.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {TABLE_COMMENTS} (
                  id              INTEGER PRIMARY KEY,
                  title           VARCHAR(255) NOT NULL,
                  task_id          INTEGER NOT NULL,
                  created_at           VARCHAR(255) NOT NULL,
                  FOREIGN KEY(task_id) REFERENCES {TABLE_TASKS}(id)
                  );"
        ),
        [],
    )?;

    Ok(())
}

pub fn get_database_path() -> String {
    format!("{}.db3", env!("CARGO_PKG_NAME"))
}

///title, id
fn select_comment(comments_raw: &Vec<Comment>) -> Option<(String, u16)> {
    let mut comments: Vec<String> = comments_raw
        .iter()
        .map(|x| format!("{} - {} ({})", &x.id, &x.title, &x.created_at))
        .collect();
    comments.sort();

    let selected_comment = user_input::get_user_selection(&comments, "Comment");
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
fn select_board() -> Option<(String, u16)> {
    let records_qtd = get_records_qtd(TABLE_BOARDS).unwrap();

    if records_qtd == 0 {
        display_message("info", "No Boards found in database", "blue");
        return None;
    }

    let boards_raw = get_boards().unwrap();

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
        format!("Action on {}", board_title).as_str(),
    );

    match action_index {
        0 => delete_board(&board_title, board_id)?,
        1 => edit_board(&board_title, board_id)?,
        _ => return Ok(()),
    };

    Ok(())
}

fn edit_board(title: &str, id: u16) -> Result<()> {
    let conn = get_connection();
    let title = user_input::get_user_input("New Board title", title, true).unwrap();

    conn.execute(
        &format!("UPDATE {TABLE_BOARDS} SET title = ?1 WHERE id = ?2"),
        params![title, id],
    )
    .unwrap();
    Ok(())
}

fn delete_board(board_title: &str, board_id: u16) -> Result<()> {
    let deletion_confirmation =
        get_user_confirmation(format!("Are you sure you want to delete {}", &board_title).as_str());

    if deletion_confirmation {
        let deletion_successful = delete_record_by_id(TABLE_BOARDS, board_id);
        match deletion_successful {
            Ok(_) => display_message(
                "ok",
                format!("Board {} has been deleted", &board_title).as_str(),
                "green",
            ),
            Err(_) => display_message(
                "error",
                format!("Could not delete {} ", &board_title).as_str(),
                "red",
            ),
        }
    }
    Ok(())
}

///title, id
fn select_task(done: u8) -> Option<(String, u16)> {
    let records_qtd = get_records_qtd(TABLE_TASKS).unwrap();
    if records_qtd == 0 {
        display_message("info", "No Tasks found in database", "blue");
        return None;
    }
    let mut query = format!("SELECT * FROM {TABLE_TASKS} WHERE done = {done}");

    let all_boards = user_input::get_user_confirmation("From all boards");

    if !all_boards {
        let (_, board_id) = select_board().unwrap();
        query.push_str(&format!(" AND board_id = {board_id}"));
    }

    let tasks_raw = get_tasks(&query).unwrap();
    if tasks_raw.len() == 0 {
        display_message(
            "info",
            "No Tasks found in database with these criteria",
            "blue",
        );
        return None;
    }

    let mut tasks: Vec<String> = tasks_raw
        .iter()
        .map(|x| format!("{} - {}", &x.id, &x.title))
        .collect();

    tasks.sort();

    let (selected_task, _) = get_user_selection(&tasks, "title");

    let task_id: u16 = selected_task
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap();

    let selected_task = tasks_raw.iter().find(|x| x.id == task_id).unwrap();
    let selected_task = (selected_task.title.to_string(), selected_task.id);
    Some(selected_task)
}

fn delete_task(task_title: &str, task_id: u16) -> Result<()> {
    let deletion_confirmation =
        get_user_confirmation(format!("Are you sure you want to delete {}", &task_title).as_str());

    if deletion_confirmation {
        let deletion_successful = delete_record_by_id(TABLE_TASKS, task_id);
        match deletion_successful {
            Ok(_) => display_message(
                "ok",
                format!("task {} has been deleted", &task_title).as_str(),
                "green",
            ),
            Err(_) => display_message(
                "error",
                format!("Could not delete {} ", &task_title).as_str(),
                "red",
            ),
        }
    }
    Ok(())
}

pub fn list_tasks(done: u8) -> Result<()> {
    let selected_task = select_task(done);
    if selected_task.is_none() {
        return Ok(());
    }

    let (task_title, task_id) = selected_task.unwrap();

    let (_, action_index) = get_user_selection(
        &TASK_ACTIONS.to_vec(),
        format!("Action on {}", task_title).as_str(),
    );

    match action_index {
        0 => delete_task(&task_title, task_id)?,
        1 => switch_task_status(task_id)?,
        2 => create_comment(task_id)?,
        3 => list_comments(task_id)?,
        _ => return Ok(()),
    };

    Ok(())
}

fn list_comments(task_id: u16) -> Result<()> {
    let comments = get_comments_by_task_id(task_id)?;

    let selected_comment = select_comment(&comments).unwrap();
    println!("{:?}", selected_comment);

    Ok(())
}

fn get_comments_by_task_id(task_id: u16) -> Result<Vec<Comment>> {
    let conn = get_connection();
    let query = format!("SELECT * FROM {TABLE_COMMENTS} WHERE task_id = {task_id}");
    let mut comments: Vec<Comment> = Vec::new();

    let mut stmt = conn.prepare(&query)?;

    let result_iter = stmt.query_map([], |row| {
        Ok(Comment {
            id: row.get(0)?,
            title: row.get(1)?,
            task_id: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    for i in result_iter {
        comments.push(i?);
    }
    Ok(comments)
}

fn switch_task_status(task_id: u16) -> Result<()> {
    let conn = get_connection();

    let done: u8 = match user_input::get_user_confirmation("Done") {
        true => 1,
        false => 0,
    };

    conn.execute(
        &format!("UPDATE {TABLE_TASKS} SET done = ?1 WHERE id = ?2"),
        params![done, task_id],
    )
    .unwrap();
    Ok(())
}

//Delete database records
fn delete_record_by_id(table: &str, id: u16) -> Result<()> {
    let conn = get_connection();
    conn.execute(&format!("DELETE FROM {table} WHERE id = ?1"), params![id])?;
    Ok(())
}

pub fn other() -> Result<()> {
    // let a = select_task();
    // create_board().unwrap();
    // create_task().unwrap();
    // let year: i32 = get_user_input("year", "2022", false)
    //     .unwrap()
    //     .parse()
    //     .unwrap();
    // let month: u32 = get_user_input("month", "9", false)
    //     .unwrap()
    //     .parse()
    //     .unwrap();
    // let day: u32 = get_user_input("day", "22", false).unwrap().parse().unwrap();
    // let b = Utc.ymd(year, month, day).and_hms(10, 15, 0);

    // println!("{:?}", b);

    // println!("{:?}", a);
    Ok(())
}

pub fn display_message(message_type: &str, message: &str, color: &str) {
    let msg = format!("[{}] {}", message_type.to_uppercase(), message).color(color);
    println!("{msg}");
}

fn get_tasks(query: &str) -> Result<Vec<Task>> {
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

fn get_boards() -> Result<Vec<Board>> {
    let conn = get_connection();
    let query = format!("SELECT * FROM {TABLE_BOARDS}");
    let mut records: Vec<Board> = Vec::new();

    let mut stmt = conn.prepare(&query)?;

    let result_iter = stmt.query_map([], |row| {
        Ok(Board {
            id: row.get(0)?,
            title: row.get(1)?,
        })
    })?;

    for i in result_iter {
        records.push(i?);
    }
    Ok(records)
}

fn create_comment(task_id: u16) -> Result<()> {
    let conn = get_connection();

    let title = user_input::get_user_input("Comment title", SAMPLE_TITLE, true).unwrap();
    let created_at = Utc::now().to_string();

    conn.execute(
        &format!("INSERT INTO {TABLE_COMMENTS} (title, task_id, created_at) VALUES (?1, ?2, ?3)"),
        params![title, task_id, created_at],
    )?;
    Ok(())
}

pub fn create_task() -> Result<()> {
    let conn = get_connection();
    let boards_qtd = get_records_qtd(TABLE_BOARDS).unwrap();

    if boards_qtd == 0 {
        display_message("info", "Create a initial Board", "blue");
        create_board().unwrap();
    }

    let title = user_input::get_user_input("Task title", SAMPLE_TITLE, true).unwrap();
    let board_id = select_board().unwrap().1;
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
    let title = user_input::get_user_input("Board title", SAMPLE_TITLE, true).unwrap();
    let board = Board { id: 0, title };

    conn.execute(
        &format!("INSERT INTO {TABLE_BOARDS} (title) VALUES (?1)"),
        params![board.title],
    )
    .unwrap();
    Ok(())
}

fn get_records_qtd(table: &str) -> Result<u16> {
    let conn = get_connection();
    let query = format!("SELECT COUNT(*) FROM {table}");
    let mut stmt = conn.prepare(&query)?;
    let query_result = stmt.query_map([], |row| Ok(Record { qtd: row.get(0)? }))?;
    let qtd_records = query_result.last().unwrap()?.qtd;
    Ok(qtd_records)
}
