use crate::http_Proc::get_webhook;
use crate::http_Test::{read_lines, read_lines_utf8};
use bincode;
use reqwest;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str, json};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::{Read, Write};
use std::process::id;
use std::ptr::hash;
pub mod http_Parser;
pub mod http_Proc;
pub mod http_Test;
use anyhow::Context;
use once_cell::sync::Lazy;
use std::thread;
use std::time::Duration;
use common::*;

const DEFAULT_DUMP: &str = "all_dump.bin";
const ADD_DUMP: &str = "snoyman.bin";
const SYNTEKA_TOKEN_FILE: &str = "synteka";


macro_rules! hashmap {
    ($($key: expr => $val: expr), *) => {
        {
            let mut map = ::std::collections::HashMap::new();
            $(map.insert($key, $val); )*
            map        }    };}
//genned
static CHATS_ID: Lazy<HashMap<Collab, &str>> = Lazy::new(|| {
    hashmap!(
        Collab::PAYMENTS => "chat9224",        Collab::OLIVIA => "chat6998",        Collab::BABEFA => "chat6974",        Collab::OKLAND => "chat6986",        Collab::RED => "chat7018",
        Collab::TETRIS => "chat7014",          Collab::SCANDINAVIA => "chat9796",   Collab::KUIB => "chat7210",          Collab::POLZ => "chat7208",          Collab::ZVEZD => "chat7242",
        Collab::SKY => "chat6966",             Collab::OWN => "chat13372"    )});
//genned

static CHAT_NUM_ID: Lazy<HashMap<Collab, u64>> = Lazy::new(|| {
    CHATS_ID
        .iter()
        .map(|(collab, &id_str): (&Collab, &&str)| {
            let num = id_str.strip_prefix("chat").unwrap_or(id_str).parse().expect("INVALID ID");
            (*collab, num)})
        .collect()});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum ADDITIONAL_FIELDS {
    WORK_POSITION,
    PERSONAL_BIRTHDAY,
    UF_DEPARTMENT,
}

impl ADDITIONAL_FIELDS {
    fn all_values() -> Vec<Self> {
        vec![
            ADDITIONAL_FIELDS::WORK_POSITION,
            ADDITIONAL_FIELDS::PERSONAL_BIRTHDAY,
            ADDITIONAL_FIELDS::UF_DEPARTMENT,
        ]
    }

    fn to_string(&self) -> String {
        match self {
            ADDITIONAL_FIELDS::WORK_POSITION => "WORK_POSITION",
            ADDITIONAL_FIELDS::PERSONAL_BIRTHDAY => "PERSONAL_BIRTHDAY",
            ADDITIONAL_FIELDS::UF_DEPARTMENT => "UF_DEPARTMENT",
        }
        .to_string()
    }
}

struct Operation {
    id_for_item: i32,
    map_params: HashMap<String, String>,
}

struct Operations {
    data: Vec<Operation>,
}

impl Operation {
    fn new(id: i32, m: HashMap<String, String>) -> Self {    Operation {            id_for_item: id,            map_params: m,        }    }
    fn to_string(&self) -> String {format!("Struct Operation::\nid::{}, params::{}", self.id_for_item,  Operation::map_to_string(self.map_params.clone()) ) }
    fn map_to_string(m: HashMap<String, String>) -> String {m.iter().map(|(key, value)| format!("{}: {}", key, value)).collect::<Vec<String>>().join(", ")    }
}

fn get_enum__by_string(target: &str) -> ADDITIONAL_FIELDS {    ADDITIONAL_FIELDS::all_values().iter().find(|&field| field.to_string() == target).cloned().unwrap_or_else(|| panic!("No field found with string: {}", target))}

impl std::fmt::Display for ADDITIONAL_FIELDS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {  ADDITIONAL_FIELDS::WORK_POSITION => write!(f, "Должность"),  ADDITIONAL_FIELDS::PERSONAL_BIRTHDAY => write!(f, "День рождения"),  ADDITIONAL_FIELDS::UF_DEPARTMENT => write!(f, "Отдел пользователя"),
        }
    }
}

