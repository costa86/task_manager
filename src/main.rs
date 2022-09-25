use rusqlite::Result;
use sample::*;
pub mod dao;
pub mod user_input;

fn main() -> Result<()> {
    dao::create_database()?;
    dao::list_delayed_tasks();

    loop {
        let (_, index) = user_input::get_user_selection(&MAIN_MENU_OPTIONS.to_vec(), "Option");
        match index {
            0 => dao::create_task()?,
            1 => list_tasks(0)?,
            2 => list_tasks(1)?,
            3 => dao::create_board()?,
            4 => list_boards()?,
            5 => other()?,
            _ => break,
        };
    }

    Ok(())
}
