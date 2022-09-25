use crate::{
    display_message, Color, ALTERNATIVE_DATETIME_FORMAT, DATETIME_FORMAT, DATE_FORMAT, TIME_FORMAT,
};
use chrono::{TimeZone, Utc};
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
        display_message("error", "Spaces are not allowed", crate::Color::Red);
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

//Get date response
pub fn get_user_date(midnight: bool, must_be_future: bool) -> Option<String> {
    let now = Utc::now();
    let today_str = now.format(DATE_FORMAT).to_string();
    let midnight_time = "00:00:00";

    let current_time_str = match midnight {
        true => midnight_time.to_string(),
        false => now.format(TIME_FORMAT).to_string(),
    };

    let datetime_str = format!(
        "{} {}",
        get_user_input(
            format!("Date ({})", DATE_FORMAT).as_str(),
            &today_str,
            false
        )
        .unwrap(),
        current_time_str
    );
    let datetime_date = Utc.datetime_from_str(&datetime_str, &ALTERNATIVE_DATETIME_FORMAT);

    if datetime_date.is_err() {
        display_message("error", "Invalid date", Color::Red);
        return None;
    }

    if must_be_future {
        if now > datetime_date.unwrap() {
            display_message("error", "Datetime cannot be past", Color::Red);
            return None;
        }
    }
    let datetime_str = datetime_date.unwrap().format(DATETIME_FORMAT).to_string();
    Some(datetime_str)
}
