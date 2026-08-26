
use crate::EFES;

pub fn get_work_pos(input: String) -> String {
    if let Some(pos) = input.find('=') {
        input[..pos].to_string()
    } else {
        EFES.to_string()
    }
}