use crate::http_test::{read_lines_utf8};
use bincode;
use reqwest;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str, json};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
pub mod http_Parser;
pub mod http_Proc;
pub mod http_test;
pub mod http_synteka;
pub mod r_oc;
use std::thread;
use std::time::Duration;
use common::*;

pub    const EMPLOYES_FILE_NAME_JS  : &str = "employess_cynteka.json";
pub    const COLLEGUES_FILE_NAME_JS : &str = "collegues_cynteka.json";

const DEFAULT_DUMP: &str = "all_dump.bin";
const ADD_DUMP: &str = "snoyman.bin";
const SYNTEKA_TOKEN_FILE: &str = "synteka";
const CL_ADDRESS_FILE: &str = "cl_address";


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum ADDITIONAL_FIELDS {    WORK_POSITION,    PERSONAL_BIRTHDAY,    UF_DEPARTMENT,}

impl ADDITIONAL_FIELDS { fn all_values() -> Vec<Self> {vec![ADDITIONAL_FIELDS::WORK_POSITION,ADDITIONAL_FIELDS::PERSONAL_BIRTHDAY,ADDITIONAL_FIELDS::UF_DEPARTMENT,]    }

    fn to_string(&self) -> String {match self {ADDITIONAL_FIELDS::WORK_POSITION => "WORK_POSITION",ADDITIONAL_FIELDS::PERSONAL_BIRTHDAY => "PERSONAL_BIRTHDAY",ADDITIONAL_FIELDS::UF_DEPARTMENT => "UF_DEPARTMENT",}.to_string()}
}

struct Operation {    id_for_item: i32,    map_params: HashMap<String, String>,}

struct Operations {    data: Vec<Operation>,}

impl Operation {
    fn new(id: i32, m: HashMap<String, String>) -> Self {Operation {id_for_item: id,map_params: m}}
    fn to_string(&self) -> String {format!("Struct Operation::\nid::{}, params::{}", self.id_for_item,  Operation::map_to_string(self.map_params.clone()) ) }
    fn map_to_string(m: HashMap<String, String>) -> String {m.iter().map(|(key, value)| format!("{}: {}", key, value)).collect::<Vec<String>>().join(", ")    }
}

fn get_enum__by_string(target: &str) -> ADDITIONAL_FIELDS {    ADDITIONAL_FIELDS::all_values().iter().find(|&field| field.to_string() == target).cloned().unwrap_or_else(|| panic!("No field found with string: {}", target))}

impl std::fmt::Display for ADDITIONAL_FIELDS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {  ADDITIONAL_FIELDS::WORK_POSITION => write!(f, "Должность"),  ADDITIONAL_FIELDS::PERSONAL_BIRTHDAY => write!(f, "День рождения"),  ADDITIONAL_FIELDS::UF_DEPARTMENT => write!(f, "Отдел пользователя"),}
    }
}

