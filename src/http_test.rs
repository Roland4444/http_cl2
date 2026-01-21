use encoding_rs::WINDOWS_1251;
use reqwest::Client;
use std::fs;

const WEBHOOK_FILENAME: &str = "webhook";

fn read_bytes(filename: &str)-> Vec<u8> {
    fs::read(filename).expect("Cant read files")
}
fn read_lines(filename: &str) -> Vec<String>{
    let bytes = read_bytes(filename);
    let (decoded, _, had_errors) = WINDOWS_1251.decode(&bytes);
    if had_errors{
        println!("Some characters not decoded")
    }
    decoded.to_string().lines().map(|line| line.to_string()).collect()    
}


fn get_webhook() -> String{
    get_webhook_(WEBHOOK_FILENAME)
}

fn get_webhook_(filename: &str) -> String{
    let vec = read_lines(filename);
    let elem = vec[0].clone();
    elem
}
pub async fn update_param_(client_reqwest: Client, params: &[(&str, &str)])-> Result<String, Box<dyn std::error::Error>>{
    // let client = reqwest:: Client::new();
    let url = get_webhook()+"user.update";
    println!("Resulted url{}", url);
    let responce = client_reqwest
        .post(url)
        .form(params)
        .send()
        .await?;
    println!("Status:{}", responce.status());
    println!("Headers: {:#?}", responce.headers());

    let body = responce.text().await?;
    Ok(body)
}
