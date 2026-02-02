use encoding_rs::WINDOWS_1251;
use reqwest::Client;
use std::fs;
use serde_json::Value;
use std::error::Error;
use serde_json::json;


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



//  curl -X POST   "https://relits.bitrix24.ru/rest/336/9xuqnmbu879m3zg3/im.message.add"   -H "Content-Type: application/json"   -d '{
//     "DIALOG_ID": "296",
//     "MESSAGE": "ghghghghghghghghgghgggghgg"
//   }'


pub async fn send_notification_to_user(client_reqwest: Client, id: &str, message: &str) -> Result<String, Box<dyn std::error::Error>>{
 let base_url = get_webhook(); // Предполагается, что эта функция возвращает базовый URL
    let url = format!("{}im.message.add", base_url);
    println!("Resulted url: {}", url);

    // Формируем JSON тело запроса
    let request_body = json!({
        "DIALOG_ID": id,
        "MESSAGE": message
    });

    // Отправляем POST запрос с JSON телом
    let response = client_reqwest
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    println!("Status: {}", response.status());
    println!("Headers: {:#?}", response.headers());

    let body = response.text().await?;
    Ok(body)

    
};


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