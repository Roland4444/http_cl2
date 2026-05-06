use reqwest::Client;
use serde_json::{json, Value};
use std::{collections::HashMap, io::Write, option};
use anyhow::{Context, Result};
use reqwest::header::{ACCEPT, HeaderValue};
use crate::File;


// Структуры для сериализации тела запроса (можно использовать и json! макрос, но с типами надёжнее)
#[derive(serde::Serialize)]
struct CreateOrderRequest {
    name: String,
    project: Project,
    state: String,
    finish_date: String, // поле finishDate преобразуется в finish_date через rename
    source_account: SourceAccount,
    consignee: Consignee,
    region: Region,
    responsible: Responsible,
    delay: u32,
    external_id: u64,
    order_items: Vec<OrderItem>,
}

#[derive(serde::Serialize)]
struct Project {    id: u32,}

#[derive(serde::Serialize)]
struct SourceAccount {    id: u32,}

#[derive(serde::Serialize)]
struct Consignee {    id: u32,}

#[derive(serde::Serialize)]
struct Region {    id: u32,}

#[derive(serde::Serialize)]
struct Responsible {    id: u32,}

#[derive(serde::Serialize)]
struct OrderItem {
    good_name: String,
    count: u32,
    unit: Unit,
    budget_item: BudgetItem,
    cost_item: CostItem,
    analog_allow: bool,
    inner_comment: String,
    good_position: GoodPosition,
}

#[derive(serde::Serialize)]
struct Unit {
    id: u32,
}

#[derive(serde::Serialize)]
struct BudgetItem {
    id: u32,
}

#[derive(serde::Serialize)]
struct CostItem {
    id: u32,
}

#[derive(serde::Serialize)]
struct GoodPosition {
    external_id: String,
}


fn unit_to_code(unit: &str) -> Option<u32> {
    let normalized = unit.trim_matches('.');
    match normalized {
        "шт" | "штук" | "штуки" => Some(1),
        "м" | "метр" | "метра" => Some(2),
        "м.п" | "п.м" | "пог.м" | "мп" => Some(3),
        "л" | "литр" | "литра" => Some(4),
        "кг" | "килограмм" | "килограмма" => Some(5),
        "м2" | "кв.м" | "квадратный метр" => Some(6),
        "м3" | "куб.м" | "кубический метр" => Some(7),
        _ => None,
    }
}


const ITEMS: &str = r#"Для производства работ по объекту Рыбацкая прошу согласовать :\n1) Клей для газоблока - 240 кулей\n\nКонт. тел. 89170911410 Дмитрий"#;




fn parse_item_line(line: &str) -> Option<(String, u32, u32)> {
    let paren_pos = line.find(')')?;
    let name_start = paren_pos + 1;
    let dash_pos = line[name_start..].find(" - ")?;
    let dash_abs = name_start + dash_pos;
    let name = line[name_start..dash_abs].trim().to_string();
    let rest = &line[dash_abs + 3..];
    let space_pos = rest.rfind(' ')?;
    let quantity_str = rest[..space_pos].trim().replace(',', ".");
    let quantity = quantity_str.parse().ok()?;
    let unit = rest[space_pos + 1..].trim();
    let code = unit_to_code(unit)?;
    Some((name, quantity, code))
}




pub fn get_id_user_via_fio_cynteka(fio: Vec<String>, json__: &Value) -> Option<u64> {
    let employees = json__.get("employees")?.as_array()?;
    let last_name = fio.get(0)?;
    let first_name = fio.get(1)?;

    for emp in employees {
        let lname = emp.get("lastName")?.as_str()?;
        let fname = emp.get("firstName")?.as_str()?;
        if lname == last_name && fname == first_name {            return emp.get("id")?.as_u64();        }
    }
    None
}


pub async fn create_order_with_params(token: &str, client: &Client, responsible: u32, consignee: u32, finish_date: String, project_id: u32) -> Result<Value> {
    let url = "https://restetris.cynteka.ru/api/v1/orders?format=json&isoDate=true";
    let request_body = CreateOrderRequest {
        name: "Тестовый заказ".to_string(),
        project: Project { id: project_id },
        state: "DRAFT".to_string(),
        finish_date: finish_date,
        source_account: SourceAccount { id: 34 },
        consignee: Consignee { id: consignee },
        region: Region { id: 23 },
        responsible: Responsible { id: responsible },
        delay: 30,
        external_id: 1744320000,
        order_items: vec![
            OrderItem {
                good_name: "Тестовый товар".to_string(),
                count: 1,
                unit: Unit { id: 76 },
                budget_item: BudgetItem { id: 10 },
                cost_item: CostItem { id: 93 },
                analog_allow: false,
                inner_comment: "Тест".to_string(),
                good_position: GoodPosition {
                    external_id: "000000004100008693".to_string(),
                },
            }
        ],
    };

    let response = client
        .post(url)
        .header("accept", "application/json")
        .header("ZakupayToken", token)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .context("Ошибка отправки запроса")?;

    let status = response.status();
    let body = response.text().await.context("Ошибка чтения тела ответа")?;

    if status.is_success() {let json: Value = serde_json::from_str(&body).with_context(|| format!("Ошибка парсинга JSON: {}", body))?;Ok(json)} 
    else {        anyhow::bail!("HTTP request failed with status {}: {}",status.as_u16(),body)    
}
}

