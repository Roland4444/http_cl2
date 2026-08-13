


//base64 любого фолрмата отправляется в плтофрму


use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use encoding_rs::WINDOWS_1251;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use serde_json::Value;

const INSTRUCTION_VAR: &str = "Извлеки из этого изображения: весь текст по строкам";
const INSTRUCTION_VAR_2FA: &str = r#"Извлеки из этого изображения: статус напротив "Двухфакторная аутентификация" "#;



#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ExtractedInvoice {
    pub number: String,
    pub date: String,
    pub amount: String,
    pub issuer_inn: String,
    pub recipient_inn: String,
}

#[derive(Debug)]
pub enum VibeError {
    RequestFailed(reqwest::Error),
    ParseError(serde_json::Error),
    EmptyResponse,
    IoError(std::io::Error),
}

// ---- Основные функции ----

pub fn build_vision_payload(image_base64: &str, instruction: &str) -> serde_json::Value {
    json!({
        "messages": [
            {
                "role": "system",
                "content": "Ты — AI-помощник, который извлекает данные для заявок подрядчиков из изображений.  Приведи распарсенные данные в следующий формат {номер})  {наименование} - {количество} {единица измерения}. например \"1) Доска 25х100 - 20 шт\" .  Для знака новой строки используй $$   \n не вставляй  , в конце каждой строки ставь точку .  "                              //  "content": "Ты — AI-помощник, который извлекает данные из счетов-фактур."
            },
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": instruction },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/jpeg;base64,{}", image_base64)
                        }
                    }
                ]
            }
        ]
    })
}


pub fn build_vision_payload_2FA(image_base64: &str, instruction: &str) -> serde_json::Value {
    json!({
        "messages": [
            {
                "role": "system",
                "content": "Ты — AI-помощник, который извлекает данные для заявок подрядчиков из изображений.  Приведи распарсенные данные в следующий формат {номер})  {наименование} - {количество} {единица измерения}. например \"1) Доска 25х100 - 20 шт\" .  Для знака новой строки используй $$   \n не вставляй  , в конце каждой строки ставь точку .  "                              //  "content": "Ты — AI-помощник, который извлекает данные из счетов-фактур."

              //  "content": "Ты — AI-помощник, который извлекает данные для извлечения статуса двухфакторной аутентификации из изображений.  Приведи статус двухфакторной аутентификакции из загржуенного изображения "                              //  "content": "Ты — AI-помощник, который извлекает данные из счетов-фактур."
            },
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": instruction },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/jpg;base64,{}", image_base64)
                        }
                    }
                ]
            }
        ]
    })
}

// Отправка запроса – возвращает сырой JSON
async fn call_vibe_api(
    client: &reqwest::Client,
    api_key: &str,
    payload: serde_json::Value,
) -> Result<serde_json::Value, reqwest::Error> {
    let response = client
        .post("https://vibecode.bitrix24.tech/v1/ai/chat/completions")
        .header("X-Api-Key", api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;
    let json_response: serde_json::Value = response.json().await?;
    Ok(json_response)
}


pub fn read_bytes(filename: &str) -> Vec<u8> {    fs::read(filename).expect("Cant read files")}


pub fn read_lines(filename: &str) -> Vec<String> {
    let bytes = read_bytes(filename);
    let (decoded, _, had_errors) = WINDOWS_1251.decode(&bytes);
    if had_errors {        println!("Some characters not decoded")    }
    decoded.to_string().lines().map(|line| line.to_string()).collect()
}

const KEY_FILE: &str = "KEY"; 

pub fn get_webhook_(filename: &str) -> String {
    let vec = read_lines(filename);
    let elem = vec[0].clone();
    elem
}

fn extract_text_from_responseV(value: &Value) -> Result<String, Box<dyn std::error::Error>> {
    value["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Поле content отсутствует или не является строкой".into())
}



fn extract_text_from_response(json_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    let v: Value = serde_json::from_str(json_str)?;
    
    let content = v["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Не удалось получить content")?
        .to_string();
    
    Ok(content)
}

fn extract_clean_table(json_str: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = extract_text_from_response(json_str)?;
    
    let lines: Vec<String> = content
        .lines()
        .map(|line| {
            line.trim_start_matches(|c: char| c == '*' || c.is_ascii_digit() || c == ':' || c.is_whitespace())
                .to_string()
        })
        .filter(|line| !line.is_empty())
        .collect();
    
    Ok(lines)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let API_KEY = get_webhook_(KEY_FILE);   
    let file_path = "___1.jpg"; 

    let v = vec!["1.jpg", "1.tif", "1.bmp", "1.gif"];

    for i in v {
        let image_data = std::fs::read(i)?;
        let base64_image = general_purpose::STANDARD.encode(&image_data);
        let payload = build_vision_payload(&base64_image, INSTRUCTION_VAR);
        let client = reqwest::Client::new();

        let response = call_vibe_api(&client, &API_KEY, payload).await?;


        println!("✅ Ответ от VibeCode:");
        println!("{}\n",serde_json::to_string_pretty(&response)?);



    }

    let image_data = std::fs::read(file_path)?;
    let base64_image = general_purpose::STANDARD.encode(&image_data);

    let payload = build_vision_payload(&base64_image, INSTRUCTION_VAR);
    let client = reqwest::Client::new();

    let response = call_vibe_api(&client, &API_KEY, payload).await?;

    println!("✅ Ответ от VibeCode:");
   // println!("{}", serde_json::to_string_pretty(&response)?);

    let  js_resp = serde_json::to_string(&response).unwrap();
    let target_text = extract_text_from_response ( js_resp.as_str()).unwrap();

    println!("EXTRACTED::{}", target_text);

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{assert_eq, mem::take};

use super::*;

    #[tokio::test]
    async fn test_ocr()-> Result<(), Box<dyn std::error::Error>> {
        let API_KEY = get_webhook_(KEY_FILE);   
        let file_path = "___1.jpg"; 
        let v = vec![ "00.tif"];
   //     let v = vec!["1.jpg", "1.tif", "1.bmp", "1.gif", "00.tif"];
        for i in v {
            let image_data = std::fs::read(i)?;
            let base64_image = general_purpose::STANDARD.encode(&image_data);
            let payload = build_vision_payload(&base64_image, INSTRUCTION_VAR);
            let client = reqwest::Client::new();

            let response = call_vibe_api(&client, &API_KEY, payload).await?;

            // Извлекаем полный текст
            let content = match extract_text_from_responseV(&response) {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("⚠️ Не удалось извлечь текст из ответа для файла {}: {}", i, e);
                    continue;
                }
            };
            println!("📄 Полный ответ для {}:\n{}", i, content);
            match extract_clean_table(&content) {
            Ok(rows) => {
                println!("✅ Извлечённые строки из файла {}:", i);
                for row in rows {
                    println!("{}", row);
                }
            println!();
            }
            Err(e) => {
                eprintln!("⚠️ Не удалось извлечь таблицу из файла {}: {}", i, e);
            }
            }
        }



        let ETALON_EXTRACT = "1) Кабель силовой ВВГнг(А)-LS 3х6,0 - 1000 м. $$ 2) Дюбель-хомут под плоский кабель 5-10 - 2000 шт. $$ 3) Дюбель-хомут под плоский кабель 6-12 - 2000 шт.";

        let image_data = std::fs::read(file_path)?;
        let base64_image = general_purpose::STANDARD.encode(&image_data);

        let payload = build_vision_payload(&base64_image, INSTRUCTION_VAR);
        let client = reqwest::Client::new();

        let response = call_vibe_api(&client, &API_KEY, payload).await?;


        let  js_resp = serde_json::to_string(&response).unwrap();
        let target_text = extract_text_from_response ( js_resp.as_str()).unwrap();

        let normalized_etalon = ETALON_EXTRACT.replace('х', "x");
        let normalized_target = target_text.replace('х', "x");
        assert_eq!(normalized_etalon, normalized_target);
    //    assert_eq!(ETALON_EXTRACT.to_string(), target_text.to_string());
        println!("EXTRACTED::{}", target_text);


        println!("✅ Ответ от VibeCode:");
        println!("{}", serde_json::to_string_pretty(&response)?);

        Ok(())
    }


    #[tokio::test]
    async fn test_ocr_2FA()-> Result<(), Box<dyn std::error::Error>> {
        let API_KEY = get_webhook_(KEY_FILE);   
        let file_path = "00.tif"; 

    //    let ETALON_EXTRACT = "1) Кабель силовой ВВГнг(А)-LS 3х6,0 - 1000 м. $$ 2) Дюбель-хомут под плоский кабель 5-10 - 2000 шт. $$ 3) Дюбель-хомут под плоский кабель 6-12 - 2000 шт.";

        let image_data = std::fs::read(file_path)?;
        let base64_image = general_purpose::STANDARD.encode(&image_data);

        let payload = build_vision_payload(&base64_image, INSTRUCTION_VAR_2FA);
        let client = reqwest::Client::new();

        let response = call_vibe_api(&client, &API_KEY, payload).await?;


        let  js_resp = serde_json::to_string(&response).unwrap();
        let target_text = extract_text_from_response ( js_resp.as_str()).unwrap();

    //    let normalized_etalon = ETALON_EXTRACT.replace('х', "x");
        let normalized_target = target_text.replace('х', "x");
     //   assert_eq!(normalized_etalon, normalized_target);
    //    assert_eq!(ETALON_EXTRACT.to_string(), target_text.to_string());
        println!("EXTRACTED::{}", target_text);


        println!("✅ Ответ от VibeCode:");
        println!("{}", serde_json::to_string_pretty(&response)?);

        Ok(())
    }



    #[test]
    fn test_extract_text() {
        let json_data = r#"{
            "choices": [{
                "message": {
                    "content": "**Строка 1:** 1 | Кабель | 1000 | м\n**Строка 2:** 2 | Дюбель | 2000 | шт"
                }
            }]
        }"#;
        
        let text = extract_text_from_response(json_data).unwrap();
        println!("Полный текст:\n{}", text);
        
        let table = extract_clean_table(json_data).unwrap();
        for row in table {
            println!("{}", row);
        }
    }
}

        // "text": "BMP",
        //     40672

        // "text": "GIF",
        //     40670


        // "text": "JPG",
        //     40668

        // "text": "TIF",
        //     40666
  