fn deserialize_from_file<T: DeserializeOwned>(
    filename: &str,
) -> Result<T, Box<dyn std::error::Error>> {   
    let mut file: File = File::open(filename)?;   
    let mut buffer: Vec<u8> = Vec::new();   
     file.read_to_end(&mut buffer)?;
    let decoded: T = bincode::deserialize(&buffer)?;
    Ok(decoded)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Employee {    id: i32,    name: String,    last_name: String,    middle_name: String,    map_add: HashMap<ADDITIONAL_FIELDS, String>,}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Pack {    pack: Vec<Employee>,}

impl Pack {
    fn new(pack: Vec<Employee>) -> Self {        Pack { pack }    }

    fn serialize_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let encoded: Vec<u8> = bincode::serialize(self)?;
        let mut file = File::create(filename)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    fn deserialize_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {        deserialize_from_file(filename)    }

    fn push_and_update(&mut self, entry: Employee) -> () {
        if self.is_contains(&entry) {            self.remove(&entry);        }
        self.pack.push(entry);
    }

    fn is_contains(&self, entry: &Employee) -> bool {
        let entry_id = entry.id;
        let entry_name = &entry.name;
        let entry_last_name = &entry.last_name;

        for emp in &self.pack {
            let cur_id = emp.id;
            let cur_name = &emp.name;
            let cur_last_name = &emp.last_name;

            if cur_id == entry_id {                return true;            }

            if (cur_name == entry_name) && (cur_last_name == entry_last_name) {                return true;            }
        }
        false
    }

    fn remove(&mut self, entry: &Employee) -> bool {
        let index = self.pack.iter().position(|emp| {
            emp.id == entry.id || (emp.name == entry.name && emp.last_name == entry.last_name)
        });

        if let Some(idx) = index {            self.pack.remove(idx);            true} 
        else {            false        }
    }

    fn to_string(&self, ender: String) -> String {
        let mut result = String::from("");
        for emp in &self.pack {
            result.push_str(&emp._to_string());
            result.push_str(&ender);
        }
        result
    }

    fn to_string_poetic(&self, ender: String) -> String {
        let mut result = String::from("");
        for emp in &self.pack {
            result.push_str(&&emp._to_string_poetic());
            result.push_str(&ender);
        }
        result
    }

    fn get_id_by_fi(&self, fi: String) -> Option<i32> {
        let splitted: Vec<String> = split_to_fi(fi);
        if splitted.len() < 3 {            return None;        }

        let last_name = &splitted[0];
        let first_name = &splitted[1];

        println!("F:{}", last_name.to_string());
        println!("I:{}", first_name.to_string());

        for employee in &self.pack {
            println!("EMPLOYEE    F:{}", employee.last_name.to_string());
            println!("EMPLOYEE    I:{}", employee.name.to_string());
            if employee.last_name == last_name.to_string()  && employee.name == first_name.to_string()  {                return Some(employee.id);            }
        }
        None
    }
}

impl Employee {
    fn new(        id: i32,        name: String,        last_name: String,        middle_name: String,        map_add: HashMap<ADDITIONAL_FIELDS, String>,    ) -> Self {
        Employee {            id,            name,            last_name,            middle_name,            map_add,        }}

    fn serialize_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let encoded: Vec<u8> = bincode::serialize(self)?;
        let mut file = File::create(filename)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    fn deserialize_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {        deserialize_from_file(filename)    }


    fn _to_string(&self) -> String {
        format!(            "{} {} {} {} {}",            self.id,            self.last_name,            self.name,            self.middle_name,            format!("<{}>", Employee::map_to_string(self.map_add.clone())))
    }

    fn _to_string_poetic(&self) -> String {
        format!( "{} {} {} {} {}",  self.id,  self.name,  self.middle_name,  self.last_name,  format!("<{}>", Employee::map_to_string(self.map_add.clone()))        )
    }

    fn map_to_string(m: HashMap<ADDITIONAL_FIELDS, String>) -> String {
        m.iter().map(|(key, value)| format!("{}: {}", key, value)).collect::<Vec<String>>().join(", ")
    }
}

fn get_i32_from_value(value: &Value) -> Option<i32> {
    match value {        Value::Number(n) => n.as_i64().map(|x| x as i32),        Value::String(s) => s.parse::<i32>().ok(),        _ => None,    }
}

fn codegen(data: String, filter_names: &[&str]) -> () {
    let v: Value = serde_json::from_str(&data).expect("ERROR PARCING");
    if let Some(items) = v["result"]["items"].as_array() {
        for item in items {
            let title = item["title"].as_str().unwrap_or("");
            let id = item["id"].as_str().unwrap_or("");
            if filter_names.contains(&title) {
                println!("{:?} => {:?}, ", title, id)
            }
        }
    } else {
        eprint!("Не найден массив items")
    }
}

fn codegen2(data: String, collabs: &[Collab]) {
    let v: Value = serde_json::from_str(&data).expect("ERROR PARSING");

    let mut title_to_id: HashMap<&str, &str> = HashMap::new();
    if let Some(items) = v["result"]["items"].as_array() {
        for item in items {
            if item["type"].as_str() == Some("chat") {
                let title = item["title"].as_str().unwrap_or("");
                let id = item["id"].as_str().unwrap_or("");
                title_to_id.insert(title, id);
            }
        }
    } else {
        eprintln!("Не найден массив items");
        return;
    }

    println!("const CHATS_ID: std::collections::HashMap<Collab, &str> = hashmap!(");
    for collab in collabs {
        let title = collab.title(); // &str
        if let Some(&id) = title_to_id.get(title) {
            println!("    Collab::{:?} => {:?},", collab, id);
        }
    }
    println!(");");
}

fn p(s: &Value) -> String {    s.to_string().replace("\"", "")}

fn process_no_mobile(arr_list: Vec<String>, pack: Pack, filename_out: String) -> Pack {
    //filename_out :: Binary Pack
    let pack = Pack::new(Vec::new());
    pack
}

async fn grub_data(    index_start: i32,    index_stop: i32,    filename_to_dump: &str,) -> Result<(), Box<dyn std::error::Error>> {
    let init_buffer: Vec<Employee> = Vec::new();
    let mut pack = Pack::new(init_buffer);
    let client_reqwest: Client = reqwest::Client::new();
    for _i in index_start..index_stop {
        match http_Test::get_user_by_id(client_reqwest.clone(), _i).await {
            Ok(data) => {
                if let Some(users) = data.get("result") {
                    if users.is_array() {
                        for user in users.as_array().unwrap() {
                            println!("------------------------------------------");
                            println!("USER:: {}", user);
                            let id = user.get("ID").unwrap_or(&Value::Null);
                            let name = user.get("NAME").unwrap_or(&Value::Null);
                            let last_name = user.get("LAST_NAME").unwrap_or(&Value::Null);
                            let second_name = user.get("SECOND_NAME").unwrap_or(&Value::Null);
                            let work_position = user.get("WORK_POSITION").unwrap_or(&Value::Null);
                            println!("ID: {}", id);
                            println!("Имя: {}", name);
                            println!("Фамилия: {}", last_name);
                            println!("Отчество: {}", second_name);
                            println!("Должность: {}", work_position);
                            println!("---");
                            let number_id = get_i32_from_value(id).expect("shit");
                            let emp = Employee::new(
                                number_id,
                                p(name),
                                p(last_name),
                                p(second_name),
                                HashMap::from([(
                                    ADDITIONAL_FIELDS::WORK_POSITION,
                                    work_position.to_string(),
                                )]),
                            );
                            pack.push_and_update(emp);
                        }
                    }
                }
            }
            Err(e) => println!("Ошибка: {}", e),
        }
    }
    println!("RESULT:: {}", pack.to_string("\n".to_string()));
    pack.serialize_to_file(filename_to_dump);
    Ok(())
}

async fn grub_data_with_add_params(    index_start: i32,    index_stop: i32,    filename_to_dump: &str,    params: Vec<String>,) -> Result<(), Box<dyn std::error::Error>> {
    let init_buffer: Vec<Employee> = Vec::new();
    let mut pack = Pack::new(init_buffer);
    let client_reqwest: Client = reqwest::Client::new();
    for _i in index_start..index_stop {
        match http_Test::get_user_by_id(client_reqwest.clone(), _i).await {
            Ok(data) => {
                if let Some(users) = data.get("result") {
                    if users.is_array() {
                        for user in users.as_array().unwrap() {
                            println!("------------------------------------------");
                            println!("USER:: {}", user);
                            let id = user.get("ID").unwrap_or(&Value::Null);
                            let name = user.get("NAME").unwrap_or(&Value::Null);
                            let last_name = user.get("LAST_NAME").unwrap_or(&Value::Null);
                            let second_name = user.get("SECOND_NAME").unwrap_or(&Value::Null);

                            let mut map22: HashMap<ADDITIONAL_FIELDS, String> = HashMap::new();

                            for item in params.iter() {
                                let enum_ = get_enum__by_string(item);
                                let value = user.get(enum_.to_string()).unwrap_or(&Value::Null);
                                map22.insert(enum_, value.to_string());
                            }

                            println!("ID: {}", id);
                            println!("Имя: {}", name);
                            println!("Фамилия: {}", last_name);
                            println!("Отчество: {}", second_name);
                            println!("ДОП ПАРАМЕТРЫ: {}", Employee::map_to_string(map22.clone()));

                            println!("---");
                            let number_id = get_i32_from_value(id).expect("shit");
                            let emp = Employee::new(
                                number_id,
                                p(name),
                                p(last_name),
                                p(second_name),
                                map22,
                            );
                            pack.push_and_update(emp);
                        }
                    }
                }
            }
            Err(e) => println!("Ошибка: {}", e),
        }
    }
    println!("RESULT:: {}", pack.to_string("\n".to_string()));
    pack.serialize_to_file(filename_to_dump);
    Ok(())
}

async fn try_grub() -> Result<(), Box<dyn std::error::Error>> {
    let res = grub_data_with_add_params(        1,        450,        ADD_DUMP,        vec![          ADDITIONAL_FIELDS::WORK_POSITION.to_string(),
            ADDITIONAL_FIELDS::PERSONAL_BIRTHDAY.to_string(),
            ADDITIONAL_FIELDS::UF_DEPARTMENT.to_string(),
        ],
    ).await; //DEFAULT_DUMP).await;

    let pack = Pack::deserialize_from_file(ADD_DUMP).expect("msg");
    println!("RESULT:: {}", pack.to_string("\n".to_string()));
    let filename = "lastdump.txt";
    std::fs::write(filename, pack.to_string("\n".to_string()));
    res
}

async fn send_message(msg: &str, id_to_send: i32) -> Result<(), Box<dyn std::error::Error>> {
    http_Test::send_notification_to_user(reqwest::Client::new(), &id_to_send.to_string(), msg).await?;
    Ok(())
}

async fn process_packed_no_mobile(filename: String) -> Result<(), Box<dyn std::error::Error>> {
    let lines = read_lines_utf8(&filename);
    let direct_message = "Здравствуйте! В связи с нестабильной работой WhatsUp, рабочие чаты будут переноситься из WhatsApp в Bitrix24. 
    Необходимо установить Bitrix24 на телефон. 
    При возникновении трудностей можете написать ко мне в личку или обратиться в ИТ отдел";

    for item in lines.iter() {
        let v: Vec<String> = item.split_whitespace().map(|s| s.to_string()).collect();
        let id3 = get_index_via_fio_result(v.clone(), "USERS.init");
        let id = get_index_via_fio_result(v.clone(), "USERS.init").expect("error");

        if id3 == None {            panic!("ID is null");        }

        println!("ID as string: '{}'", &id.to_string());
        println!("ID as string trimmed: '{}'", &id.to_string().trim());

        send_message(direct_message, id3.unwrap()).await;
        println!("\n\n\n\n\nID::{}", id3.unwrap());

    }
    Ok(())
}

async fn process(client_reqwest: Client) -> Result<(), Box<dyn std::error::Error>> {
    let FOMINA = [("SECOND_NAME", "Александровна"), ("ID", "292")];
    let z1 = [("SECOND_NAME", "Вячеславовна"), ("ID", "225")];

    let z11 = [("SECOND_NAME", "Викторовна"), ("ID", "233")];
    let z12 = [("SECOND_NAME", "Юрьевна"), ("ID", "235")];
    let z13 = [("SECOND_NAME", "Алексеевна"), ("ID", "241")];
    let z14 = [("SECOND_NAME", "Ахмеднадырович"), ("ID", "247")];
    let z15 = [("SECOND_NAME", "Владимирович"), ("ID", "255")];
    let z16 = [("SECOND_NAME", "Мураткалиевич"), ("ID", "259")];
    let z17 = [("SECOND_NAME", "Магарамовна"), ("ID", "267")];
    let z18 = [("SECOND_NAME", "Сергеевна"), ("ID", "283")];
    let z19 = [("SECOND_NAME", "Петрович"), ("ID", "285")];

    let z20 = [("ID", "283"), ("PERSONAL_MOBILE", "+7927581-68-51")];
    let z22 = [        ("ID", "283"),        ("WORK_POSITION", "Ведущий инженер-конструктор"),    ];
    let z21 = [        ("ID", "285"),        ("WORK_POSITION", "Специалист по корпоративной безопасности"),    ];

    let arr = vec![z1, z11, z12, z13, z14, z15, z16, z17, z18, z19];

    let arr2: Vec<[(&str, &str); 2]> = vec![z20, z21, z22];

    for item in arr.iter() {
        let result = http_Test::update_param_(client_reqwest.clone(), item).await?;
        println!("Response body: {}", result);
    }

    for item in arr2.iter() {
        let result = http_Test::update_param_(client_reqwest.clone(), item).await?;
        println!("Response body: {}", result);
    }
    Ok(())
}

fn gen_batch_str(input: String, pack: Pack, _1c_info_file: String) -> String {    return "".to_string();}

async fn getting_users(client_reqwest: Client) -> () {
    println!("=== Получение пользователя по ID 111111111111111111111===");
    match http_Test::get_user_by_id(client_reqwest.clone(), 1).await {
        Ok(data) => {
            if let Some(users) = data.get("result") {
                if users.is_array() {
                    for user in users.as_array().unwrap() {
                        println!("ID: {}", user.get("ID").unwrap_or(&Value::Null));
                        println!("Имя: {}", user.get("NAME").unwrap_or(&Value::Null));
                        println!("Фамилия: {}", user.get("LAST_NAME").unwrap_or(&Value::Null));
                        println!(
                            "Отчество: {}",
                            user.get("SECOND_NAME").unwrap_or(&Value::Null)
                        );

                        /// println!("Email: {}", user.get("EMAIL").unwrap_or(&Value::Null));
                        println!("---");
                    }
                }
            }
        }
        Err(e) => println!("Ошибка: {}", e),
    }

    println!("\n=== Получение пользователя с определенными полями    222222222222222222 ===");
    let fields = vec!["ID", "NAME", "LAST_NAME", "EMAIL", "PERSONAL_MOBILE"];
    match http_Test::get_user_with_fields(client_reqwest.clone(), 1, &fields).await {
        Ok(data) => println!("{:#?}", data),
        Err(e) => println!("Ошибка: {}", e),
    }

    println!("\n=== Получение нескольких пользователей 333333333333333333333333333333===");
    let user_ids = vec![1, 2, 3];
    match http_Test::get_multiple_users(client_reqwest.clone(), &user_ids).await {
        Ok(data) => {
            if let Some(result) = data.get("result") {
                println!(
                    "Найдено пользователей: {}",
                    result.as_array().unwrap().len()
                );
            }
        }
        Err(e) => println!("Ошибка: {}", e),
    }
}

fn compare(fromVik: String, fromDump: String) -> bool {
    return false;

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_reqwest: Client = reqwest::Client::new();


    let server_handle = tokio::spawn(async {
        if let Err(e) = http_Proc::spawn().await {
            eprintln!("Server error: {}", e);
        }
    });
    try_grub().await;
    loop {
        println!("Main thread works...");
        thread::sleep(Duration::from_secs(1));
    }

    //  try_grub().await
    Ok(())
}

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn bad_add(a: i32, b: i32) -> i32 {
    a - b
}

pub fn split_to_fi(input: String) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    let splitted = input.split_whitespace();
    for _i in splitted {
        res.push(_i.to_string());
    }
    res.push(input);
    res
}

fn write_js_to_file(filename: &str, json: Value) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(filename)?;
    serde_json::to_writer_pretty(file, &json)?;
    Ok(())
}

async fn get_tasks_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let client_reqwest: Client = reqwest::Client::new();
    //let value = http_Test::read_tasks2(&client_reqwest).await?;
    let value = http_Test::read_tasks2(&client_reqwest).await?;
    write_js_to_file("dump2.js", value)
}

fn get_index_via_fio_result(fio: Vec<String>, filename: &str) -> Option<i32> {
    let lines = read_lines_utf8(filename);
    for item in lines.iter() {
        println!("current string {}", item);
        if fio.iter().all(|s| item.contains(s)) {
            if let Some(space_index) = item.find(' ') {
                if space_index > 0 {
                    if let Ok(num) = item[..space_index].parse::<i32>() {
                        return Some(num);
                    }
                }
            }
        }
    }
    None
}

fn synteka() -> String {
    return http_Test::get_webhook_(SYNTEKA_TOKEN_FILE);
}

fn find_dep_name_by_id(target_id: i32, lines: &[String]) -> Option<String> {
    let full = lines.concat();
    let pattern = format!("\"ID\":\"{}\",\"NAME\":\"", target_id);
    full.find(&pattern).and_then(|pos| {
        let after = &full[pos + pattern.len()..];
        let name: String = after.chars().take_while(|&c| c != '"').collect();
        if name.is_empty() { None } else { Some(name) }
    })
}