pub async fn create_order(token: &str, client: &Client) -> Result<Value> {
    let url = "https://restetris.cynteka.ru/api/v1/orders?format=json&isoDate=true";

    let request_body = CreateOrderRequest {
        name: "Тестовый заказ".to_string(),
        project: Project { id: 12 },
        state: "DRAFT".to_string(),
        finish_date: "2026-04-10".to_string(),
        source_account: SourceAccount { id: 34 },
        consignee: Consignee { id: 2 },
        region: Region { id: 23 },
        responsible: Responsible { id: 45 },
        delay: 30,
        external_id: 1744320000,
        order_items: vec![
            OrderItem {
                good_name: "Тестовый товар".to_string(),
                count: 1,
                unit: Unit { id: 76 },
                budget_item: BudgetItem { id: 10 },
                cost_item: CostItem { id: 93 },
                analog_allow: false,
                inner_comment: "Тест".to_string(),
                good_position: GoodPosition {
                    external_id: "000000004100008693".to_string(),
                },
            }
        ],
    };

    let response = client.post(url).header("accept", "application/json").header("ZakupayToken", token).header("Content-Type", "application/json")
        .json(&request_body).send().await.context("Ошибка отправки запроса")?;

    let status = response.status();
    let body = response.text().await.context("Ошибка чтения тела ответа")?;

    if status.is_success() {
        let json: Value = serde_json::from_str(&body).with_context(|| format!("Ошибка парсинга JSON: {}", body))?;
        Ok(json)} 
    else {anyhow::bail!("HTTP request failed with status {}: {}",status.as_u16(),body)}
}



pub async fn fetch_and_save_deliveries(token: &str, client: &Client, output_path: &str) -> Result<()> {
    let url = "https://restetris.cynteka.ru/api/v1/deliveries?format=json";

    let response = client
        .get(url)
        .header(ACCEPT, "application/json")
        .header("ZakupayToken", HeaderValue::from_str(&token)?)
        .send()
        .await
        .context("Не удалось выполнить запрос")?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP ошибка: {} — {}", response.status(), response.text().await?);
    }

    let json_text = response.text().await?;
    let json_value: Value = serde_json::from_str(&json_text)
        .with_context(|| "Ответ не является валидным JSON")?;

    let pretty_json = serde_json::to_string_pretty(&json_value)?;

    let mut file = File::create(output_path)
        .with_context(|| format!("Не удалось создать файл: {}", output_path))?;
    file.write_all(pretty_json.as_bytes())
        .context("Ошибка записи в файл")?;

    println!("✅ JSON успешно сохранён в {}", output_path);
    Ok(())
}


pub async fn create_order_from_cl(token: &str, title: String) -> Result<serde_json::Value> {
    let url = "https://restetris.cynteka.ru/api/v1/orders?format=json&isoDate=true";
    let client = Client::new();

    let payload = json!({
        "name": title,
        "project": { "id": 12 },
        "state": "DRAFT",
        "finishDate": "2026-06-10",
        "sourceAccount": { "id": 34 },
        "consignee": { "id": 4 },
        "region": { "id": 30 },
        "responsible": { "id": 353 },
        "delay": 30,
        "externalId": 1744320000,
        "orderItems": [
            {
                "goodName": "Тестовый товар",
                "count": 1,
                "unit": { "id": 1 },
                "analogAllow": false,
                "innerComment": "Тест",
                "goodPosition": { "externalId": "000000004100008693" }
            },
            {
                "goodName": "Crude Oil2",
                "count": 1,
                "unit": { "id": 1 },
                "analogAllow": false,
                "innerComment": "",
                "goodPosition": { "externalId": "000000004100008693" }
            }
        ]
    });

    let response = client
        .post(url)
        .header("accept", "application/json")
        .header("ZakupayToken", token)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("Ошибка отправки запроса")?;

    let status = response.status();
    let body = response.text().await.context("Ошибка чтения тела ответа")?;

    if status.is_success() {
        let json: serde_json::Value = serde_json::from_str(&body)
            .with_context(|| format!("Ошибка парсинга JSON: {}", body))?;
        Ok(json)
    } else {
        anyhow::bail!("HTTP request failed with status {}: {}", status, body)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::EMPLOYES_FILE_NAME_JS;
    use std::fs;

#[test]
fn test_get_id_skutin() {
    let etalon_id = 222;
    let js_content = fs::read_to_string(EMPLOYES_FILE_NAME_JS).expect("Не удалось прочитать файл");
    let json_value: Value = serde_json::from_str(&js_content).expect("Ошибка парсинга JSON");
    let fio = "Скутин Дмитрий".to_string();
    // Преобразуем &str в String
    let parts: Vec<String> = fio.split_whitespace().map(|s| s.to_string()).collect();
    let result = get_id_user_via_fio_cynteka(parts, &json_value).unwrap();
    assert_eq!(etalon_id, result);
}



use crate::synteka;
#[tokio::test]
async fn test_create_order() -> Result<()>{    
    let token = &synteka();
    let result = create_order_from_cl(token, "RUST CREATE".to_string()).await?;
    println!("Ответ сервера: {:#?}", result);
    Ok(())
}


#[test]
fn test_parse_item_line() {
    let line = "1) Доска 25х100 - 20 шт.";
    let (name, qty, code) = parse_item_line(line).unwrap();
    assert_eq!(name, "Доска 25х100");
    assert_eq!(qty, 20);
    assert_eq!(code, 1);
}


}
//             https://app.swaggerhub.com/apis-docs/Cynteka/cynteka/Cynteka#/