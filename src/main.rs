use std::fs::read_to_string;
use encoding_rs::WINDOWS_1251;
use std::fs;

fn read_bytes(filename: &str)-> Vec<u8> {
    fs::read(filename).expect("Cant read files")
}


const TARGET_ADDRESS: &str = "http://localhost:11111/custom";
const WEBHOOK_FILENAME: &str = "webhook";
const WEBHOOK_FILENAME_TEST: &str = "webhook_test";

fn read_lines(filename: &str) -> Vec<String>{
    let bytes = read_bytes(filename);
    let (decoded, _, had_errors) = WINDOWS_1251.decode(&bytes);

    if had_errors{
        println!("Some characters not decoded")
    }

    decoded.to_string().lines().map(|line| line.to_string()).collect()
    
}

fn get_first_line(filename: &str) -> String{
    let mut result = Vec::new();
    for line in read_to_string(filename).unwrap().lines(){
        result.push(line.to_string());
    }

    result[0].clone()
}

pub fn update_second_name(second_name: &str, ID: i32) -> String{
    "SUCCESS".to_string()
}


pub fn update_param_(param_name: &str, value: &str, ID: i32) -> String{


    "SUCCESS".to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let param_name = "SECOND_NAME";
    let body = reqwest::get(TARGET_ADDRESS)
        .await?
        .text()
        .await?;
    println!("body = {body:?}");
    Ok(())
}


pub fn add(a: i32, b:i32) -> i32 {
    a + b
}

pub fn bad_add(a: i32, b: i32) -> i32{
    a - b
}


fn get_webhook() -> String{
    get_webhook_(WEBHOOK_FILENAME)
}

fn get_webhook_(filename: &str) -> String{
    let vec = read_lines(filename);
    let elem = vec[0].clone();
    elem
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_add(){
        assert_eq!(add(1, 2), 3);
    }

    #[test]
    fn test_bad_add(){
        assert_eq!(bad_add(1, 2), -1);
    }

    #[test]
    fn test_read_str(){
        let filename = "lstdata.csv";
        let etalon = ";ФИО;Должность;;;;телефон;юр. лицо";
        let vect = read_lines(filename);
        let line0 = vect[0].clone();
        assert_eq!(etalon, line0);
    }

    #[test]
    fn test_read_webbhook(){
        let test_webhook = get_webhook_(WEBHOOK_FILENAME_TEST);
        let etalon = "http://google.com";
        assert_eq!(etalon, test_webhook);

    }
}