pub async fn get_last_id_for_collab(    collab: Collab,    client: Client,    webhook_url: &str,) -> anyhow::Result<u64> {
    let json = http_Proc::fetch_recent_list_raw(client, webhook_url, json!({})).await.context("Ошибка получения списка чатов")?;
    let items = json["result"]["items"].as_array().context("Нет поля result.items в JSON")?;
    let target_title = collab.title();
    for item in items {
        let item_type = item["type"].as_str().unwrap_or("");
        let title = item["title"].as_str().unwrap_or("");
        if item_type == "chat" && title == target_title {
            let last_id = item["last_id"]                .as_u64()                .context("Поле last_id отсутствует или не число")?;
            return Ok(last_id);
        }
    }
    anyhow::bail!("Чат с названием '{}' не найден", target_title);
}

pub fn extract_messages_from_json(value: &Value) -> Vec<ExtractedMessage> {
    let mut user_names = HashMap::new();
    if let Some(users) = value["result"]["users"].as_array() {
        for user in users {
            if let (Some(id), Some(name)) = (user["id"].as_u64(), user["name"].as_str()) {
                user_names.insert(id, name.to_string());
            }
        }
    }

    let mut result = Vec::new();
    if let Some(messages) = value["result"]["messages"].as_array() {
        for msg in messages {
            let author_id = msg["author_id"].as_u64().unwrap_or(0);
            if author_id == 0 {                continue;            }
            let id = msg["id"].as_u64().unwrap_or(0);
            let chat_id = msg["chat_id"].as_u64().unwrap_or(0);
            let author_name = user_names                .get(&author_id)                .cloned()                .unwrap_or_else(|| format!("unknown_{}", author_id));
            let text = msg["text"].as_str().unwrap_or("").to_string();
            let uuid = msg["uuid"].as_str().map(|s| s.to_string());
            result.push(ExtractedMessage {                author_name,                text,                uuid,                id,                chat_id,            });
        }
    }
    result
}