fn deserialize_from_file<T: DeserializeOwned>(    filename: &str,) -> Result<T, Box<dyn std::error::Error>> {   
    let mut file: File = File::open(filename)?;   
    let mut buffer: Vec<u8> = Vec::new();   
    file.read_to_end(&mut buffer)?;
    let decoded: T = bincode::deserialize(&buffer)?;
    Ok(decoded)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct MsgPack {    pack: Vec<ExtractedMessage>,}

impl MsgPack {
    fn new(pack: Vec<ExtractedMessage>) -> Self {MsgPack { pack }}

    fn serialize_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = bincode::serialize(self)?;
        let mut file = File::create(filename)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    fn deserialize_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {        deserialize_from_file(filename)    }

    fn push_and_update(&mut self, entry: ExtractedMessage) {
        if self.is_contains(&entry) {self.remove(&entry);}
        self.pack.push(entry);
    }

    fn is_contains(&self, entry: &ExtractedMessage) -> bool {
        if let Some(uuid) = &entry.uuid {self.pack.iter().any(|msg| msg.uuid.as_ref() == Some(uuid))        } 
        else {false}
    }

    fn remove(&mut self, entry: &ExtractedMessage) -> bool {
        if let Some(pos) = self.pack.iter().position(|msg| msg.uuid == entry.uuid) {
            self.pack.remove(pos);
            true} 
        else {false}
    }
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

    fn deserialize_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {deserialize_from_file(filename)}

    fn push_and_update(&mut self, entry: Employee) -> () {
        if self.is_contains(&entry) {self.remove(&entry);}
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

            if cur_id == entry_id {return true;}

            if (cur_name == entry_name) && (cur_last_name == entry_last_name) {return true;}
        }
        false
    }

    fn remove(&mut self, entry: &Employee) -> bool {
        let index = self.pack.iter().position(|emp| {
            emp.id == entry.id || (emp.name == entry.name && emp.last_name == entry.last_name)
        });
        if let Some(idx) = index {self.pack.remove(idx);true} 
        else {false}
    }

    fn to_string(&self, ender: String) -> String {
        let mut result = String::from("");
        for emp in &self.pack {result.push_str(&emp._to_string());result.push_str(&ender);}
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

    fn  to_string_poetic34(&self, ender: String) -> String {
        let mut result = String::from("");
        for emp in &self.pack {
            result.push_str(&&emp._to_string_poetic34());
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
            if employee.last_name == last_name.to_string()  && employee.name == first_name.to_string()  {return Some(employee.id);}
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


    fn _to_string(&self) -> String {format!("{} {} {} {} {}",self.id,self.last_name,self.name,self.middle_name,format!("<{}>", Employee::map_to_string(self.map_add.clone())))}

    fn _to_string_poetic(&self) -> String {format!( "{} {} {} {} {}",  self.id,  self.name,  self.middle_name,  self.last_name,  format!("<{}>", Employee::map_to_string(self.map_add.clone())))}

    fn _to_string_poetic34(&self) -> String {format!( "{} {} {} {} ",  self.id,  self.name,  self.middle_name,  self.last_name)}


    fn map_to_string(m: HashMap<ADDITIONAL_FIELDS, String>) -> String {m.iter().map(|(key, value)| format!("{}: {}", key, value)).collect::<Vec<String>>().join(", ")}
}

fn get_i32_from_value(value: &Value) -> Option<i32> {match value {Value::Number(n) => n.as_i64().map(|x| x as i32),Value::String(s) => s.parse::<i32>().ok(),_ => None,    }}

fn codegen(data: String, filter_names: &[&str]) -> () {
    let v: Value = serde_json::from_str(&data).expect("ERROR PARCING");
    if let Some(items) = v["result"]["items"].as_array() {
        for item in items {
            let title = item["title"].as_str().unwrap_or("");
            let id = item["id"].as_str().unwrap_or("");
            if filter_names.contains(&title) {                println!("{:?} => {:?}, ", title, id)            } }} 
    else {        eprint!("Не найден массив items")    }
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
        }} 
    else {eprintln!("Не найден массив items");return;}

    println!("const CHATS_ID: std::collections::HashMap<Collab, &str> = hashmap!(");
    for collab in collabs {
        let title = collab.title(); // &str
        if let Some(&id) = title_to_id.get(title) {println!("Collab::{:?} => {:?},", collab, id);}
    }
    println!(");");
}

fn p(s: &Value) -> String {s.to_string().replace("\"", "")}

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
        match http_test::get_user_by_id(client_reqwest.clone(), _i).await {
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
                            let emp = Employee::new(number_id,p(name),p(last_name),p(second_name),
                            HashMap::from([(ADDITIONAL_FIELDS::WORK_POSITION,work_position.to_string(),)]),
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
        match http_test::get_user_by_id(client_reqwest.clone(), _i).await {
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
                            let emp = Employee::new(number_id,p(name),p(last_name),p(second_name),map22,);
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
    let res = grub_data_with_add_params(        1,        700,        ADD_DUMP,        vec![          ADDITIONAL_FIELDS::WORK_POSITION.to_string(),
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
    http_test::send_notification_to_user(reqwest::Client::new(), &id_to_send.to_string(), msg).await?;
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

        if id3 == None {panic!("ID is null");}

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
        let result = http_test::update_param_(client_reqwest.clone(), item).await?;
        println!("Response body: {}", result);
    }

    for item in arr2.iter() {
        let result = http_test::update_param_(client_reqwest.clone(), item).await?;
        println!("Response body: {}", result);
    }
    Ok(())
}

fn gen_batch_str(input: String, pack: Pack, _1c_info_file: String) -> String {    return "".to_string();}

async fn getting_users(client_reqwest: Client) -> () {
    println!("=== Получение пользователя по ID 111111111111111111111===");
    match http_test::get_user_by_id(client_reqwest.clone(), 1).await {
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
    match http_test::get_user_with_fields(client_reqwest.clone(), 1, &fields).await {
        Ok(data) => println!("{:#?}", data),
        Err(e) => println!("Ошибка: {}", e),
    }

    println!("\n=== Получение нескольких пользователей 333333333333333333333333333333===");
    let user_ids = vec![1, 2, 3];
    match http_test::get_multiple_users(client_reqwest.clone(), &user_ids).await {
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

fn compare(fromVik: String, fromDump: String) -> bool {    return false;}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_reqwest: Client = reqwest::Client::new();
    let server_handle = tokio::spawn(async {if let Err(e) = http_Proc::spawn().await {eprintln!("Server error: {}", e);}});
    try_grub().await;
    loop {
        println!("Main thread works...");
        thread::sleep(Duration::from_secs(1));
    }
    //  try_grub().await
    Ok(())
}

pub fn add(a: i32, b: i32) -> i32 {    a + b}

pub fn bad_add(a: i32, b: i32) -> i32 {    a - b}

pub fn split_to_fi(input: String) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    let splitted = input.split_whitespace();
    for _i in splitted {        res.push(_i.to_string());    }
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
    let value = http_test::read_tasks2(&client_reqwest).await?;
    write_js_to_file("dump2.js", value)
}

fn get_index_via_fio_result(fio: Vec<String>, filename: &str) -> Option<i32> {
    let lines = read_lines_utf8(filename);
    for item in lines.iter() {
        println!("current string {}", item);
        if fio.iter().all(|s| item.contains(s)) {
            if let Some(space_index) = item.find(' ') {if space_index > 0 {if let Ok(num) = item[..space_index].parse::<i32>() {return Some(num);}}}
        }
    }
    None
}

pub fn synteka() -> String {    return http_test::get_webhook_(SYNTEKA_TOKEN_FILE);}
pub fn cl_address() -> String {    return http_test::get_webhook_(CL_ADDRESS_FILE);}


fn find_dep_name_by_id(target_id: i32, lines: &[String]) -> Option<String> {
    let full = lines.concat();
    let pattern = format!("\"ID\":\"{}\",\"NAME\":\"", target_id);
    full.find(&pattern).and_then(|pos| {
        let after = &full[pos + pattern.len()..];
        let name: String = after.chars().take_while(|&c| c != '"').collect();
        if name.is_empty() { None } else { Some(name) }
    })
}

    const WEBHOOK_TEST_BASE__: &str = "webhook.test";
    const WEBHOOCK_PROD_CHAT__: &str = "webhook.prod";
    const WEBHOOCK_PROC__: &str = "web2";
    const WEBHOOCK_DISK__: &str = "webhook.disk";



    pub fn webhook_base_test() -> String {        http_Proc::get_webhook_(WEBHOOK_TEST_BASE__)    }

    pub fn webhook_base_prod() -> String {        http_Proc::get_webhook_(WEBHOOCK_PROD_CHAT__)    }

    pub fn webhook_base_bp() -> String {        http_Proc::get_webhook_(WEBHOOCK_PROC__)    }

    pub fn webhook_disk() -> String {        http_Proc::get_webhook_(WEBHOOCK_DISK__)    }



    

pub async fn get_file_by_attached_id(
    client: &Client,
    base_webhook_url: &str,
    attached_id: i32,
) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {

let info_url = format!("{}/disk.file.get",  webhook_disk());
let req_info = json!({ "id": attached_id });

    // let info_url = format!("{}/disk.attachedObject.get", webhook_disk());
    // let req_info = json!({ "id": attached_id });

    let resp_info = client
        .post(&info_url)
        .header("Content-Type", "application/json")
        .json(&req_info)
        .send()
        .await?;

    let json_info: Value = resp_info.json().await?;

    if let Some(error) = json_info.get("error").and_then(|e| e.as_str()) {
        let desc = json_info
            .get("error_description")
            .and_then(|d| d.as_str())
            .unwrap_or("без описания");
        return Err(format!("Ошибка Bitrix24: {} - {}", error, desc).into());
    }

    let result = json_info
        .get("result")
        .ok_or("Нет поля 'result' в ответе")?;

    let download_url = result
        .get("DOWNLOAD_URL")
        .and_then(|v| v.as_str())
        .ok_or("Нет поля DOWNLOAD_URL")?;

    let filename = result
        .get("NAME")
        .and_then(|v| v.as_str())
        .unwrap_or("file.bin")
        .to_string();

    let download_full = if download_url.starts_with("http") {
        download_url.to_string()
    } else {
        let base = reqwest::Url::parse(base_webhook_url)?;
        let full = base.join(download_url)?;
        full.to_string()
    };

    let resp_file = client.get(&download_full).send().await?;
    if !resp_file.status().is_success() {
        return Err(format!("Ошибка скачивания: {}", resp_file.status()).into());
    }

    let file_bytes = resp_file.bytes().await?.to_vec();
    Ok((filename, file_bytes))
}




#[cfg(test)]
mod tests {

    use chrono::format;
    use futures::TryFutureExt;
    use crate::File;
    use crate::http_Proc::{consumer_loop_proc, fetch_recent_list_raw};

    use super::*;

    #[test]
    fn test_pperation() {
        let operation: Operation = Operation::new(12,Some((ADDITIONAL_FIELDS::WORK_POSITION.to_string(),"Главный гитарист".to_string(),)).into_iter().collect(),);
        let to_str = operation.to_string();
        println!("{}", to_str);
        assert_eq!(true, to_str.contains("id::12, params:"));
    }

    #[test]
    fn test_batch_func_gen() {        let input = "Курьянов Владимир Владимирович: должность сделать как в 1С";    }

    #[test]
    fn test_get_enum() {        assert_eq!(            get_enum__by_string("WORK_POSITION"),            ADDITIONAL_FIELDS::WORK_POSITION        )    }

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
        let vect = http_test::read_lines(filename);
        let line0 = vect[0].clone();
        assert_eq!(etalon, line0);
    }

    #[test]
    fn test_read_webbhook() {
        const WEBHOOK_FILENAME_TEST: &str = "webhook_test";
        let test_webhook = http_test::get_webhook_(WEBHOOK_FILENAME_TEST);
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


    #[test]    ////////// To ThirtyFOUR APP WEBDRIVER
    fn test_get_id2_poetic34() {
        let mut pack = Pack::deserialize_from_file(ADD_DUMP).expect("msg");
        println!("{}", pack.to_string_poetic34("\n".to_string()));
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


    #[tokio::test]
    async fn test_pull_messages() {
        let id = 56;
        let limit = 120;
        let out = "./out7.js";
        let Client = Client::new();
        let dialog_id = "chat13372";//"chat9796";//"chat8";  
        http_Proc::pull_messages(Client::new(),
        &webhook_base_prod().as_str(), //webhook_base_test().as_str(),
        dialog_id,id,limit,out,).await;
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
        let id =http_Proc::get_last_id_for_collab(Collab::OKLAND, Client::new(), webhook_base_prod().as_str()).await.unwrap();
        let limit = 120;
        let out = format!("{}_last{}.js", OKLAND, limit);
        let _ = http_Proc::pull_messages(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&Collab::OKLAND).expect("OKLAND not found"),  id as i64,limit,&out,).await;
    }


    #[tokio::test]    /////need repair failed!
    async fn test_pull_messages_prod_skan() {
        let id =http_Proc::get_last_id_for_collab(Collab::SCANDINAVIA, Client::new(), webhook_base_prod().as_str()).await.unwrap();
        let limit = 120;
        let out = format!("{}_last{}.js", SCANDINAVIA, limit);
        let _ = http_Proc::pull_messages(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&Collab::SCANDINAVIA).expect("SCANDINAVIA not found"),  id as i64,limit,&out,).await;
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
        let id =   http_Proc::get_last_id_for_collab(Collab::OKLAND, Client::new(), webhook_base_prod().as_str()).await.unwrap();

        println!("\n\n\n\nLAST ID IN OKLAND:: {}\n\n\n", id);
        let limit = 1200;

        let json_value = http_Proc::pull_messages_raw(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&Collab::OKLAND).expect("OKLAND not found"),
            (id + 2) as i64,  limit,        ).await.unwrap();

        let messages =  http_Proc::extract_messages_from_json(&json_value);
        println!("Извлечено {} сообщений", messages.len());
        for msg in messages.iter() {                     println!("{:?}", msg);        } 

        let out_extracted = format!("{}_last{}_extracted.json", "OKLAND", limit);
        let json_output = serde_json::to_string_pretty(&messages).unwrap();
        std::fs::write(out_extracted, json_output).unwrap();

        let _ = std::fs::write(format!("{}_last{}_extracted_FULL.json", "OKLAND", limit),serde_json::to_string_pretty(&json_value).unwrap(),        );
    }
    use crate::http_Proc::CHATS_ID;
    #[tokio::test]
    async fn test_pull_messages_prod_Scandinavia2() {
        let current_collab = Collab::SCANDINAVIA;
        let title = current_collab.title();
        let id =    http_Proc::         get_last_id_for_collab(current_collab, Client::new(), webhook_base_prod().as_str()).await.unwrap();

        println!("\n\n\n\nLAST ID IN {}:: {}\n\n\n", title, id);
        let limit = 1220;

        let json_value = http_Proc::pull_messages_raw(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&current_collab).expect(&format!("{} not found", title)),(id + 1) as i64, // <=============== pull with last!
            limit,).await.unwrap();

        let messages =  http_Proc::extract_messages_from_json(&json_value);
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
        let id = http_Proc::get_last_id_for_collab(current_collab, Client::new(), webhook_base_prod().as_str()).await.unwrap();

        println!("\n\n\n\nLAST ID IN {}:: {}\n\n\n", title, id);
        let limit = 120;

        let json_value = http_Proc::pull_messages_raw(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&current_collab)
                .expect(&format!("{} not found", title)),(id + 1) as i64,limit,).await.unwrap();

        let messages =  http_Proc::extract_messages_from_json(&json_value);
        println!("Извлечено {} сообщений", messages.len());
        for msg in messages.iter() {                    println!("{:?}", msg);        }

        let out_extracted = format!("{}_last{}_extracted.json", current_collab.title(), limit);
        let json_output = serde_json::to_string_pretty(&messages).unwrap();
        std::fs::write(out_extracted, json_output).unwrap();

        let _ = std::fs::write(format!("{}_last{}_extracted_FULL.json",current_collab.title(),limit),serde_json::to_string_pretty(&json_value).unwrap(),);
    }

    #[tokio::test]
    async fn test_create_order(){
            let token = synteka(); // можно взять из конфига
            let client = Client::new();
            let result = http_synteka::create_order(&token, &client).await;
            println!("Ответ сервера: {:#?}", result);
           // Ok(())
    }


    #[tokio::test]
    async fn test_get_deliver(){          ///////////DELIVERIES
            let token = synteka(); // можно взять из конфига
            let client = Client::new();
            let out = "./delivery.js";
            let result = http_synteka::fetch_and_save_deliveries(&token, &client, out).await;
            println!("Ответ сервера: {:#?}", result);
           // Ok(())
    }

    


     #[tokio::test]
    async fn test_download_attached_file() {
        // Получаем URL вебхука из переменной окружения (не храним в коде!)
        let webhook_url = webhook_disk();

        // ID прикреплённого файла из вашего примера
        let attached_id = 40582;

        let client = Client::new();
        let result = get_file_by_attached_id(&client, &webhook_url, attached_id).await;

        match result {
            Ok((filename, data)) => {
                // Проверяем, что файл не пустой
                assert!(!data.is_empty(), "Файл не должен быть пустым");
                println!("Файл '{}' успешно загружен, размер {} байт", filename, data.len());

                // Можно сохранить файл в папку test_output для визуальной проверки
                let output_dir = std::path::Path::new("./test_output");
                if !output_dir.exists() {
                    std::fs::create_dir_all(output_dir).unwrap();
                }
                std::fs::write(output_dir.join(&filename), data).unwrap();
            }
            Err(e) => {
                // Если ошибка связана с правами – даём понятное сообщение
                if e.to_string().contains("insufficient_scope") {
                    panic!("Вебхук не имеет прав на 'disk'. Добавьте разрешение в настройках вебхука.");
                } else {
                    panic!("Ошибка загрузки файла: {}", e);
                }
            }
        }
    }

    // Дополнительный тест, который проверяет только получение метаданных (без скачивания)
#[tokio::test]
async fn test_get_attached_metadata() {
    let webhook_url = webhook_disk();
    let attached_id = 40582;
    let client = Client::new();

    match get_file_by_attached_id(&client, &webhook_url, attached_id).await {
        Ok((name, data)) => {
            println!("✅ Файл успешно получен!");
            println!("📄 Имя: {}", name);
            println!("📦 Размер: {} байт ({} КБ)", data.len(), data.len() as f64 / 1024.0);
            // При необходимости можно вывести первые 30 байт для отладки
            // println!("🔍 Первые 30 байт: {:?}", &data[..data.len().min(30)]);
            assert!(!name.is_empty());
            assert!(!data.is_empty());
        }
        Err(e) => {
            eprintln!("❌ Ошибка при получении файла: {}", e);
            panic!("Тест провален из-за ошибки");
        }
    }
}


    


    use std::fs;

    #[tokio::test]
    async fn test_pull_employess() -> Result<() , anyhow::Error> {
        let url = "https://restetris.cynteka.ru/api/v1/refbooks/employees/3000014640";
        let token = synteka();
        let client = Client::new();
        let response = client.get(url).header("accept", "application/json").header("ZakupayToken", token).send().await?;

        if !response.status().is_success() {        anyhow::bail!("HTTP error: {}", response.status());    }

        let json_text = response.text().await?;
        let json_value: Value = serde_json::from_str(&json_text)?; 
        let pretty_json = serde_json::to_string_pretty(&json_value)?;

        fs::write(EMPLOYES_FILE_NAME_JS, pretty_json)?;
        println!("Ответ сохранён в response.json");
        Ok(())
    }


    const URL: &str = "https://restetris.cynteka.ru/api/v1/refbooks/colleagues?format=xml";
    const OUTPUT_FILE: &str = "colleagues.json";

    #[tokio::test]
    async fn test_pull_collegues() -> Result<() , anyhow::Error> {
        let client = Client::new();
        let response = client.get(URL).header("accept", "application/json").header("ZakupayToken", synteka()).send().await?;

    if !response.status().is_success() {        anyhow::bail!("HTTP error: {}", response.status());    }

    let body = response.text().await?;
    let json: Value = serde_json::from_str(&body)?;
    let pretty = serde_json::to_string_pretty(&json)?;
    fs::write(OUTPUT_FILE, pretty)?;
    println!("Ответ сохранён в {}", OUTPUT_FILE);
    Ok(())
    }
    


    #[test]
    fn test_extract_id_via_fio() {
        let etalon_id = 299;
        let js_content = fs::read_to_string(EMPLOYES_FILE_NAME_JS).expect("Не удалось прочитать файл");
        let json_value: Value = serde_json::from_str(&js_content).expect("Ошибка парсинга JSON");
        let vec_fio = vec!["Сидагалиев".to_string(), "Нурлан".to_string()];
        let result = http_synteka::get_id_user_via_fio_cynteka(vec_fio, &json_value).unwrap();
        assert_eq!(etalon_id, result);
    }

    #[test]
    fn test_extract_id_via_fio2() {
        let etalon_id = 276;
        let js_content = fs::read_to_string(EMPLOYES_FILE_NAME_JS).expect("Не удалось прочитать файл");
        let json_value: Value = serde_json::from_str(&js_content).expect("Ошибка парсинга JSON");
        let vec_fio = vec!["Музданбаев".to_string(), "Сергей".to_string()];
        let result = http_synteka::get_id_user_via_fio_cynteka(vec_fio, &json_value).unwrap();
        assert_eq!(etalon_id, result);
    }

    #[test]
    fn test_try_create_order(){

    }

    #[tokio::test]
    async fn test_thread_call(){ 
        http_Proc::process_function("FILENAME.bin".to_string()).await;
        std::thread::sleep(Duration::from_secs(5));//handle.join();
    }

    #[tokio::test]
    async fn test_pull_messages_prod_Payments_with_dump() {
        let current_collab = Collab::PAYMENTS;
        let title = current_collab.title();
        let id = http_Proc:: get_last_id_for_collab(current_collab, Client::new(), webhook_base_bp().as_str()).await.unwrap();

        println!("\n\n\n\nLAST ID IN {}:: {}\n\n\n", title, id);
        let limit = 1220;

        let json_value:Value = http_Proc::pull_messages_raw(Client::new(),webhook_base_bp().as_str(),CHATS_ID.get(&current_collab).expect(&format!("{} not found", title)),
        (id + 1) as i64,limit,    ).await.unwrap();
 
        let messages =  http_Proc::extract_messages_from_json(&json_value);
        println!("Извлечено {} сообщений", messages.len());
        for msg in messages.iter() {        println!("{:?}", msg);    }

        {
            let mut queue = http_Proc::QUEUE.lock();
            queue.extend(messages.clone());
        }

        let bin_filename = format!("{}_queue.bin", current_collab.title());
        {
            let queue_data = http_Proc::QUEUE.lock();
            let serialized = bincode::serialize(&*queue_data).expect("Ошибка сериализации");
            std::fs::write(&bin_filename, serialized).expect("Не удалось записать бинарный файл");
        }
        println!("Очередь сохранена в {}", bin_filename);

        println!("\n\n\nSTARING WATCH!\n\n\n");
        http_Proc::watch(); 

        let out_extracted = format!("{}_last{}_extracted.json", current_collab.title(), limit);
        let json_output = serde_json::to_string_pretty(&messages).unwrap();
        std::fs::write(out_extracted, json_output).unwrap();

        let _ = std::fs::write(format!("{}_last{}_extracted_FULL.json", current_collab.title(), limit),serde_json::to_string_pretty(&json_value).unwrap(),);
        http_Proc::move_queue_to_queue2();
        // let handle = thread::spawn(http_Proc::process_function());//<===good
        // handle.join();  //process

        http_Proc::process_function("FILENAME.bin".to_string()).await;

}



    #[tokio::test]
    async fn test_pull_messages_prod_Scandinavia2_with_dump() {
        let current_collab = Collab::SCANDINAVIA;
        let title = current_collab.title();
        let id = http_Proc:: get_last_id_for_collab(current_collab, Client::new(), webhook_base_prod().as_str()).await.unwrap();

        println!("\n\n\n\nLAST ID IN {}:: {}\n\n\n", title, id);
        let limit = 1220;

        let json_value:Value = http_Proc::pull_messages_raw(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&current_collab).expect(&format!("{} not found", title)),
        (id + 1) as i64,limit,    ).await.unwrap();
 
        let messages =  http_Proc::extract_messages_from_json(&json_value);
        println!("Извлечено {} сообщений", messages.len());
        for msg in messages.iter() {        println!("{:?}", msg);    }

        {
            let mut queue = http_Proc::QUEUE.lock();
            queue.extend(messages.clone());
        }

        let bin_filename = format!("{}_queue.bin", current_collab.title());
        {
            let queue_data = http_Proc::QUEUE.lock();
            let serialized = bincode::serialize(&*queue_data).expect("Ошибка сериализации");
            std::fs::write(&bin_filename, serialized).expect("Не удалось записать бинарный файл");
        }
        println!("Очередь сохранена в {}", bin_filename);

        println!("\n\n\nSTARING WATCH!\n\n\n");
        http_Proc::watch(); 

        let out_extracted = format!("{}_last{}_extracted.json", current_collab.title(), limit);
        let json_output = serde_json::to_string_pretty(&messages).unwrap();
        std::fs::write(out_extracted, json_output).unwrap();

        let _ = std::fs::write(format!("{}_last{}_extracted_FULL.json", current_collab.title(), limit),serde_json::to_string_pretty(&json_value).unwrap(),);
        http_Proc::move_queue_to_queue2();
        // let handle = thread::spawn(http_Proc::process_function());//<===good
        // handle.join();  //process

        http_Proc::process_function("FILENAME.bin".to_string()).await;

}



    const TEST_DUMP_FILE: &str = "3test.bin";

    use http_Proc::consumer_loop;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};   
    #[tokio::test]
    async fn test_pull_messages_test_OKLAND_with_dump() {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let flag_clone = stop_flag.clone();
        let hello_task = tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut interval = tokio::time::interval(Duration::from_secs(1));
                let mut condition =  !flag_clone.load(Ordering::Relaxed);
                while condition {
                    interval.tick().await;
                    println!("привет");
                    condition =  !flag_clone.load(Ordering::Relaxed);   //comment to loop
                }
            });
        });

        let current_collab = Collab::OKLAND;
        let title = current_collab.title();
        let id = http_Proc:: get_last_id_for_collab(current_collab, Client::new(), webhook_base_prod().as_str()).await.unwrap();

        // println!("\n\n\n\nLAST ID IN {}:: {}\n\n\n", title, id);
        let limit: u32 = 1220;

        let json_value:Value = http_Proc::pull_messages_raw(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&current_collab).expect(&format!("{} not found", title)),
        (id + 1) as i64,limit,    ).await.unwrap();
 
        let messages = http_Proc::fn_to_produce_msg_to_collab(current_collab, &http_Proc::QUEUE).await;// http_Proc::extract_messages_from_json(&json_value);
        // println!("Извлечено {} сообщений", messages.len());
        // for msg in messages.iter() {        println!("{:?}", msg);    }

        // {
        //     let mut queue = http_Proc::QUEUE.lock();
        //     queue.extend(messages.clone());
        // }

        let bin_filename = format!("{}_queue.bin", current_collab.title());
        {
            let queue_data = http_Proc::QUEUE.lock();
            let serialized = bincode::serialize(&*queue_data).expect("Ошибка сериализации");
            std::fs::write(&bin_filename, serialized).expect("Не удалось записать бинарный файл");
        }
        println!("Очередь сохранена в {}", bin_filename);

        println!("\n\n\nSTARING WATCH!\n\n\n");
        http_Proc::watch(); 

        let out_extracted = format!("{}_last{}_extracted.json", current_collab.title(), limit);
        let json_output = serde_json::to_string_pretty(&messages).unwrap();
        std::fs::write(out_extracted, json_output).unwrap();

        let _ = std::fs::write(format!("{}_last{}_extracted_FULL.json", current_collab.title(), limit),serde_json::to_string_pretty(&json_value).unwrap(),);
    //    http_Proc::move_queue_to_queue2();
      {  
        let queue_data2 = http_Proc::QUEUE.lock();
        println!("\n\n\nQUEUE SIZE::{}\n\n\n\n", queue_data2.len());

      }  

       {
            let consumer: tokio::task::JoinHandle<()> = tokio::spawn(consumer_loop(TEST_DUMP_FILE));
            while !http_Proc::QUEUE.lock().is_empty() {    tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;}
            consumer.abort(); // прерываем бесконечный цикл
            println!("Очередь обработана, фоновый поток остановлен");
            stop_flag.store(true, Ordering::Relaxed);
            hello_task.await.unwrap(); // дождаться завершения
        }



}



    #[tokio::test]
    async fn test_pull_messages_prod_OKLAND_with_dump_via_prod_queue() {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let flag_clone = stop_flag.clone();
        let hello_task = tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut interval = tokio::time::interval(Duration::from_secs(1));
                let mut condition =  !flag_clone.load(Ordering::Relaxed);
                while condition {
                    interval.tick().await;
                    println!("привет");
                    condition =  !flag_clone.load(Ordering::Relaxed);   //comment to loop
                }
            });
        });

        let current_collab = Collab::OKLAND;
        let title = current_collab.title();
        let id = http_Proc:: get_last_id_for_collab(current_collab, Client::new(), webhook_base_prod().as_str()).await.unwrap();

        println!("\n\n\n\nLAST ID IN {}:: {}\n\n\n", title, id);
        let limit = 1220;

        let json_value:Value = http_Proc::pull_messages_raw(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&current_collab).expect(&format!("{} not found", title)),
        (id + 1) as i64,limit,    ).await.unwrap();
 
        let messages =  http_Proc::extract_messages_from_json(&json_value);
        println!("Извлечено {} сообщений", messages.len());
        for msg in messages.iter() {        println!("{:?}", msg);    }

        {
            let mut queue = http_Proc::QUEUE_PROC.lock();
            queue.extend(messages.clone());
        }

        let bin_filename = format!("{}_queue.bin", current_collab.title());
        {
            let queue_data = http_Proc::QUEUE_PROC.lock();
            let serialized = bincode::serialize(&*queue_data).expect("Ошибка сериализации");
            std::fs::write(&bin_filename, serialized).expect("Не удалось записать бинарный файл");
        }
        println!("Очередь сохранена в {}", bin_filename);

        println!("\n\n\nSTARING WATCH!\n\n\n");
        http_Proc::watch(); 

        let out_extracted = format!("{}_last{}_extracted.json", current_collab.title(), limit);
        let json_output = serde_json::to_string_pretty(&messages).unwrap();
        std::fs::write(out_extracted, json_output).unwrap();

        let _ = std::fs::write(format!("{}_last{}_extracted_FULL.json", current_collab.title(), limit),serde_json::to_string_pretty(&json_value).unwrap(),);
    //    http_Proc::move_queue_to_queue2();
      {  
        let queue_data2 = http_Proc::QUEUE_PROC.lock();
        println!("\n\n\nQUEUE SIZE::{}\n\n\n\n", queue_data2.len());
      }  

       {
            let consumer: tokio::task::JoinHandle<()> = tokio::spawn(consumer_loop_proc(TEST_DUMP_FILE));
            while !http_Proc::QUEUE_PROC.lock().is_empty() {    tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;}
            consumer.abort(); // прерываем бесконечный цикл
            println!("Очередь обработана, фоновый поток остановлен");
            stop_flag.store(true, Ordering::Relaxed);
            hello_task.await.unwrap(); // дождаться завершения
        }
}

    #[tokio::test]
    async fn test_1_msg(){
    let extracted: ExtractedMessage = ExtractedMessage { author_name: "Артур Сераждинов".to_string(), 
        text: "Согласовано".to_string(), 
        uuid: Some("aa2fe320-f7f1-4c3d-9028-75848448ac6d".to_string()), 
        id: 123820, 
        chat_id: 6986 , 
        attaches: Vec::new()};
    let stop_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = stop_flag.clone();    
    let msgs = vec![extracted];
    {
            let mut queue = http_Proc::QUEUE_PROC.lock();
            queue.extend(msgs.clone());
    }


    {
        let consumer: tokio::task::JoinHandle<()> = tokio::spawn(consumer_loop_proc(TEST_DUMP_FILE));
        while !http_Proc::QUEUE_PROC.lock().is_empty() {    tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;}
        consumer.abort(); // прерываем бесконечный цикл
        println!("Очередь обработана, фоновый поток остановлен");
        stop_flag.store(true, Ordering::Relaxed);
        //hello_task.await.unwrap(); // дождаться завершения
    }
}

    const TEST_DUMP_FILE_TEMP: &str = "77test.bin";

    #[tokio::test]
    async fn test_1_msg_test(){
    let extracted: ExtractedMessage = ExtractedMessage { author_name: "Артур Сераждинов".to_string(), 
        text: "Согласовано".to_string(), 
        uuid: Some("aa2fe320-f7f1-4c3d-9028-75848448ac6d".to_string()), 
        id: 123820, 
        chat_id: 6986,
        attaches: Vec::new() };
    let stop_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = stop_flag.clone();    
    let msgs = vec![extracted];
    {
            let mut queue = http_Proc::QUEUE_PROC.lock();
            queue.extend(msgs.clone());
    }
    {
        let consumer: tokio::task::JoinHandle<()> = tokio::spawn(consumer_loop_proc(TEST_DUMP_FILE_TEMP));
        while !http_Proc::QUEUE_PROC.lock().is_empty() {    tokio::time::sleep(tokio::time::Duration::from_millis(30000)).await;}
        consumer.abort(); // прерываем бесконечный цикл
        println!("Очередь обработана, фоновый поток остановлен");
        stop_flag.store(true, Ordering::Relaxed);
        //hello_task.await.unwrap(); // дождаться завершения
    }
}

