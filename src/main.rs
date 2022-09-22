use rusqlite::Result;
use sample::*;

fn main() -> Result<()> {
    loop {
        let (_, index) = get_user_selection(&CHOICES.to_vec(), "Option");

        match index {
            0 => create_database()?,
            1 => create_board()?,
            2 => create_task()?,
            3 => list_tasks()?,
            4 => list_boards()?,
            5 => other()?,
            _ => break,
        };
    }

    Ok(())
}