#[cfg(test)]
mod tests {

    use chrono::format;
    use futures::TryFutureExt;

    use crate::http_Proc::fetch_recent_list_raw;

    use super::*;

    #[test]
    fn test_pperation() {
        let operation: Operation = Operation::new(12,Some((ADDITIONAL_FIELDS::WORK_POSITION.to_string(),"Главный гитарист".to_string(),)).into_iter().collect(),;
        let to_str = operation.to_string();
        println!("{}", to_str);
        assert_eq!(true, to_str.contains("id::12, params:"));
    }

    #[test]
    fn test_batch_func_gen() {        let input = "Курьянов Владимир Владимирович: должность сделать как в 1С";    }

    #[test]
    fn test_get_enum() {
        assert_eq!(            get_enum__by_string("WORK_POSITION"),            ADDITIONAL_FIELDS::WORK_POSITION        )
    }

    #[test]
    fn test_map() {
        let mut m: HashMap<ADDITIONAL_FIELDS, String> = HashMap::new();
        m.insert(ADDITIONAL_FIELDS::WORK_POSITION, "seller".to_string());
        assert_eq!("seller", m.get(&ADDITIONAL_FIELDS::WORK_POSITION).unwrap());
    }

    #[test]
    fn test_getindx() {
        let fio = vec!["Тест".to_string(), "Тестер".to_string()];
        let id = get_index_via_fio_result(fio, "USERS.init").expect("error");
        assert_eq!(296, id);
    }

    #[test]
    fn test_add() {        assert_eq!(add(1, 2), 3);    }

    #[test]
    fn test_bad_add() {        assert_eq!(bad_add(1, 2), -1);    }

    #[test]
    fn test_read_str() {
        let filename = "lstdata.csv";
        let etalon = ";ФИО;Должность;;;;телефон;юр. лицо";
        let vect = http_Test::read_lines(filename);
        let line0 = vect[0].clone();
        assert_eq!(etalon, line0);
    }

    #[test]
    fn test_read_webbhook() {
        const WEBHOOK_FILENAME_TEST: &str = "webhook_test";
        let test_webhook = http_Test::get_webhook_(WEBHOOK_FILENAME_TEST);
        let etalon = "http://google.com";
        assert_eq!(etalon, test_webhook);
    }

    #[test]
    fn test_ser() {
        let emp = Employee::new(1,  "John Doe".to_string(),  "DOE".to_string(),  "DOE".to_string(),  HashMap::new(),  );
        emp.serialize_to_file("employee.bin");
        println!("Data serialized to employee.bin");
        let emp2 = Employee::deserialize_from_file("employee.bin").expect("shit");
        println!("Deserialized data: {:?}", emp2);
        assert_eq!(emp, emp2);
    }

    #[test]
    fn test_sr_dsr() {
        let emp1 = Employee::new(1,"John Doe".to_string(),"DOE".to_string(),"DOE".to_string(),HashMap::new(),        );
        let emp2 = Employee::new(1,"John Doe".to_string(),"DOE".to_string(),"DOE".to_string(),HashMap::new(),        );
        let pack_filename = "pack.bin";
        let emp_Vecs = vec![emp1, emp2];
        let pack = Pack::new(emp_Vecs);
        pack.serialize_to_file(pack_filename);
        let restored = Pack::deserialize_from_file(pack_filename).expect("shit");
        assert_eq!(pack, restored);

        assert_eq!(2, 2);
    }

    #[test]
    fn test_contains_pack() {
        let emp1 = Employee::new(1,"Michael".to_string(),"Snoyman".to_string(),"".to_string(),HashMap::new(),        );
        let emp2 = Employee::new(2,"Roman".to_string(),"Pastushkov".to_string(),"DOE".to_string(),HashMap::new(),        );
        let pack_filename = "pack.bin";
        let emp_Vecs = vec![emp1, emp2];
        let mut pack = Pack::new(emp_Vecs);
        let emp3 = Employee::new(1,"Michael".to_string(),"Snoyman".to_string(),"".to_string(),HashMap::new(),        );
        let emp4 = Employee::new(99,"Michael".to_string(),"Snoyman".to_string(),"".to_string(),HashMap::new(),);
        let emp5 = Employee::new(99,"Michael2".to_string(),"Snoyman".to_string(),"".to_string(),HashMap::new(),);
        let emp6 = Employee::new(1,"Michael2".to_string(),"Snoyman".to_string(),"".to_string(),HashMap::new(),);

        let res = pack.is_contains(&emp3);        assert_eq!(true, res);

        let res2 = pack.is_contains(&emp4);        assert_eq!(true, res2);

        let res3 = pack.is_contains(&emp5);        assert_eq!(false, res3);

        let res4 = pack.is_contains(&emp6);        assert_eq!(true, res4);
        println!("{}", pack.to_string("\n".to_string()))
    }

    #[test]
    fn test_delete_pack() {
        let emp1 = Employee::new(1,"Michael".to_string(),"Snoyman".to_string(),"".to_string(),HashMap::new(),        );
        let emp2 = Employee::new(2,"Roman".to_string(),"Pastushkov".to_string(),"DOE".to_string(),HashMap::new(),        );
        let emp_Vecs = vec![emp1, emp2];
        let mut pack = Pack::new(emp_Vecs);
        let emp3 = Employee::new(1,"Michael".to_string(),"Snoyman".to_string(),"".to_string(),HashMap::new(),);

        let res = pack.is_contains(&emp3);        assert_eq!(true, res);

        pack.remove(&emp3);
        let res2 = pack.is_contains(&emp3);
        assert_eq!(false, res2);
    }

    #[test]
    fn test_get_id() {
        let emp1 = Employee::new(1,"Michaelen".to_string(),"Snoyman".to_string(),"".to_string(),HashMap::new(),        );
        let emp2 = Employee::new(2,"Роман".to_string(),"Пастушков".to_string(),"DOE".to_string(),HashMap::new(),        );
        let emp3 = Employee::new(1,"Michael".to_string(),"Snoyman".to_string(),"".to_string(),HashMap::new(),        );
        let emp_Vecs = vec![emp1, emp2, emp3];
        let mut pack = Pack::new(emp_Vecs);
        assert_eq!(2,pack.get_id_by_fi("Пастушков Роман".to_string()).expect("shit"));
    }

    #[test]
    fn test_get_id2() {
        let mut pack = Pack::deserialize_from_file(ADD_DUMP).expect("msg");
        println!("{}", pack.to_string("\n".to_string()));
        assert_eq!(            1,            pack.get_id_by_fi("Цыбульский Сергей".to_string())                .expect("shit")        );
    }

    #[test]
    fn test_get_id2_poetic() {
        let mut pack = Pack::deserialize_from_file(ADD_DUMP).expect("msg");
        println!("{}", pack.to_string_poetic("\n".to_string()));
        assert_eq!(            1,            pack.get_id_by_fi("Цыбульский Сергей".to_string())                .expect("shit")        );
    }

    #[test]
    fn test_fi() {
        let mut fi = "Roman Pastushkov";
        assert_eq!(3, split_to_fi(fi.to_string()).len());
    }

    #[test]
    fn suspend_to_file_dump() {
        let mut pack = Pack::deserialize_from_file(ADD_DUMP).expect("msg");
        println!("RESULT:: {}", pack.to_string("\n".to_string()));
        let filename = "lastdump.txt";
        let file = File::create(filename);
        let res = std::fs::write(filename, pack.to_string("\n".to_string()));
        // assert_eq!(res., Ok(()))
    }

    #[test]
    fn test_read_deps() {
        let strs = read_lines_utf8("deps.js");
        let part1 = find_dep_name_by_id(17, &strs).unwrap();
        assert_eq!(part1, "Юридический отдел");

        for n in 0..10000 {
            let dep = find_dep_name_by_id(n, &strs).unwrap_or("efes".to_string());
            if dep != "efes" {                println!("{}:{}\n", n, dep);            }
        }
    }

    #[test]
    fn test_synteka_token() {
        let etalon = "token";
        let mut file = File::create(SYNTEKA_TOKEN_FILE).expect("cant write file");
        file.write_all(etalon.as_bytes());
        assert_eq!(etalon, synteka());
    }
    const WEBHOOK_TEST_BASE__: &str = "webhook.test";
    const WEBHOOCK_PROD_CHAT__: &str = "webhook.prod";

    fn webhook_base_test() -> String {        http_Proc::get_webhook_(WEBHOOK_TEST_BASE__)    }

    fn webhook_base_prod() -> String {        http_Proc::get_webhook_(WEBHOOCK_PROD_CHAT__)    }

    #[tokio::test]
    async fn test_pull_messages() {
        let id = 56;
        let limit = 120;
        let out = "./out7.js";
        let Client = Client::new();
        let dialog_id = "chat8";  
        http_Proc::pull_messages(Client::new(),webhook_base_test().as_str(),dialog_id,id,limit,out,).await;
    }

    #[tokio::test]
    async fn test_fetch_recent_chats() {let _ = http_Proc::fetch_recent_list(Client::new(),webhook_base_test().as_str(),json!({}),"recent_chats.js",).await;}

    #[tokio::test]
    async fn test_pull_messages_prod() {
        let id = 56;
        let limit = 120;
        let out = "./out7.js";
        let chatid = CHATS_ID.get(&Collab::OKLAND).unwrap().to_string();
        http_Proc::pull_messages(Client::new(),webhook_base_prod().as_str(),&chatid,id,limit,out,        ).await;
    }

    #[tokio::test]
    async fn test_pull_messages_prod_okland() {
        let id =get_last_id_for_collab(Collab::OKLAND, Client::new(), webhook_base_prod().as_str()).await.unwrap();
        let limit = 120;
        let out = format!("{}_last{}.js", OKLAND, limit);
        let _ = http_Proc::pull_messages(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&Collab::OKLAND).expect("OKLAND not found"),  id as i64,limit,&out,).await;
    }

