use anyhow::Ok;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct ApiResponce {
    result: ResultData,
}

#[derive(Debug, Deserialize)]
struct ResultData {
    items: Vec<RecentItem>,
}

#[derive(Debug, Deserialize)]
struct RecentItem {
    id: serde_json::Value,
    title: String,
    #[serde(rename = "type")]
    item_type: String,
    last_id: Option<u64>,
    message: Option<Message>,
}

#[derive(Debug, Deserialize)]
struct Message {
    text: Option<String>,
}

pub fn print_chats(json_str: String) -> u32 {
    let response: ApiResponce = match serde_json::from_str(&json_str) {
        Result::Ok(data) => data,
        Result::Err(e) => {
            eprintln!("Error parcing JS:: {}", e);
            return 1;
        }
    };

    for chat in response
        .result
        .items
        .into_iter()
        .filter(|item| item.item_type == "chat")
    {
        let last_id = chat.last_id.unwrap_or(0);
        let message_text = chat
            .message
            .and_then(|m| m.text)
            .unwrap_or_else(|| "NO TEXT".to_string());

        println!("NAME CHAT::  {}", chat.title);
        println!(
            "ID CHAT::  {}",
            chat.id.as_str().unwrap_or(&chat.id.to_string())
        );
        println!("LAST ID::  {}", last_id);
        println!("LAST MESSAGE::  {}", message_text);
        println!("===============================");
    }
    0
}

pub fn main() -> u32 {
    let json_str = r#"
    {
      "items": [
        {
          "id": "chat9562",
          "title": "Евгений Т. - Металл Трейд",
          "type": "chat",
          "last_id": 104712,
          "message": {
            "text": "[Файл]"
          }
        },
        {
          "id": "chat7208",
          "title": "Ползунова",
          "type": "chat",
          "last_id": 108084,
          "message": {
            "text": "Согласовано, работы по монолиту/опалубка/."
          }
        },
        {
          "id": "chat7018",
          "title": "РЭД Грузинская",
          "type": "chat",
          "last_id": 107232,
          "message": {
            "text": "Согласовано"
          }
        },
        {
          "id": "125",
          "title": "Виктория Лысякова",
          "type": "user",
          "last_id": 109170,
          "message": {
            "text": "ок"
          }
        }
      ]
    }
    "#;
    print_chats(json_str.to_string())
}