//                        cargo test test_embed_queue_test -- --nocapture --test-threads=1

#[tokio::test]
async fn test_embed_queue_test() {
    let current_collab = Collab::OKLAND;
    let stop_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = stop_flag.clone();

    // Генератор сообщений (бесконечный цикл)
    let msg_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        while !flag_clone.load(Ordering::Relaxed) {
            interval.tick().await;
            println!("QUEUE Produce messages run!");
            let msgs = http_Proc::fn_to_produce_msg_to_collab(current_collab, &http_Proc::QUEUE).await;
            {
                let mut queue = http_Proc::QUEUE_PROC.lock(); // .await не нужен – мьютекс синхронный
                queue.extend(msgs);
            }
        }
    });

    // Потребитель (бесконечный цикл) – предполагается, что consumer_loop_proc работает постоянно
    let consumer_task = tokio::spawn(async move {
        consumer_loop_proc(TEST_DUMP_FILE).await;
    });

    // Ждём сигнала завершения (Ctrl+C)
    tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    println!("Shutting down...");
    stop_flag.store(true, Ordering::Relaxed);

    // Прерываем задачи (они могут завершиться сами)
    msg_task.abort();
    consumer_task.abort();
    // Даём задачам время на завершение (опционально)
    tokio::time::sleep(Duration::from_millis(100)).await;
}

    // #[tokio::test]
    // async fn test_prod_emu


     }


