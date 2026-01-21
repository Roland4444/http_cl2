use encoding_rs::WINDOWS_1251;
use reqwest::Client;
use std::fs;
use serde_json::Value;
use std::error::Error;

const WEBHOOK_FILENAME: &str = "webhook";

pub fn read_bytes(filename: &str)-> Vec<u8> {
    fs::read(filename).expect("Cant read files")
}

pub fn read_lines(filename: &str) -> Vec<String>{
    let bytes = read_bytes(filename);
    let (decoded, _, had_errors) = WINDOWS_1251.decode(&bytes);
    if had_errors{
        println!("Some characters not decoded")
    }
    decoded.to_string().lines().map(|line| line.to_string()).collect()    
}


pub async fn get_user_by_id(client_reqwest: Client, user_id: i32) -> Result<Value, Box<dyn Error>> {
    let webhook = get_webhook();
    
    
    let params = [
        ("ID", user_id.to_string()),
    ];
    
    let url = format!("{}/user.get", webhook.trim_end_matches('/'));
    
    let response = client_reqwest
        .post(&url)
        .form(&params)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    
    let json: Value = response.json().await?;
    
    if let Some(error) = json.get("error") {
        return Err(format!("Bitrix24 error: {}", error).into());
    }
    
    Ok(json)
}



pub fn get_webhook() -> String{
    get_webhook_(WEBHOOK_FILENAME)
}

pub fn get_webhook_(filename: &str) -> String{
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


pub async fn get_multiple_users(client_reqwest: Client, user_ids: &[i32]) -> Result<Value, Box<dyn Error>> {
    let webhook = get_webhook();
    
    let mut params = vec![];
    
    // Формируем массив ID
    for (i, &id) in user_ids.iter().enumerate() {
        let param: String = format!("ID[{}]", i);
        params.push((param, id.to_string()));
    }
    
    let url = format!("{}/user.get", webhook.trim_end_matches('/'));
    
    let response = client_reqwest
        .post(&url)
        .form(&params)
        .send()
        .await?;
    
    let json: Value = response.json().await?;
    
    if let Some(error) = json.get("error") {
        return Err(format!("Bitrix24 error: {}", error).into());
    }
    
    Ok(json)
}


pub async fn get_user_with_fields(client_reqwest: Client, user_id: i32, fields: &[&str]) -> Result<Value, Box<dyn Error>> {
    let webhook = get_webhook();
    
    // Создаем параметры в одну строку
    let params: Vec<(String, String)> = std::iter::once(("ID".to_string(), user_id.to_string()))
        .chain(fields.iter().enumerate().map(|(i, field)| {
            (format!("SELECT[{}]", i), field.to_string())
        }))
        .collect();
    
    let url = format!("{}/user.get", webhook.trim_end_matches('/'));
    
    let response = client_reqwest
        .post(&url)
        .form(&params)
        .send()
        .await?;
    
    let json: Value = response.json().await?;
    
    if let Some(error) = json.get("error") {
        return Err(format!("Bitrix24 error: {}", error).into());
    }
    
    Ok(json)
}