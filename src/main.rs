use rusqlite::Result;
use task_manager::*;
pub mod dao;
pub mod user_input;

fn main() -> Result<()> {
    display_app_intro();
    dao::create_database()?;
    dao::list_delayed_tasks();

    loop {
        let action = user_input::get_user_selection_text(&MAIN_MENU_OPTIONS.to_vec(), "Option");

        match action.as_str() {
            CREATE_TASK => dao::create_task()?,
            VIEW_PENDING_TASKS => list_tasks(0)?,
            VIEW_DONE_TASKS => list_tasks(1)?,
            CREATE_BOARD => dao::create_board()?,
            VIEW_BOARDS => list_boards()?,
            _ => break,
        };
    }

    Ok(())
}