    #[tokio::test]
    async fn test_fetch_recent_chats_prod() {        let _ = http_Proc::fetch_recent_list( Client::new(),webhook_base_prod().as_str(),json!({}),"recent_chats_prod.js",).await;}

    #[tokio::test]
    async fn test_main() {
        let js =            fetch_recent_list_raw(Client::new(), webhook_base_prod().as_str(), json!({})).await;
        match js {
            Ok(js) => {
                println!("RAW JSON::{}", js);
                let res = http_Parser::print_chats(js.to_string());
                assert_eq!(0, res);
            }
            Err(e) => {                panic!("SHIT HAPPENS: {}", e)           }
        }
    }

    #[tokio::test]
    async fn test_codegen() {
        let js = http_Proc::fetch_recent_list_raw(Client::new(),webhook_base_prod().as_str(),json!({}),).await;

        match js {
            Ok(value) => {
                let pretty = serde_json::to_string_pretty(&value);
                match pretty {
                    Ok(nice_str) => {println!("GENNED:\n{}\n\n\n\n", nice_str);codegen2(nice_str, VECTORS_COLLABS);}
                    Err(e) => {panic!("SHIT HAPPENS: {}", e)}
                }
            }

            Err(e) => {                panic!("SHIT HAPPENS22222: {}", e)}
        }
    }

