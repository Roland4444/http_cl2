use reqwest::Client;
use serde_json::{json, Value};
use std::{collections::HashMap, option};
use anyhow::{Context, Result};

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

// Пример использования:
