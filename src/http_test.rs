use encoding_rs::WINDOWS_1251;
use reqwest::Client;
use serde_json::Value;
use serde_json::json;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;

const WEBHOOK_FILENAME: &str = "webhook";

pub fn read_bytes(filename: &str) -> Vec<u8> {
    fs::read(filename).expect("Cant read files")
}

pub fn read_lines_utf8(filename: &str) -> Vec<String> {
    match fs::read_to_string(filename) {
        Ok(content) => content.lines().map(|s| s.to_string()).collect(),
        Err(e) => {
            eprintln!("Ошибка чтения файла {}: {}", filename, e);
            Vec::new() // Возвращаем пустой вектор при ошибке
        }
    }
}

pub async fn read_tasks2(client_reqwest: &Client) -> Result<Value, Box<dyn Error>> {
    let webhook = get_webhook();
    let url = format!("{}/tasks.task.list", webhook.trim_end_matches('/'));

    // Создаем тело запроса как в curl примере
    let mut body = json!({
            "order": {
                "DEADLINE": "asc",
                "PRIORITY": "desc"
            },

    //  "select": [
    //             "ID", "TITLE", "DESCRIPTION", "STATUS", "subStatus",
    //             "DEADLINE", "CREATED_DATE", "RESPONSIBLE_ID",
    //             "ACCOMPLICES", "AUDITORS", "TAGS", "COUNTERS",
    //             "PRIORITY", "MARK", "COMMENTS"
    //         ],


            "select": [
                "ID", "TITLE",  "COMMENTS"
            ],
            "params": {
                "WITH_TIMER_INFO": true,
                "WITH_RESULT_INFO": true,
                "WITH_PARSED_DESCRIPTION": true
            }, "start":150
        });

    let response = client_reqwest
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body) // Автоматически сериализует в JSON и устанавливает Content-Type
        .send()
        .await?;

    // Проверяем статус ответа
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let json: Value = response.json().await?;

    // Проверяем наличие ошибки в ответе Bitrix24
    if let Some(error) = json.get("error") {
        return Err(format!("Bitrix24 error: {}", error).into());
    }

    // Также проверяем наличие поля "error" в result, если Bitrix24 так возвращает ошибки
    if let Some(result) = json.get("result") {
        if let Some(error) = result.get("error") {
            return Err(format!("Bitrix24 result error: {}", error).into());
        }
    }

    Ok(json)
}

pub async fn read_tasks(client_reqwest: Client) -> Result<Value, Box<dyn Error>> {
    let webhook = get_webhook();
    let url = format!("{}/tasks.task.list", webhook.trim_end_matches('/'));
    let response = client_reqwest.post(&url).send().await?;
    let json: Value = response.json().await?;
    if let Some(error) = json.get("error") {
        return Err(format!("Bitrix24 error: {}", error).into());
    }
    Ok(json)
}

pub fn read_lines(filename: &str) -> Vec<String> {
    let bytes = read_bytes(filename);
    let (decoded, _, had_errors) = WINDOWS_1251.decode(&bytes);
    if had_errors {
        println!("Some characters not decoded")
    }
    decoded
        .to_string()
        .lines()
        .map(|line| line.to_string())
        .collect()
}

pub async fn get_user_by_id(client_reqwest: Client, user_id: i32) -> Result<Value, Box<dyn Error>> {
    let webhook = get_webhook();
    let params = [("ID", user_id.to_string())];
    let url = format!("{}/user.get", webhook.trim_end_matches('/'));
    let response = client_reqwest.post(&url).form(&params).send().await?;
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    let json: Value = response.json().await?;
    if let Some(error) = json.get("error") {
        return Err(format!("Bitrix24 error: {}", error).into());
    }
    Ok(json)
}

pub fn get_webhook() -> String {
    get_webhook_(WEBHOOK_FILENAME)
}

pub fn get_webhook_(filename: &str) -> String {
    let vec = read_lines(filename);
    let elem = vec[0].clone();
    elem
}

pub async fn update_param_(
    client_reqwest: Client,
    params: &[(&str, &str)],
) -> Result<String, Box<dyn std::error::Error>> {
    let url = get_webhook() + "user.update";
    println!("Resulted url{}", url);
    let responce = client_reqwest.post(url).form(params).send().await?;
    println!("Status:{}", responce.status());
    println!("Headers: {:#?}", responce.headers());
    let body = responce.text().await?;
    Ok(body)
}

//  curl -X POST   "https://relits.bitrix24.ru/rest/336/9xuqnmbu879m3zg3/im.message.add"   -H "Content-Type: application/json"   -d '{
//     "DIALOG_ID": "296",
//     "MESSAGE": "ghghghghghghghghgghgggghgg"
//   }'

pub async fn send_notification_to_user(
    client_reqwest: Client,
    id: &str,
    message: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let base_url = get_webhook();
    let url = format!("{}im.message.add", base_url);
    println!("Resulted url: {}", url);
    let request_body = json!({
        "DIALOG_ID": id,
        "MESSAGE": message
    });

    println!("IN SEND NOTIFICATION {}", request_body.to_string());
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
}

pub async fn get_multiple_users(
    client_reqwest: Client,
    user_ids: &[i32],
) -> Result<Value, Box<dyn Error>> {
    let webhook = get_webhook();
    let mut params = vec![];
    for (i, &id) in user_ids.iter().enumerate() {
        let param: String = format!("ID[{}]", i);
        params.push((param, id.to_string()));
    }
    let url = format!("{}/user.get", webhook.trim_end_matches('/'));
    let response = client_reqwest.post(&url).form(&params).send().await?;
    let json: Value = response.json().await?;
    if let Some(error) = json.get("error") {
        return Err(format!("Bitrix24 error: {}", error).into());
    }
    Ok(json)
}

pub async fn get_user_with_fields(
    client_reqwest: Client,
    user_id: i32,
    fields: &[&str],
) -> Result<Value, Box<dyn Error>> {
    let webhook = get_webhook();
    let params: Vec<(String, String)> = std::iter::once(("ID".to_string(), user_id.to_string()))
        .chain(
            fields
                .iter()
                .enumerate()
                .map(|(i, field)| (format!("SELECT[{}]", i), field.to_string())),
        )
        .collect();
    let url = format!("{}/user.get", webhook.trim_end_matches('/'));
    let response = client_reqwest.post(&url).form(&params).send().await?;
    let json: Value = response.json().await?;
    if let Some(error) = json.get("error") {
        return Err(format!("Bitrix24 error: {}", error).into());
    }
    Ok(json)
}