//
//
//
// <div class="bx-im-message-base__container"><div class="bx-im-message-base__content"><div class="bx-im-message-base__body"><div class="bx-im-message-default__container"><div class="bx-im-message-author-title__container --clickable"><div class="bx-im-chat-title__scope bx-im-chat-title__container"><span class="bx-im-chat-title__content"><!----><span class="bx-im-chat-title__text" title="Сергей Музданбаев" style="color: rgb(88, 204, 71);">Сергей Музданбаев</span><!----><!----><!----></span></div></div><div class="bx-im-message-default-content__container bx-im-message-default-content__scope"><div class="bx-im-message-quote --reply --collapsed --clickable" data-context="chat6986/117754"><div class="bx-im-message-quote__wrap"><div class="bx-im-message-quote__name"><div class="bx-im-message-quote__name-text">Дмитрий Бердников</div></div><div class="bx-im-message-quote__text">Прошу согласовать на 24.04.26  к 13:00 манипулятор для перевозки уголков и перемычек с Куйбышевой 86 на Рыбацкую <br><br>Конт. тел. 89170911410 Дмитрий</div><!----></div></div><div class="bx-im-message-default-content__text">Согласовано</div><!----><div class="bx-im-message-default-content__bottom-panel"><!----><div class="bx-im-message-default-content__status-container"><div class="bx-im-message-status__container"><!----><div class="bx-im-message-status__date">14:06</div><!----></div></div></div></div></div><!----><div class="bx-im-reaction-selector__container"><div class="bx-im-reaction-selector__selector"><div class="bx-im-reaction-selector__icon"></div></div></div></div><div class="bx-im-message-context-menu__container bx-im-message-context-menu__scope"><button title="Кликните для открытия меню действий или удерживайте CTRL для цитирования сообщения" class="bx-im-message-context-menu__button"></button></div></div><!----></div>
//
//
//
//
//<div class="bx-im-message-base__body"><div class="bx-im-message-default__container"><div class="bx-im-message-author-title__container --clickable"><div class="bx-im-chat-title__scope bx-im-chat-title__container"><span class="bx-im-chat-title__content"><!----><span class="bx-im-chat-title__text" title="Сергей Музданбаев" style="color: rgb(88, 204, 71);">Сергей Музданбаев</span><!----><!----><!----></span></div></div><div class="bx-im-message-default-content__container bx-im-message-default-content__scope"><div class="bx-im-message-quote --reply --collapsed --clickable" data-context="chat6986/119486"><div class="bx-im-message-quote__wrap"><div class="bx-im-message-quote__name"><div class="bx-im-message-quote__name-text">Артур Сераждинов</div></div><div class="bx-im-message-quote__text">Прошу согласовать материал <br>1.гофра серая 20-ый диаметр-5000м</div><!----></div></div><div class="bx-im-message-default-content__text">Согласовано</div><!----><div class="bx-im-message-default-content__bottom-panel"><!----><div class="bx-im-message-default-content__status-container"><div class="bx-im-message-status__container"><!----><div class="bx-im-message-status__date">11:25</div><!----></div></div></div></div></div><!----><div class="bx-im-reaction-selector__container"><div class="bx-im-reaction-selector__selector"><div class="bx-im-reaction-selector__icon"></div></div></div></div>
// 
// 
// 
// 
// 
// 
// 
// Извлечено 50 сообщений
// ExtractedMessage { author_name: "Сергей Музданбаев", text: "Прошу поставить:\n1. бетономешалку код в Добрострой 3016584.\n2. УШМ-230 Ресанта код 3003088.", uuid: Some("00d99f7b-2fcf-4932-96ad-ab7cd4964dc8"), id: 180296, chat_id: 6986 }
// regerenced to phpto ExtractedMessage { author_name: "Сергей Музданбаев", text: "Согласовано", uuid: Some("71fc41f8-2f53-4612-ba11-3b79d5575f8a"), id: 180274, chat_id: 6986 }
// with photo scana   ExtractedMessage { author_name: "Артур Сераждинов", text: "Прошу согласовать материал", uuid: None, id: 180232, chat_id: 6986 }
