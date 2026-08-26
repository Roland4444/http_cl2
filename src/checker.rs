
use crate::EFES;

pub fn get_work_pos(input: String) -> String {
    if let Some(pos) = input.find('=') {
        input[..pos].to_string()
    } else {
        EFES.to_string()
    }
}


pub fn get_f_(input: String) -> String {
    if let Some(pos) = input.find('=') {
        let after = &input[pos + 1..];
        return after.split_whitespace().next().unwrap_or("").to_string();
    } else
    {
        "" .to_string()
    }
}