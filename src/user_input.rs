use crate::display_message;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::fmt::Display;

///Get boolean response
pub fn get_user_confirmation(question: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .default(true)
        .interact()
        .unwrap()
}

///Get text response.
pub fn get_user_input(text: &str, default_text: &str, allow_spaces: bool) -> Option<String> {
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
