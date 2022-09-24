use rusqlite::Result;
use sample::*;
pub mod user_input;

fn main() -> Result<()> {
    loop {
        let (_, index) = user_input::get_user_selection(&MAIN_MENU_OPTIONS.to_vec(), "Option");
        create_database()?;

        match index {
            0 => create_board()?,
            1 => create_task()?,
            2 => list_tasks(1)?,
            3 => list_tasks(0)?,
            4 => list_boards()?,
            5 => other()?,
            _ => break,
        };
    }

    Ok(())
}