    #[tokio::test]
    async fn test_pull_messages_prod_okland2() {
        let id =            get_last_id_for_collab(Collab::OKLAND, Client::new(), webhook_base_prod().as_str()).await.unwrap();

        println!("\n\n\n\nLAST ID IN OKLAND:: {}\n\n\n", id);
        let limit = 120;

        let json_value = http_Proc::pull_messages_raw(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&Collab::OKLAND).expect("OKLAND not found"),
            (id + 2) as i64,  limit,        ).await.unwrap();

        let messages = extract_messages_from_json(&json_value);
        println!("Извлечено {} сообщений", messages.len());
        for msg in messages.iter() {                     println!("{:?}", msg);        }

        let out_extracted = format!("{}_last{}_extracted.json", "OKLAND", limit);
        let json_output = serde_json::to_string_pretty(&messages).unwrap();
        std::fs::write(out_extracted, json_output).unwrap();

        let _ = std::fs::write(            format!("{}_last{}_extracted_FULL.json", "OKLAND", limit),serde_json::to_string_pretty(&json_value).unwrap(),        );
    }

    #[tokio::test]
    async fn test_pull_messages_prod_Scandinavia2() {
        let current_collab = Collab::SCANDINAVIA;
        let title = current_collab.title();
        let id =            get_last_id_for_collab(current_collab, Client::new(), webhook_base_prod().as_str()).await.unwrap();

        println!("\n\n\n\nLAST ID IN {}:: {}\n\n\n", title, id);
        let limit = 120;

        let json_value = http_Proc::pull_messages_raw(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&current_collab).expect(&format!("{} not found", title)),(id + 1) as i64, // <=============== pull with last!
            limit,).await.unwrap();

        let messages = extract_messages_from_json(&json_value);
        println!("Извлечено {} сообщений", messages.len());
        for msg in messages.iter() {               println!("{:?}", msg);        }

        let out_extracted = format!("{}_last{}_extracted.json", current_collab.title(), limit);
        let json_output = serde_json::to_string_pretty(&messages).unwrap();
        std::fs::write(out_extracted, json_output).unwrap();

        let _ = std::fs::write(format!("{}_last{}_extracted_FULL.json",current_collab.title(),limit),serde_json::to_string_pretty(&json_value).unwrap(),);
    }

    #[tokio::test]
    async fn test_pull_messages_prod_OWN() {
        let current_collab = Collab::OWN;
        let title = current_collab.title();
        let id =get_last_id_for_collab(current_collab, Client::new(), webhook_base_prod().as_str()).await.unwrap();

        println!("\n\n\n\nLAST ID IN {}:: {}\n\n\n", title, id);
        let limit = 120;

        let json_value = http_Proc::pull_messages_raw(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&current_collab)
                .expect(&format!("{} not found", title)),(id + 1) as i64,limit,).await.unwrap();

        let messages = extract_messages_from_json(&json_value);
        println!("Извлечено {} сообщений", messages.len());
        for msg in messages.iter() {                    println!("{:?}", msg);        }

        let out_extracted = format!("{}_last{}_extracted.json", current_collab.title(), limit);
        let json_output = serde_json::to_string_pretty(&messages).unwrap();
        std::fs::write(out_extracted, json_output).unwrap();

        let _ = std::fs::write(format!("{}_last{}_extracted_FULL.json",current_collab.title(),limit),serde_json::to_string_pretty(&json_value).unwrap(),);
    }
}

