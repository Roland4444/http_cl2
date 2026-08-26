
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


#[cfg(test)]
mod tests {

use std::assert_eq;

use super::*;

    #[test]
    fn test_ocr() {
        let input = "Руководитель отдела  по внутренней отделке=Шагинян А.	";
        assert_eq!("Руководитель отдела  по внутренней отделке", get_work_pos(input.to_string()));
        assert_eq!("Шагинян", get_f_(input.to_string()));
      
    }

    
}
