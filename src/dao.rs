use crate::{
    datetime_str_is_past, display_message, get_connection, get_tasks, select_board,
    user_input::{get_user_confirmation, get_user_date, get_user_input, get_user_selection},
    Board, Color, Comment, Record, Task, DATETIME_FORMAT, SAMPLE_TITLE, TABLE_BOARDS,
    TABLE_COMMENTS, TABLE_TASKS,
};
use chrono::Utc;
use rusqlite::{params, Result};
use tabled::{Disable, Style, Table};

pub fn get_records_qtd(table: &str) -> Result<u16> {
    let conn = get_connection();
    let query = format!("SELECT COUNT(*) FROM {table}");
    let mut stmt = conn.prepare(&query)?;
    let query_result = stmt.query_map([], |row| Ok(Record { qtd: row.get(0)? }))?;
    let qtd_records = query_result.last().unwrap()?.qtd;
    Ok(qtd_records)
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

pub fn edit_board(title: &str, id: u16) -> Result<()> {
    let conn = get_connection();
    let title = get_user_input("New Board title", title, true).unwrap();

    conn.execute(
        &format!("UPDATE {TABLE_BOARDS} SET title = ?1 WHERE id = ?2"),
        params![title, id],
    )
    .unwrap();
    Ok(())
}

pub fn get_comments_by_task_id(task_id: u16) -> Result<Vec<Comment>> {
    let conn = get_connection();
    let query = format!("SELECT * FROM {TABLE_COMMENTS} WHERE task_id = {task_id}");
    let mut comments: Vec<Comment> = Vec::new();

    let mut stmt = conn.prepare(&query)?;

    let result_iter = stmt.query_map([], |row| {
        Ok(Comment {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(3)?,
        })
    })?;

    for i in result_iter {
        comments.push(i?);
    }
    Ok(comments)
}

///title, id
pub fn select_task(done: u8) -> Option<(String, u16)> {
    let records_qtd = get_records_qtd(TABLE_TASKS).unwrap();
    if records_qtd == 0 {
        display_message("info", "No Tasks found in database", Color::Cyan);
        return None;
    }
    let mut query = format!("SELECT * FROM {TABLE_TASKS} WHERE done = {done}");

    let all_boards = get_user_confirmation("From all boards");

    if !all_boards {
        let (_, board_id) = select_board().unwrap();
        query.push_str(&format!(" AND board_id = {board_id}"));
    }

    let tasks_raw = get_tasks(&query).unwrap();
    if tasks_raw.len() == 0 {
        display_message(
            "info",
            "No Tasks found in database with these criteria",
            Color::Cyan,
        );
        return None;
    }

    let mut tasks: Vec<String> = tasks_raw
        .iter()
        .map(|x| format!("{} - {}", &x.id, &x.title))
        .collect();

    tasks.sort();

    let (selected_task, _) = get_user_selection(&tasks, "Task");

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

pub fn list_delayed_tasks() {
    let query = format!("SELECT * FROM {TABLE_TASKS} WHERE done = 0");
    let tasks_raw = get_tasks(&query).unwrap();

    if tasks_raw.len() == 0 {
        return;
    }

    let delayed_tasks: Vec<&Task> = tasks_raw
        .iter()
        .filter(|x| datetime_str_is_past(&x.reminder))
        .collect();

    if delayed_tasks.len() == 0 {
        return;
    }
    println!("Delayed Tasks: {}", &delayed_tasks.len());

    let table = Table::new(delayed_tasks)
        .with(Style::modern())
        .with(Disable::Column(2..4));

    println!("{}", table);
}

//Delete database records
pub fn delete_record_by_id(table: &str, id: u16) -> Result<()> {
    let conn = get_connection();
    conn.execute(&format!("DELETE FROM {table} WHERE id = ?1"), params![id])?;
    Ok(())
}

pub fn get_boards() -> Result<Vec<Board>> {
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

pub fn create_comment(task_id: u16) -> Result<()> {
    let conn = get_connection();

    let title = get_user_input("Comment title", SAMPLE_TITLE, true).unwrap();
    let created_at = Utc::now().to_rfc2822();

    conn.execute(
        &format!("INSERT INTO {TABLE_COMMENTS} (title, task_id, created_at) VALUES (?1, ?2, ?3)"),
        params![title, task_id, created_at],
    )?;
    Ok(())
}

pub fn set_reminder(task_id: u16) -> Result<()> {
    let conn = get_connection();

    let reminder = get_user_date(true, false);
    if reminder.is_none() {
        return Ok(());
    }

    conn.execute(
        &format!("UPDATE {TABLE_TASKS} SET reminder = ?1 WHERE id = ?2"),
        params![reminder, task_id],
    )?;
    Ok(())
}

pub fn create_task() -> Result<()> {
    let conn = get_connection();
    let boards_qtd = get_records_qtd(TABLE_BOARDS).unwrap();

    if boards_qtd == 0 {
        display_message("info", "Create a initial Board", Color::Cyan);
        create_board().unwrap();
    }

    let title = get_user_input("Task title", SAMPLE_TITLE, true).unwrap();
    let board_id = select_board().unwrap().1;
    let created_at = Utc::now().format(DATETIME_FORMAT).to_string();
    let with_reminder = get_user_confirmation("Set reminder");

    let reminder = match with_reminder {
        true => get_user_date(true, true).unwrap(),
        false => "".to_string(),
    };

    conn.execute(
        &format!("INSERT INTO {TABLE_TASKS} (title, done, board_id, created_at, reminder) VALUES (?1, ?2, ?3, ?4, ?5)"),
        params![title, 0, board_id, created_at, reminder],
    )
    .unwrap();
    Ok(())
}

pub fn create_board() -> Result<()> {
    let conn = get_connection();
    let title = get_user_input("Board title", SAMPLE_TITLE, true).unwrap();
    let board = Board { id: 0, title };

    conn.execute(
        &format!("INSERT INTO {TABLE_BOARDS} (title) VALUES (?1)"),
        params![board.title],
    )
    .unwrap();
    Ok(())
}

pub fn switch_task_status(task_id: u16) -> Result<()> {
    let conn = get_connection();

    let done: u8 = match get_user_confirmation("Done") {
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