////
////
////
//// <div class="bx-im-message-base__container"><div class="bx-im-message-base__content"><div class="bx-im-message-base__body"><div class="bx-im-message-default__container"><div class="bx-im-message-author-title__container --clickable"><div class="bx-im-chat-title__scope bx-im-chat-title__container"><span class="bx-im-chat-title__content"><!----><span class="bx-im-chat-title__text" title="Сергей Музданбаев" style="color: rgb(88, 204, 71);">Сергей Музданбаев</span><!----><!----><!----></span></div></div><div class="bx-im-message-default-content__container bx-im-message-default-content__scope"><div class="bx-im-message-quote --reply --collapsed --clickable" data-context="chat6986/117754"><div class="bx-im-message-quote__wrap"><div class="bx-im-message-quote__name"><div class="bx-im-message-quote__name-text">Дмитрий Бердников</div></div><div class="bx-im-message-quote__text">Прошу согласовать на 24.04.26  к 13:00 манипулятор для перевозки уголков и перемычек с Куйбышевой 86 на Рыбацкую <br><br>Конт. тел. 89170911410 Дмитрий</div><!----></div></div><div class="bx-im-message-default-content__text">Согласовано</div><!----><div class="bx-im-message-default-content__bottom-panel"><!----><div class="bx-im-message-default-content__status-container"><div class="bx-im-message-status__container"><!----><div class="bx-im-message-status__date">14:06</div><!----></div></div></div></div></div><!----><div class="bx-im-reaction-selector__container"><div class="bx-im-reaction-selector__selector"><div class="bx-im-reaction-selector__icon"></div></div></div></div><div class="bx-im-message-context-menu__container bx-im-message-context-menu__scope"><button title="Кликните для открытия меню действий или удерживайте CTRL для цитирования сообщения" class="bx-im-message-context-menu__button"></button></div></div><!----></div>
////
////
////
////
