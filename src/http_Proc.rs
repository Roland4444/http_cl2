use axum::extract::State;
use axum::{    Router,    body::Bytes,    http::StatusCode,    response::IntoResponse,    routing::{get, post},};
use futures_util::{SinkExt, StreamExt};
use anyhow::{Context, Result};
use futures::stream::{self,  TryStreamExt}; // в начало файла

use reqwest;  // <-- Добавить это


use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde::{Serialize, Deserialize};
use crate::http_test::{read_lines, read_lines_utf8, update_param_};
use tokio::net::TcpListener;
use common::*;
const WEBHOOK_FILENAME: &str = "webhook";
use once_cell::sync::Lazy;
use crate::thread;
use parking_lot::Mutex as Mutex2;
use crate::HashMap;
use std::sync::RwLock;
use std::fs;

use crate::webhook_base_prod;
use crate::cl_address;
static IS_PENDING: AtomicBool = AtomicBool::new(false);


const SYSTEM_MESSAGE: &str = "0";    
const CREATE_QUEUE: &str = "CREATE_QUEUE";         
const RUN_QUEUE: &str = "RUN_QUEUE";   
const CONTENT_TYPE: &str = "Content-type";
const APPROVED: &str = "СОГЛАСОВАНО";

pub const  FILE_NAME_4_HASHSET: &str = "processed.bin";

pub static PROCESSED_IDS: Lazy<RwLock<HashSet<u32>>> = Lazy::new(|| RwLock::new(HashSet::new()));

pub fn add_processed_id(id: u32){
    let mut set  = PROCESSED_IDS.write().unwrap();
    set.insert(id);
}

pub fn is_processed(id: u32) -> bool {
    let set = PROCESSED_IDS.read().unwrap();
    set.contains(&id)
}

pub fn suspend_to_file(path: &str) -> Result<()>{
    let set = PROCESSED_IDS.read().unwrap();
    let ids: Vec<u32> = set.iter().copied().collect();
    let encoded = bincode::serialize(&ids)?;
    fs::write(path, encoded)?;
    Ok(())
}

pub fn restore_processed_ids_from_file(path: &str) -> Result<()>{
    let data = fs::read(path)?;
    let ids: Vec<u32> = bincode::deserialize(&data)?;
    let mut set = PROCESSED_IDS.write().unwrap();
    *set = ids.into_iter().collect();
    Ok(())
}

pub fn restore_processed_ids(input_file: &str) -> Result<()>{
    let data = fs::read(input_file)?;
    let ids: Vec<u32> = bincode::deserialize(&data)?;
    let mut set = PROCESSED_IDS.write().unwrap();
    *set = ids.into_iter().collect();
    Ok(())
}


pub fn suspend_processed_ids() -> Result<()>{
    let data = fs::read(FILE_NAME_4_HASHSET)?;
    let ids: Vec<u32> = bincode::deserialize(&data)?;
    let mut set = PROCESSED_IDS.write().unwrap();
    *set = ids.into_iter().collect();
    Ok(())
}


#[derive(Debug, Clone, PartialEq)]  pub struct KeyValueMessage {    pub id: i32,    pub key: String,    pub value: String,}

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)] pub struct ConfigProcess {    pub switch_mode: SwitchIDMode,   pub from: u32,    pub to: u32, pub enabled_collabs: Vec<common::Collab>}


macro_rules! hashmap {
    ($($key: expr => $val: expr), *) => {
        {
            let mut map = ::std::collections::HashMap::new();
            $(map.insert($key, $val); )*
            map        }    };
        }


//genned
pub static CHATS_ID: Lazy<HashMap<Collab, &str>> = Lazy::new(|| {
    hashmap!(
        Collab::PAYMENTS => "chat9224",        Collab::OLIVIA => "chat6998",        Collab::BABEFA => "chat6974",        Collab::OKLAND => "chat6986",        Collab::RED => "chat7018",
        Collab::TETRIS => "chat7014",          Collab::SCANDINAVIA => "chat9796",   Collab::KUIB => "chat7210",          Collab::POLZ => "chat7208",          Collab::ZVEZD => "chat7242",
        Collab::SKY => "chat6966",             Collab::OWN => "chat13372"    )});
//genned

pub static CHAT_NUM_ID: Lazy<HashMap<Collab, u64>> = Lazy::new(|| {
    CHATS_ID.iter().map(|(collab, &id_str): (&Collab, &&str)| {
            let num = id_str.strip_prefix("chat").unwrap_or(id_str).parse().expect("INVALID ID");
            (*collab, num)}).collect()});


pub static CHAT_NUM_TO_COLLAB: Lazy<HashMap<u64, Collab>> = Lazy::new(|| {
    CHAT_NUM_ID.iter().map(|(&collab, &id)| (id, collab)).collect()
});

pub fn collab_by_num_id(id: u64) -> Option<Collab> {
    CHAT_NUM_TO_COLLAB.get(&id).copied()
}


pub static CHAT_NUM_ID_PROD: Lazy<RwLock<Vec<u64>>> = Lazy::new(|| RwLock::new(Vec::new()));
//AUTHOR::Александр Минин, TEXT::Будет корректировка в большую сторону., UUID::597650e4-1faa-40da-ab05-350005abcbbb, ID::102072, CHAT_ID::9796
pub fn reinit_CHAT_NUM() {
    let mut target = CHAT_NUM_ID_PROD.write().unwrap();
    target.clear();
    for collab in CONFIG.enabled_collabs.iter() {
        if let Some(&num) = CHAT_NUM_ID.get(collab) {target.push(num);} 
        else {eprintln!("Warning: Collab {:?} not found in CHAT_NUM_ID", collab);}
    }
}

pub fn reinit_CHAT_NUM__(config: ConfigProcess) {
    let mut target = CHAT_NUM_ID_PROD.write().unwrap();
    target.clear();
    for collab in config.enabled_collabs.iter() {
        if let Some(&num) = CHAT_NUM_ID.get(collab) {            target.push(num);        } 
        else {            eprintln!("Warning: Collab {:?} not found in CHAT_NUM_ID", collab);        }
    }
}

// pub fn predicate(input: ExtractedMessage) -> bool {
//     reinit_CHAT_NUM();
//     let ids = CHAT_NUM_ID_PROD.read().unwrap();
//     let filter_via_collabs = ids.contains(&input.chat_id);
//     let mut filter_via_id = true;
//     if (CONFIG.switch_mode == common::
//     filter_via_collabs && filter_via_id
// }
pub static CONFIG: Lazy<ConfigProcess> = Lazy::new(|| ConfigProcess {switch_mode: common::SwitchIDMode::FROM_CURRENT,from: 0,to: 1000000,enabled_collabs: vec![Collab::OKLAND],});

pub fn process_msg(entry: ExtractedMessage)-> bool{   
    let author = entry.author_name;    
    let collab = entry.chat_id;    
    true   
}

pub fn process_message<F, R>(msg: ExtractedMessage, processor: F) -> R
where
    F: Fn(ExtractedMessage) -> R,
{
    processor(msg)
}
//pub static QUEUE: Lazy< std::sync::Mutex<Vec<ExtractedMessage>>> = Lazy::new(||  std::sync::Mutex::new(Vec::new()));
pub static QUEUE: Mutex2<Vec<ExtractedMessage>> = Mutex2::new(Vec::new());
pub static QUEUE_PROC: Mutex2<Vec<ExtractedMessage>> = Mutex2::new(Vec::new());
//pub static QUEUE__: Lazy<Mutex<Vec<ExtractedMessage>>> = Lazy::new(|| Mutex::new(Vec::new()));
//   add::             QUEUE.lock().unwrap().push(msg);
//   remove::             QUEUE.lock().unwrap().remove(index);
//  iteration
//  let mut queue = QUEUE.lock().unwrap();
//  for item in queue.iter() { ... }
pub fn copy_queue_to_queue2() {    
    let queue = QUEUE.lock();    
    let mut queue2 = QUEUE_PROC.lock();    
    queue2.extend(queue.clone()); 
}

pub fn move_queue_to_queue2() -> usize {   
     let mut queue = QUEUE.lock();    
     let mut queue2 = QUEUE_PROC.lock();    
     let len = queue.len();    
     queue2.extend(queue.drain(..));    len
}


pub fn adding_to_Pack(msg: ExtractedMessage) -> bool {    QUEUE.lock().push(msg);    true}

// Синхронный просмотр очереди
pub fn watch() {
    let queue = QUEUE.lock();
    for item in queue.iter() {println!("{}", item.to_string());}
}

pub async fn get_full_info_via_id_and_chat(chat_name: String, message_id: u64) -> Result<QuoteInfo> {
    let (mut ws_stream, _) = connect_async(URL_WS_CONNECT)        .await        .context("Не удалось подключиться к WebSocket")?;

    let req = json!({        "collab": chat_name,        "message_id": message_id,        "type__": "ExtractFull"    });
    let req_bytes = serde_json::to_vec(&req)?;
    ws_stream.send(Message::Binary(req_bytes.into())).await?;

    if let Some(Ok(Message::Text(resp_text))) = ws_stream.next().await {
        let resp: ExtractResp = serde_json::from_str(&resp_text)?;  // сначала разбираем обёртку
        if resp.success {
            if let Some(json_str) = resp.quoted_text {
                let quote_info: QuoteInfo = serde_json::from_str(&json_str)?; // потом внутренность
                println!("Полная информация: {:?}", quote_info);
                return Ok(quote_info);
            } else {
                anyhow::bail!("Ответ не содержит данных");
            }
        } else {
            anyhow::bail!("Ошибка сервера: {}", resp.error.unwrap_or_default());
        }
    }
    anyhow::bail!("Не получен ответ от сервера");
}


// async fn process_atom_queue() -> Result<()>{
//     let mut queue: parking_lot::lock_api::MutexGuard<'_, parking_lot::RawMutex, Vec<ExtractedMessage>> = QUEUE.lock();
//     let elem = queue.first();
//     match elem{
//         Some(el__) => {
//             let text = el__.text.as_str();
//             println!("text:{}",text);
// ////////   GET QUOTES  //////////////////////////////////////////////////////////////////////////////////
// //  ==================>>>>>>>>>>>>>>>>>>>>      WEBSOCKET    AGENT RUST
// // example agent::main.rs::test_websocket_extract_full_info() {            //                         cargo test test_websocket_extract_full_info -- --nocapture
//             let resp = get_full_info_via_id_and_chat(collab_by_num_id(el__.chat_id.into()).unwrap().title().to_string(), el__.id.into()).await;
//             match resp {
//                     Ok(qi) => {
//                         println!("ID: {}", qi.message_id);
//                         println!("Автор цитаты: {}", qi.quoted_author);
//                         println!("Текст цитаты: {}", qi.quoted_text);
//                         println!("Текст ответа: {:?}", qi.reply_text);
//                     }
//                     Err(e) => eprintln!("Ошибка: {}", e),
//             }
//         }
//         None => {  eprintln!("Ошибка:") }

// ///            ///////////////////////////////////////// /////////////////////////////////////////////////
// ///   SEND TO CL QUEUE 
// ///   =====>>>>>>>>>>>>>>> GET REQUEST
// //////////////////////////////////////

//         }
//         Ok(())
//     }        

pub async fn consumer_loop(name_4_idfile: &str) {
    restore_processed_ids(name_4_idfile);

    loop {
        // Проверяем наличие сообщений
        let has_message = {
            let queue = QUEUE.lock();
            !queue.is_empty()
        };
        if has_message {
            if let Err(e) = process_atom_queue(name_4_idfile).await {
                eprintln!("Ошибка при обработке сообщения: {}", e);
            }
        } else {
            // Очередь пуста – ждём появления новых сообщений
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }
}


pub async fn fn_to_produce_msg_to_collab(current_collab: Collab,    queue: &Mutex2<Vec<ExtractedMessage>>,) -> Vec<ExtractedMessage> {
    let title = current_collab.title();
    let id = get_last_id_for_collab(current_collab, Client::new(), webhook_base_prod().as_str()).await.unwrap();
    println!("\n\n\n\nLAST ID IN {}:: {}\n\n\n", title, id);
    let limit = 1220;
    let json_value: Value = pull_messages_raw(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&current_collab).expect(&format!("{} not found", title)),
        (id + 1) as i64,limit,).await.unwrap();
    let messages = extract_messages_from_json(&json_value);
    println!("Извлечено {} сообщений", messages.len());
    for msg in messages.iter() {        println!("{:?}", msg);    }
    {
        let mut queue_guard = queue.lock();  
        queue_guard.extend(messages.clone());
    }
    messages
}


pub async fn consumer_loop_proc(name_4_idfile: &str) {
    let _ = restore_processed_ids(name_4_idfile);
    loop {

        println!("LOOP! consumer-prod\n");
        let has_message = {
            let queue = QUEUE_PROC.lock();
            !queue.is_empty()
        };
        if has_message {
            if let Err(e) = process_atom_queue_proc(name_4_idfile).await {
                eprintln!("Ошибка при обработке сообщения: {}", e);
            }
        } else {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }
}







pub fn check_target(input: &str) -> bool {    input.to_uppercase()==APPROVED || input.to_uppercase().contains(APPROVED)  }


pub fn check_body(input: &str, resctrictors: Vec<String>) -> bool {  

    let mut result = true;
    for i in  resctrictors {
        result = result && !(input.to_uppercase().contains(&i));
    }
    result
  }




pub async fn process_atom_queue_proc(name_4_idfile: &str) -> Result<()> {

    println!("PROCESS FROM PROD QUEUE\n");
    // Извлекаем элемент из очереди (блокировка удерживается короткое время)
    let msg = {
        let mut queue = QUEUE_PROC.lock();
        let first = queue.first().cloned();
        if let Some(msg) = first {
            queue.remove(0);
            msg
        } else {
            return Ok(()); // очередь пуста
        }
    };

    // Копируем строку, чтобы избежать проблем с заимствованием
    let initial_author = msg.author_name.clone();

    println!("text: {}", msg.text);
    if !check_target(&msg.text) {        println!("DROP NOT TARGET!\n\n");         return Ok(())}     //drop not target message    with filter  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 80.74s
 // without filter    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 220.78s
    if is_processed(msg.id){println!("ID msg {} already processed, skipped...", msg.id);  return Ok(())} 

    let skipped_words: Vec<String> = vec! ["ШАХМАН".to_string(), "ГАЗЕЛЬ".to_string(), "МАНИПУЛЯТОР".to_string(),];

    println!("100!\n\n");
    // Получаем Collab по chat_id
    let collab = collab_by_num_id(msg.chat_id.into())
        .ok_or_else(|| anyhow::anyhow!("Collab not found for chat_id {}", msg.chat_id))?;
    let resp = get_full_info_via_id_and_chat(collab.title().to_string(), msg.id.into()).await;
    let acc_id = 7;
    match resp {
        Ok(qi) => {
            println!("ID: {}", qi.message_id);
            println!("Автор цитаты: {}", qi.quoted_author);
            println!("Текст цитаты: {}", qi.quoted_text);
            println!("Текст ответа: {:?}", qi.reply_text);
            println!("collab: {:?}", collab.title());

            // Безопасно извлекаем uuid (если None, передаём пустую строку)
            let uuid_str = msg.uuid.unwrap_or_default();
            if !check_body(&qi.quoted_text, skipped_words) {
                println!("DROP REQUEST TRANSPORT :: {:?}",  qi.quoted_text);
                add_processed_id(msg.id);
                suspend_to_file(name_4_idfile);
                return Ok(())
            }
            // Отправляем данные в CL Queue
            let resp_______ = send_to_cl_queue_proc(
                &qi.quoted_text,
                &initial_author,
                &qi.quoted_author,
                &uuid_str,  
                &collab.title(),
                acc_id
            ).await;
            println!("resp::{}\n\n", resp_______.unwrap());
            add_processed_id(msg.id);
            suspend_to_file(name_4_idfile);

        }
        Err(e) => eprintln!("Ошибка: {}", e),
    }

    Ok(())
}


pub async fn process_atom_queue(name_4_idfile: &str) -> Result<()> {

    println!("PROCESS FROM QUEUE\n");
    // Извлекаем элемент из очереди (блокировка удерживается короткое время)
    let msg = {
        let mut queue = QUEUE.lock();
        let first = queue.first().cloned();
        if let Some(msg) = first {
            queue.remove(0);
            msg
        } else {
            return Ok(()); // очередь пуста
        }
    };

    // Копируем строку, чтобы избежать проблем с заимствованием
    let initial_author = msg.author_name.clone();

    println!("text: {}", msg.text);
    if !check_target(&msg.text) {return Ok(())}     //drop not target message    with filter  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 80.74s
 // without filter    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 220.78s
    if is_processed(msg.id){println!("ID msg {} already processed, skipped...", msg.id);  return Ok(())} 
    // Получаем Collab по chat_id
    let collab = collab_by_num_id(msg.chat_id.into())
        .ok_or_else(|| anyhow::anyhow!("Collab not found for chat_id {}", msg.chat_id))?;

    let resp = get_full_info_via_id_and_chat(collab.title().to_string(), msg.id.into()).await;
    let acc_id = 7;
    match resp {
        Ok(qi) => {
            println!("ID: {}", qi.message_id);
            println!("Автор цитаты: {}", qi.quoted_author);
            println!("Текст цитаты: {}", qi.quoted_text);
            println!("Текст ответа: {:?}", qi.reply_text);
            println!("collab: {:?}", collab.title());

            // Безопасно извлекаем uuid (если None, передаём пустую строку)
            let uuid_str = msg.uuid.unwrap_or_default();

            // Отправляем данные в CL Queue
            let resp_______ = send_to_cl_queue(
                &qi.quoted_text,
                &initial_author,
                &qi.quoted_author,
                &uuid_str,  
                &collab.title(),
                acc_id
            ).await;
            println!("resp::{}\n\n", resp_______.unwrap());
            add_processed_id(msg.id);
            suspend_to_file(name_4_idfile);
        }
        Err(e) => eprintln!("Ошибка: {}", e),
    }

    Ok(())
}



const LOG: &str = "reqpr";
async fn send_to_cl_queue(quotes: &str, author: &str, quotes_author: &str, uuid: &str,  collab: &str, source_acc: u32 )-> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:11111/reqpr")   //cl_address().replace("decode", PROC))  //cl_address().replace("decode", LOG))
        .form(&[("input", quotes),                 ("author", author), 
                ("quotes_author", quotes_author),  ("uuid", uuid),
                ("collab", collab),                ("source_acc", source_acc.to_string().as_str())                                 ])
                
        .send()
        .await?;
    let body = response.text().await?;
    Ok(body)
}


const PROC: &str = "reqprc";
async fn send_to_cl_queue_proc(quotes: &str, author: &str, quotes_author: &str, uuid: &str,  collab: &str, source_acc: u32 )-> Result<String, Box<dyn Error>> {
    println!("sendind to CL!\n");
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:11111/reqprc")   //cl_address().replace("decode", PROC))
        .form(&[("input", quotes),                 ("author", author), 
                ("quotes_author", quotes_author),  ("uuid", uuid),
                ("collab", collab),                ("source_acc", source_acc.to_string().as_str())                                 ])
                
        .send()
        .await?;
    let body = response.text().await?;
    Ok(body)
}



pub async fn __func2(_counter2: u64) {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("PRODUCER: no new messages, sleeping");
    }
}

// pub async fn __func2(mut counter2: u64) {
//     // Даём фору, чтобы другие потоки успели инициализироваться
//     tokio::time::sleep(Duration::from_millis(500)).await;
//     println!("FUNC2 started – will move items from QUEUE2 to QUEUE");

//     loop {
//         counter2 += 1;
//         // Забираем всё, что есть в QUEUE2
//         let items_to_move = {
//             let mut queue2 = QUEUE2.lock();
//             if queue2.is_empty() {
//                 None
//             } else {
//                 let items: Vec<_> = queue2.drain(..).collect();
//                 Some(items)
//             }
//         };

//         if let Some(mut items) = items_to_move {
//             let mut queue = QUEUE.lock();
//             queue.append(&mut items);
//             println!("FUNC2: moved {} items to QUEUE, new QUEUE size: {}", items.len(), queue.len());
//         } else {
//             // QUEUE2 пуста – ждём и ничего не делаем
//             if counter2 % 5 == 0 {
//                 println!("FUNC2: QUEUE2 is empty, waiting... (counter={})", counter2);
//             }
//         }

//         tokio::time::sleep(Duration::from_secs(2)).await;
//     }
// }

/// Функция-потребитель: обрабатывает сообщения из QUEUE
// pub async fn __func1(mut counter1: u64) {
//     // Начальный размер очереди
//     {
//         let queue = QUEUE.lock();
//         println!("INITIAL QUEUE SIZE: {}", queue.len());
//     }

//     loop {
//         counter1 += 1;
//         println!("THREAD1: processing iteration {}", counter1);
//         // Обработка одного элемента (если есть)
//         process_atom_queue().await;

//         // Проверяем, опустела ли очередь
//         let is_empty = {
//             let queue = QUEUE.lock();
//             queue.is_empty()
//         };
//         if is_empty {
//             println!("QUEUE is empty, exiting __func1");
//             break;
//         }

//         tokio::time::sleep(Duration::from_secs(1)).await;
//     }
// }

pub async fn __func1(mut counter1: u64, filename: String) {
    println!("CONSUMER started (QUEUE will be processed until empty)");
    loop {
        // Проверяем, есть ли элементы в QUEUE
        let is_empty = {
            let queue = QUEUE.lock();
            queue.is_empty()
        };
        if is_empty {
            println!("QUEUE is empty, consumer finished");
            break;
        }

        counter1 += 1;
        println!("Processing iteration {}", counter1);
        process_atom_queue(filename.as_str()).await; // обрабатывает один элемент

        // Небольшая пауза, чтобы не нагружать CPU
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}






/// Асинхронная точка входа для двух задач
pub async fn process_function(filename: String) {
    println!("Starting async tasks: CONSUMER (__func1) and PRODUCER (__func2)");
    let borrowed =     filename.to_string();
    let task1 = tokio::spawn(__func1(0, filename));
    let task2 = tokio::spawn(__func2(0));
    let _ = tokio::join!(task1, task2);
}


fn decode_key_value_message(buf: &[u8]) -> Result<KeyValueMessage, String> {
    let mut bytes = buf;
    let mut id = None;
    let mut key = None;
    let mut value = None;
    while !bytes.is_empty() {
        let (key_val, rest) =read_varint(bytes).map_err(|e| format!("Ошибка чтения ключа: {}", e))?;
        bytes = rest;
        let tag = key_val >> 3;
        let wire_type = key_val & 0x07;
        match (tag, wire_type) {
            (1, 0) => {
                let (val, rest) =read_varint(bytes).map_err(|e| format!("Ошибка чтения id: {}", e))?;
                id = Some(val as i32);
                bytes = rest;
            }
            (2, 2) => {
                let (len, rest) =read_varint(bytes).map_err(|e| format!("Ошибка чтения длины key: {}", e))?;
                if rest.len() < len as usize {return Err("Недостаточно байт для key".into());}
                let str_bytes = &rest[..len as usize];
                let s = String::from_utf8(str_bytes.to_vec()).map_err(|e| format!("key не валидный UTF-8: {}", e))?;
                key = Some(s);
                bytes = &rest[len as usize..];
            }
            (3, 2) => {
                let (len, rest) =read_varint(bytes).map_err(|e| format!("Ошибка чтения длины value: {}", e))?;
                if rest.len() < len as usize {return Err("Недостаточно байт для value".into());}
                let str_bytes = &rest[..len as usize];
                let s = String::from_utf8(str_bytes.to_vec()).map_err(|e| format!("value не валидный UTF-8: {}", e))?;
                value = Some(s);
                bytes = &rest[len as usize..];
            }
            _ => match wire_type {
                0 => {
                    let (_, rest) =read_varint(bytes).map_err(|e| format!("Ошибка пропуска varint: {}", e))?;
                    bytes = rest;
                }
                2 => {
                    let (len, rest) = read_varint(bytes).map_err(|e| format!("Ошибка чтения длины для пропуска: {}", e))?;
                    if rest.len() < len as usize {return Err("Недостаточно байт для пропуска поля".into());}
                    bytes = &rest[len as usize..];
                }
                _ => {return Err(format!("Неподдерживаемый wire type {} для пропуска",wire_type));}
            },
        }
    }

    Ok(KeyValueMessage {id: id.ok_or("Отсутствует поле id")?,key: key.ok_or("Отсутствует поле key")?,value: value.ok_or("Отсутствует поле value")?,    })
}

fn read_varint(buf: &[u8]) -> Result<(u64, &[u8]), &'static str> {
    let mut result = 0u64;
    let mut shift = 0;
    let mut pos = 0;
    while pos < buf.len() {
        let byte = buf[pos];
        result |= ((byte & 0x7F) as u64) << shift;
        pos += 1;
        if byte & 0x80 == 0 {return Ok((result, &buf[pos..]));}
        shift += 7;
        if shift > 63 {return Err("varint слишком длинный");}
    }
    Err("неожиданный конец varint")
}

async fn hello_handler() -> &'static str {    "hello world"}

pub fn get_webhook() -> String {    get_webhook_(WEBHOOK_FILENAME)}

pub fn get_webhook_(filename: &str) -> String {
    let vec = read_lines(filename);
    let elem = vec[0].clone();
    elem
}

pub async fn update_param_2(client_reqwest: Client,params: &[(&str, &str)],) -> Result<String, Box<dyn Error>> {
    let url = get_webhook() + "user.update";
    println!("Resulted url: {}", url);
    let response = client_reqwest.post(url).form(params).send().await?;
    println!("Status: {}", response.status());
    println!("Headers: {:#?}", response.headers());
    let body = response.text().await?;
    Ok(body)
}



fn process_message2(msg: ExtractedMessage) -> u32 {
    if !msg.text.to_uppercase().contains(APPROVED) {
        println!("SKIPPED");
        msg.id
    } else {
        println!("PROCESSED!");
        msg.id
    }
}
const URL_WS_CONNECT: &str = "ws://127.0.0.1:3000/proc";


pub async fn get_text_via_chat_id_and_id(chat_name: String, message_id: u64) -> Result<String> {
    let (mut ws_stream, _) = connect_async(URL_WS_CONNECT).await.context("Не удалось подключиться к WebSocket")?;
    let request = json!({        "collab": chat_name,        "message_id": message_id, "type__":  "ExtractFull" });
    let request_bytes = serde_json::to_vec(&request)?;
    ws_stream.send(Message::Binary(request_bytes.into())).await?;

    if let Some(Ok(Message::Text(resp_text))) = ws_stream.next().await {
        let resp: ExtractResp = serde_json::from_str(&resp_text)?;
        if resp.success {
            if let Some(text) = resp.quoted_text {                println!("EXTRACTED: {}", text);                return Ok(text);            } 
            else {                anyhow::bail!("Ответ не содержит текста");            }}
        else {            anyhow::bail!("Ошибка сервера: {}", resp.error.unwrap_or_default());        }}
    anyhow::bail!("Не получен ответ от сервера");
}

pub async fn get_text_via_chat_id_and_id2(chat_name: String, message_id: u64) -> Result<String> {
    let (mut ws_stream, _) = connect_async("ws://127.0.0.1:3000/proc").await.context("Не удалось подключиться к WebSocket")?;
    let request = json!({"collab": chat_name,"message_id": message_id});
    let request_bytes = serde_json::to_vec(&request)?;
    ws_stream.send(Message::Binary(request_bytes.into())).await?;
    if let Some(Ok(Message::Text(resp_text))) = ws_stream.next().await {
        let resp: ExtractResp = serde_json::from_str(&resp_text)?;
        if resp.success {return Ok(resp.quoted_text.unwrap_or_default());} 
        else {            anyhow::bail!("Ошибка сервера: {}", resp.error.unwrap_or_default());        }}
    anyhow::bail!("Не получен ответ от сервера");
}

pub fn extract_messages_from_json(value: &Value) -> Vec<ExtractedMessage> {
    let mut user_names = HashMap::new();
    if let Some(users) = value["result"]["users"].as_array() {        for user in users {if let (Some(id), Some(name)) = (user["id"].as_u64(), user["name"].as_str()) {user_names.insert(id, name.to_string());}}}
    let mut result = Vec::new();
    if let Some(messages) = value["result"]["messages"].as_array() {
        for msg in messages {
            let author_id = msg["author_id"].as_u64().unwrap_or(0);
            if author_id == 0 {                continue;            }
            let id = msg["id"].as_u64().unwrap_or(0) as u32;
            let chat_id = msg["chat_id"].as_u64().unwrap_or(0) as u32;
            let author_name = user_names.get(&author_id).cloned().unwrap_or_else(|| format!("unknown_{}", author_id));
            let text = msg["text"].as_str().unwrap_or("").to_string();
            let uuid = msg["uuid"].as_str().map(|s| s.to_string());
            
            let attaches = msg["params"]
                .get("FILE_ID")                      // берём поле "FILE_ID" внутри "params"
                .and_then(|v| v.as_array())         // пытаемся получить массив
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_u64().map(|num| num as u32))
                        .collect::<Vec<u32>>()
                })
                .unwrap_or_else(Vec::new);          // если ничего нет – пустой вектор

            result.push(ExtractedMessage {
                author_name,
                text,
                uuid,
                id,
                chat_id,
                attaches,
            });
        }
            
            
        //    result.push(ExtractedMessage {author_name,text,uuid,id,chat_id,});
        
    }
    result
}


pub async fn get_last_id_for_collab(    collab: Collab,    client: Client,    webhook_url: &str,) -> anyhow::Result<u64> {
    let json = fetch_recent_list_raw(client, webhook_url, json!({})).await.context("Ошибка получения списка чатов")?;
    let items = json["result"]["items"].as_array().context("Нет поля result.items в JSON")?;
    let target_title = collab.title();
    for item in items {
        let item_type = item["type"].as_str().unwrap_or("");
        let title = item["title"].as_str().unwrap_or("");
        if item_type == "chat" && title == target_title {
            let last_id = item["last_id"].as_u64().context("Поле last_id отсутствует или не число")?;
            return Ok(last_id);
        }
    }
    anyhow::bail!("Чат с названием '{}' не найден", target_title);
}


fn approved_filter(msg: &ExtractedMessage) -> bool {    msg.text.to_uppercase().contains(APPROVED)}

pub async fn __pull_messages_prod_throw_collab_and_filter(current_collab: Collab, config: ConfigProcess) -> Vec<ExtractedMessage> {
    let title = current_collab.title();
    let id = get_last_id_for_collab(current_collab, Client::new(), webhook_base_prod().as_str()).await.unwrap();

    println!("\n\n\n\nLAST ID IN {}:: {}\n\n\n", title, id);
    let mut  limit = 1;
    if config.switch_mode == common::SwitchIDMode::FROM_TO {
        let diff = (id as i64) - (config.from as i64);
        if diff < 0 {  return  Vec::new()}
        else {limit = diff as u32;}
    }
    let json_value:Value = pull_messages_raw(Client::new(),webhook_base_prod().as_str(),CHATS_ID.get(&current_collab).expect(&format!("{} not found", title)),
    (id + 1) as i64,limit as u32,).await.unwrap(); 
    let messages = extract_messages_from_json(&json_value);
    let filtered = messages.into_iter().filter(approved_filter);
    if config.switch_mode == common::SwitchIDMode::FROM_TO {filtered.into_iter().filter(|a| a.id >= config.from && a.id <= config.to).collect()    } 
    else {        filtered.collect()     }
}

pub async fn __pull_messages_prod_throw_collabs_and_filter__(config: ConfigProcess) -> Vec<ExtractedMessage>{
    let mut result: Vec<ExtractedMessage> = Vec::new();
    for item in &config.enabled_collabs{
        let res = __pull_messages_prod_throw_collab_and_filter(*item, config.clone()).await;
        result.extend(res);
    }
    result.sort_by_key(|msg| msg.id);
    result
}

async fn processmsg(msg: KeyValueMessage,vec: &mut Vec<KeyValueMessage>,) -> Result<(), Box<dyn Error>> {
    println!("✅ Получено сообщение: {:?}", msg);
    let client_reqwest = Client::new();
    let id_str = msg.id.to_string();
    let key_str = msg.key.as_str();
    let value_str = msg.value.as_str();
    let item = [("ID", id_str.as_str()), (key_str, value_str)];
    if id_str == SYSTEM_MESSAGE.to_string() {
        println!("SYSTEM MESSAGE: {}", id_str);
        println!("VALUE MESSAGE: {}", key_str);
        if key_str == CREATE_QUEUE {IS_PENDING.store(true, Ordering::SeqCst);}
        if key_str == RUN_QUEUE {
            if vec.len() == 0 {
                println!("\n\n\n\nאפס\n\n\n\n!!");
                return Ok(());
            }
            let threads = match value_str.parse::<usize>() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("Неверное число потоков: {}", value_str);
                    return Ok(());
                }
            };

            let items: Vec<(i32, String, String)> = vec.iter().map(|kv| (kv.id, kv.key.clone(), kv.value.clone())).collect();

            let mut stream = stream::iter(items)
                .map(|(id, key, value)| {
                    let client = client_reqwest.clone();
                    async move {
                        let id_str = id.to_string();
                        let params = [("ID", id_str.as_str()), (key.as_str(), value.as_str())];
                        update_param_2(client, &params).await
                    }
                })
                .buffer_unordered(threads);

            while let Some(result) = stream.next().await {
                match result {
                    Ok(body) => println!("Response body: {}", body),
                    Err(e) => eprintln!("Ошибка запроса: {}", e),
                }
            }
            IS_PENDING.store(false, Ordering::SeqCst);
        }

        return Ok(());
    }
    if !IS_PENDING.load(Ordering::SeqCst) {
        let result = update_param_2(client_reqwest.clone(), &item).await?;
        println!("Результат обновления: {}", result);
    }
    if IS_PENDING.load(Ordering::SeqCst) {
        vec.push(msg.clone());
        let result = 5;
        println!("WAITING RUN.......: {}", result);
    }
    Ok(())
}

type SharedState = Arc<Mutex<Vec<KeyValueMessage>>>;

async fn post_handler(State(state): State<SharedState>, body: Bytes) -> impl IntoResponse {
    match decode_key_value_message(&body) {
        Ok(msg) => {
            println!("✅ Получено сообщение: {:?}", msg);
            let mut vec = state.lock().await; // теперь .await работает
            processmsg(msg, &mut *vec).await;
            (StatusCode::OK, "OK")
        }
        Err(e) => {
            eprintln!("❌ Ошибка декодирования: {}", e);
            (StatusCode::BAD_REQUEST, "Invalid protobuf")
        }
    }
}

fn json_to_file(filename: &str, value: Value) -> Result<(), Box<dyn std::error::Error>> {
    let mut file: File = File::create(filename)?;
    file.write_all(serde_json::to_string_pretty(&value)?.as_bytes())?;
    Ok(())
}

pub async fn pull_messages_raw(    client: Client,    base_webhook_url: &str,    dialog_id: &str,    last_id: i64,    limit: u32,) -> Result<Value, Box<dyn std::error::Error>> {
    let suffix = "/im.dialog.messages.get";
    let req_info = json!({"DIALOG_ID": dialog_id,"LAST_ID": last_id,"LIMIT": limit});
    let resp = client.post(format!("{}{}", base_webhook_url, suffix)).header(CONTENT_TYPE, "application/json").json(&req_info).send().await?;
    let json_value = resp.json().await?;
    Ok(json_value)
}

pub async fn pull_messages(    client: Client,    base_webhook_url: &str,    dialog_id: &str,    last_id: i64,    limit: u32,    output_file: &str,) -> Result<(), Box<dyn std::error::Error>> {
    let json_value = pull_messages_raw(client, base_webhook_url, dialog_id, last_id, limit).await.expect("ERROR");
    json_to_file(output_file, json_value)
}

pub async fn fetch_recent_list(    client: Client,    base_webhook_url: &str,    params: Value,    output_file: &str,) -> Result<(), Box<dyn std::error::Error>> {
    let js = fetch_recent_list_raw(client, base_webhook_url, params).await?;
    json_to_file(output_file, js)?;
    Ok(())
}

pub async fn fetch_recent_list_raw(    client: Client,    base_webhook_url: &str,    params: Value,) -> Result<Value, reqwest::Error> {
    let suffix: &str = "/im.recent.list";
    let resp = client.post(format!("{base_webhook_url}{suffix}")).header(CONTENT_TYPE, "application/json").json(&params).send().await?;
    let json = resp.json().await?;
    Ok(json)
}

async fn fallback_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Страница не найдена")
}


pub async fn spawn() -> anyhow::Result<()> {
    let shared_state = Arc::new(Mutex::new(Vec::<KeyValueMessage>::new()));
    let app = Router::new().route("/test", get(hello_handler)).route("/test", post(post_handler)).with_state(shared_state).fallback(fallback_handler);
    let port = 3000;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 Сервер запущен на http://localhost:{}/test", port);
    println!("   POST /test — принимает protobuf (id, key, value)");
    println!("   GET  /test — возвращает 'hello world'");
    println!("   (ручной декодер, без prost)");
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}


async fn send_to_decode(text: &str) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post(cl_address())
        .form(&[("input", text)])
        .send()
        .await?;
    let body = response.text().await?;
    Ok(body)
}
#[derive(Debug, Deserialize, Clone)]
struct Department {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "NAME")]
    name: String,
    #[serde(default, rename = "PARENT")]
    parent: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DepartmentJson {
    result: Vec<Department>,
}

// ==================== ВЫЧИСЛЕНИЕ ПОЛЕЙ ====================



// ==================== ЧТЕНИЕ ФАЙЛА ====================

/// Читает файл построчно в UTF-8

/// Загружает иерархию отделов из JSON файла
fn load_departments_from_file(path: &str) -> Result<Vec<Department>, Box<dyn std::error::Error>> {
    let lines = read_lines_utf8(path);
    
    if lines.is_empty() {
        return Err(format!("Файл {} пуст или не найден", path).into());
    }
    
    // Объединяем строки в один JSON
    let json_content = lines.join("\n");
    
    // Пробуем распарсить как {"result": [...]}
    let json: DepartmentJson = serde_json::from_str(&json_content)?;
    Ok(json.result)
}

// ==================== ВЫЧИСЛЕНИЕ ПОЛЕЙ ====================

/// Вычисляет ID департамента и управления по ID отдела пользователя
fn compute_department_ids(
    user_department_id: &str,
    departments: &[Department],
) -> Option<(String, String)> {
    let dept_map: HashMap<&str, &Department> = departments
        .iter()
        .map(|d| (d.id.as_str(), d))
        .collect();

    let mut hierarchy: Vec<&Department> = Vec::new();
    let mut current_id = user_department_id;

    while let Some(dept) = dept_map.get(current_id) {
        hierarchy.push(dept);
        match &dept.parent {
            Some(parent_id) => current_id = parent_id.as_str(),
            None => break,
        }
    }

    if hierarchy.is_empty() {
        return None;
    }

    // Департамент: ID 92 → 634
    let dept_enum_id = hierarchy
        .iter()
        .find(|d| d.id == "92")
        .map(|_| "634".to_string())
        .unwrap_or("634".to_string());

    // Управление: маппинг ID родителя в ID справочника
    let mgmt_enum_id = if hierarchy.len() >= 2 {
        match hierarchy[1].id.as_str() {
            "124" => "650", // Управление строительства
            "110" => "648", // Управление Технического заказчика
            "102" => "646", // Управление ИТ
            "144" => "638", // Управление маркетинга
            "146" => "640", // Управление продаж
            "156" => "642", // Финансовое управление
            "96"  => "644", // Управление проектирования
            "170" => "652", // Управление сервиса
            _ => "650",
        }.to_string()
    } else {
        "650".to_string()
    };

    Some((dept_enum_id, mgmt_enum_id))
}

/// Формирует JSON payload для Bitrix24 API
fn build_update_payload(user_id: &str, dept_id: &str, mgmt_id: &str) -> serde_json::Value {
    serde_json::json!({
        "ID": user_id,
        "UF_USR_1787903292765": dept_id,
        "UF_USR_1787903545634": mgmt_id
    })
}



 fn get_test_departments() -> Vec<Department> {
        load_departments_from_file("deps.js").expect("Не удалось загрузить deps.js")
    }
async fn update_user_departments(
    webhook_url: &str,
    user_id: &str,
    department_id: &str,
    departments: &[Department],
) -> Result<(), Box<dyn std::error::Error>> {
    let client: Client = Client::new();

    // Вычисляем ID департамента и управления
    let (dept_enum_id, mgmt_enum_id) = compute_department_ids(department_id, departments)
        .ok_or("Не удалось вычислить поля для пользователя")?;

    // Выводим вычисляемые параметры в консоль
    println!("=== Обновление пользователя {} ===", user_id);
    println!("  Отдел пользователя: {}", department_id);
    println!("  Департамент (UF_USR_1787903292765): {}", dept_enum_id);
    println!("  Управление (UF_USR_1787903545634): {}", mgmt_enum_id);

    // Формируем payload для обновления
    let payload = json!({
        "ID": user_id,
        "UF_USR_1787903292765": dept_enum_id,
        "UF_USR_1787903545634": mgmt_enum_id
    });

    // Отправляем запрос на обновление
    let update_response = client
        .post(format!("{}/user.update", webhook_url))
        .json(&payload)
        .send()
        .await?;

    let result: Value = update_response.json().await?;

    if result.get("result").is_some() {
        println!("✓ Пользователь {} успешно обновлён", user_id);
        Ok(())
    } else {
        println!("✗ Ошибка Bitrix24: {:?}", result);
        Err(format!("Ошибка Bitrix24: {:?}", result).into())
    }
}
/// Обновляет поля пользователя с загрузкой отделов из файла
/// 
/// # Arguments
/// * `webhook_url` - URL вебхука Bitrix24
/// * `user_id` - ID пользователя в Bitrix24
/// * `department_id` - ID отдела пользователя
/// * `deps_file` - Путь к файлу с иерархией отделов (deps.js)
pub async fn update_user_with_file(
    webhook_url: &str,
    user_id: &str,
    department_id: &str,
    deps_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let departments = load_departments_from_file(deps_file)?;
    update_user_departments(webhook_url, user_id, department_id, &departments).await
}





#[tokio::main]
async fn main() -> anyhow::Result<()> {    spawn().await}

#[cfg(test)]
mod tests {
    use core::panic;
use std::assert_eq;
use std::println;

    use crate::http_Proc;

    use super::*;
    use common::*;

    #[test]
    fn test_process_msg() {
        let msg = ExtractedMessage {author_name: "Сергей Браташов".to_string(),text: "Согласовано".to_string(),uuid: Some("851fba1b-35d9-4b61-aaaa-0258fc093efd".to_string()),
            id: 100822,chat_id: 9796, attaches: Vec::new()};
        assert_eq!(100822, http_Proc::process_message2(msg));
    }


    #[tokio::test]
    async fn test_websocket_extract_quote() {
        let resp = get_text_via_chat_id_and_id(OKLAND.to_string(), 118782).await;
        match resp {
            Ok(text) => {println!("EXTRACTED::{}", text)}
            Err(_) => {
                println!("FAILED!"); 
                panic!("SHIT HAPPENS! Seems websocket not works!");
            }

        }
    }

    #[test]
    fn test_bullshit_test() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let conf = ConfigProcess {switch_mode: common::SwitchIDMode::FROM_TO,from: 10000,to: 122000,enabled_collabs: vec![Collab::OKLAND],};
            let msg = __pull_messages_prod_throw_collab_and_filter(Collab::OKLAND, conf).await;
            println!("SIZE arr::{}", msg.len());
            assert_ne!(msg.len(), 0);
        });
    }

    #[test]
    fn test_target_msg(){
        let txt = "Согласовано";
        assert_eq!(true, check_target(txt))
    }


    #[tokio::test]
    async fn test_websocket_extract_quote22() {
        let id_old = 118782;
        let id__ = 123820;
        let resp: std::result::Result<String, anyhow::Error> = get_text_via_chat_id_and_id(OKLAND.to_string(), id_old).await;
        match resp {
            Ok(text) => {             println!("EXTRACTED::>>>{}", text)            }
            Err(e) => {                println!("FAILED!, error::{}", e)            }
        }
        let resp2: std::result::Result<String, anyhow::Error> = get_text_via_chat_id_and_id(OKLAND.to_string(), id__).await;
        match resp2 {
            Ok(text) => {                println!("EXTRACTED::>>>{}", text)            }
            Err(e) => {                   println!("FAILED!, error::{}", e)            }
        }
    }


    #[tokio::test]
    async fn extract_text_4_cl(){
        let id = 125212;
        let resp: std::result::Result<String, anyhow::Error> = get_text_via_chat_id_and_id(OKLAND.to_string(), id).await;
        match resp {
            Ok(text) => {             println!("EXTRACTED::>>>{}", text)            }
            Err(e) => {                println!("FAILED!, error::{}", e)            }
        }
    }


    #[tokio::test]
    async fn test_send_to_decode() {    //append address cl in cl_address file
        let text = "1) Доска 25х100 - 20 шт.\n2) Саморезы 3,5x51 - 1000 шт.\n3) Гвозди 100 мм. - 10 кг.";
        let response = send_to_decode(text).await.unwrap();
        println!("Ответ сервера: {}", response);
        // Проверяем, что ответ содержит хотя бы одну позицию
        assert!(response.contains("Доска"));
    }


    #[test]
    fn test_bullshit_test2() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let conf = ConfigProcess {switch_mode: common::SwitchIDMode::FROM_TO,from: 10000,to: 122000,enabled_collabs: vec![
                 Collab::OLIVIA,    Collab::BABEFA,    Collab::OKLAND,    Collab::RED,    Collab::TETRIS,
                 Collab::SCANDINAVIA, Collab::KUIB,    Collab::POLZ,      Collab::ZVEZD,     Collab::SKY,   
                ],};
            let msg = __pull_messages_prod_throw_collabs_and_filter__( conf).await;
            println!("SIZE arr::{}", msg.len());
            assert_ne!(msg.len(), 0);
            for m in msg{
                println!("MSG::{}\n", m.to_string());
            }
        });
    }

    #[test]
    fn test_body_check() {
        let input = r#"Прошу согласовать манипулятор на 16.06.26 к 08:00 на Школьная 7Я для завоза опалубки на З-Рыбацкая 41
                                    конт. тел. Школьная 8999-645-07-21 Иван
                                    Конт.тел. Рыбацкая 8917-091-14-10 Дмитрий"#;

        let input_good = r#"	Для производства работ по объекту Рыбацкая, прошу согласовать: 1. Песок строительный - 20 тн. Конт. тел 8917-091-14-10 Дмитрий"#;                                    
        let skipped_words: Vec<String> = vec! ["ШАХМАН".to_string(), "ГАЗЕЛЬ".to_string(), "МАНИПУЛЯТОР".to_string(),];
                            
        assert_eq!(false, check_body(input, skipped_words.clone()))   ;        
        assert_eq!(true, check_body(input_good, skipped_words.clone())) ;                          
                  
    }

///////////////  DEPS tests    


    #[test]
    fn test_read_file_success() {
        let lines = read_lines_utf8("deps.js");
        assert!(!lines.is_empty(), "Файл deps.js должен существовать и не быть пустым");
        println!("Прочитано строк: {}", lines.len());
    }

    #[test]
    fn test_read_file_not_found() {
        let lines = read_lines_utf8("nonexistent_file.json");
        assert!(lines.is_empty(), "Несуществующий файл должен вернуть пустой вектор");
    }

    #[test]
    fn test_load_departments() {
        let departments = get_test_departments();
        assert!(!departments.is_empty(), "Список отделов не должен быть пустым");
        println!("Всего отделов: {}", departments.len());
    }

    #[test]
    fn test_user_229_muzdanbaev() {
        // Пользователь 229: отдел 132 (Строительный участок)
        // Иерархия: 132 → 124 → 92
        let departments = get_test_departments();
        let (dept_id, mgmt_id) = compute_department_ids("132", &departments).unwrap();

        assert_eq!(dept_id, "634", "Департамент должен быть Операционный (634)");
        assert_eq!(mgmt_id, "650", "Управление должно быть строительства (650)");
    }

    #[test]
    fn test_user_336_pastushkov() {
        // Пользователь 336: отдел 108 (Отдел ИИ, интеграции и разработки)
        // Иерархия: 108 → 102 → 92
        let departments = get_test_departments();
        let (dept_id, mgmt_id) = compute_department_ids("108", &departments).unwrap();

        assert_eq!(dept_id, "634", "Департамент должен быть Операционный (634)");
        assert_eq!(mgmt_id, "646", "Управление должно быть ИТ (646)");
    }

    #[test]
    fn test_user_189_kalyuzhina() {
        // Пользователь 189: отдел 112 (Отдел управления проектами)
        // Иерархия: 112 → 110 → 92
        let departments = get_test_departments();
        let (dept_id, mgmt_id) = compute_department_ids("112", &departments).unwrap();

        assert_eq!(dept_id, "634", "Департамент должен быть Операционный (634)");
        assert_eq!(mgmt_id, "648", "Управление должно быть Тех. заказчика (648)");
    }

    #[test]
    fn test_unknown_department() {
        let departments = get_test_departments();
        let result = compute_department_ids("999", &departments);

        assert!(result.is_none(), "Неизвестный отдел должен вернуть None");
    }

    #[test]
    fn test_build_payload() {
        let payload = build_update_payload("229", "634", "650");

        assert_eq!(payload["ID"], "229");
        assert_eq!(payload["UF_USR_1787903292765"], "634");
        assert_eq!(payload["UF_USR_1787903545634"], "650");
    }

    #[test]
    fn test_hierarchy_building() {
        let departments = get_test_departments();
        let dept_map: HashMap<&str, &Department> = departments
            .iter()
            .map(|d| (d.id.as_str(), d))
            .collect();

        // Отдел 132 должен иметь родителя 124
        let dept_132 = dept_map.get("132").unwrap();
        assert_eq!(dept_132.parent, Some("124".to_string()));

        // Отдел 124 должен иметь родителя 92
        let dept_124 = dept_map.get("124").unwrap();
        assert_eq!(dept_124.parent, Some("92".to_string()));

        // Отдел 92 должен иметь родителя 178
        let dept_92 = dept_map.get("92").unwrap();
        assert_eq!(dept_92.parent, Some("178".to_string()));
    }

    #[test]
    fn test_root_department() {
        let departments = get_test_departments();
        let dept_map: HashMap<&str, &Department> = departments
            .iter()
            .map(|d| (d.id.as_str(), d))
            .collect();

        // Корневой отдел ID 1 не должен иметь родителя
        let root = dept_map.get("1").unwrap();
        assert!(root.parent.is_none(), "Корневой отдел не должен иметь родителя");
        assert_eq!(root.name, "RES DEVELOPMENT УЧРЕДИТЕЛИ");
    }

#[tokio::test]
async fn test_update_atom() {
    let webhook = get_webhook();
    let departments = load_departments_from_file("deps.js").expect("Не удалось загрузить deps.js");
    update_user_departments(webhook.as_str(), "229", "132", &departments).await;
}

use crate::Pack;
use crate::Employee;
use crate::ADD_DUMP;
#[tokio::test]
async fn test_update_atom_rahman() {
    let mut pack = Pack::deserialize_from_file(ADD_DUMP).expect("msg");
    let mass: Vec<&Employee> = pack.pack.iter().filter(|&q|  q.last_name == "Рахман" ).collect();
    println!("{}", mass[0]._to_string());
    let emp =  mass[0];
    let deps_user =  emp.map_add.get(&crate::ADDITIONAL_FIELDS::UF_DEPARTMENT).unwrap();
    let id_user = emp.id.to_string();
    if deps_user.contains(",") {
        println!("WARNING!!! user {} has multiple deps", emp._to_string());
    }
    assert_eq!("104",  deps_user.replace("[", "").replace("]", "") );
    assert_eq!("135",  id_user );


    let webhook = get_webhook();
    let departments = load_departments_from_file("deps.js").expect("Не удалось загрузить deps.js");
    let _ =update_user_departments(webhook.as_str(), id_user.as_str(), deps_user.as_str(), &departments).await;
}


#[tokio::test]
async fn test_update_atom_rahman2() {
    let mut pack = Pack::deserialize_from_file(ADD_DUMP).expect("msg");
    let mass: Vec<&Employee> = pack.pack.iter().filter(|&q| q.last_name == "Рахман").collect();
    println!("{}", mass[0]._to_string());
    let emp = mass[0];
    
    let deps_user = emp.map_add.get(&crate::ADDITIONAL_FIELDS::UF_DEPARTMENT).unwrap();
    let id_user = emp.id.to_string();
    
    if deps_user.contains(",") {
        println!("WARNING!!! user {} has multiple deps", emp._to_string());
    }
    
    // Проверяем формат
    assert_eq!("[104]", deps_user);
    assert_eq!("135", id_user);

    // Извлекаем чистый ID отдела (убираем скобки)
    let dept_id = deps_user.replace("[", "").replace("]", "");
    assert_eq!("104", &dept_id);

    let webhook = get_webhook();
    let departments = load_departments_from_file("deps.js").expect("Не удалось загрузить deps.js");
    
    // Вызываем функцию и обрабатываем результат
    let result = update_user_departments(webhook.as_str(), &id_user, &dept_id, &departments).await;
    
    // Выводим результат
    match result {
        Ok(_) => println!("✓ Тест завершён успешно"),
        Err(e) => println!("✗ Ошибка обновления: {}", e),
    }
    
    // Или через ? для propagation ошибки
    // update_user_departments(webhook.as_str(), &id_user, &dept_id, &departments).await?;
}


use crate::ADDITIONAL_FIELDS;
#[tokio::test]
async fn test_update_atom_packed() {
    let mut pack = Pack::deserialize_from_file(ADD_DUMP).expect("msg");

    // Используем for цикл вместо .for_each() для async операций
    for emp in pack.pack.iter().filter(|&q| 
        q.map_add.get(&ADDITIONAL_FIELDS::ACTIVE).unwrap().to_string() == "true"
    ) {
        println!("PROCESS::{}  {}", emp.name, emp.last_name);
        
        let deps_user = emp.map_add.get(&crate::ADDITIONAL_FIELDS::UF_DEPARTMENT).unwrap();
        let id_user = emp.id.to_string();
        
        if deps_user.contains(",") {
            println!("WARNING!!! user {} has multiple deps", emp._to_string());
        }

        let dept_id = deps_user.replace("[", "").replace("]", "");

        let webhook = get_webhook();
        let departments = load_departments_from_file("deps.js").expect("Не удалось загрузить deps.js");

        let result = update_user_departments(webhook.as_str(), &id_user, &dept_id, &departments).await;

        match result {
            Ok(_) => println!("✓ Тест завершён успешно"),
            Err(e) => println!("✗ Ошибка обновления: {}", e),
        }
    }

    // Дополнительная проверка для Рахмана
    let mass: Vec<&Employee> = pack.pack.iter().filter(|&q| q.last_name == "Рахман").collect();
    println!("{}", mass[0]._to_string());
    let emp = mass[0];
    
    let deps_user = emp.map_add.get(&crate::ADDITIONAL_FIELDS::UF_DEPARTMENT).unwrap();
    let id_user = emp.id.to_string();
    
    if deps_user.contains(",") {
        println!("WARNING!!! user {} has multiple deps", emp._to_string());
    }
    
    assert_eq!("[104]", deps_user);
    assert_eq!("135", id_user);

    let dept_id = deps_user.replace("[", "").replace("]", "");
    assert_eq!("104", &dept_id);

    let webhook = get_webhook();
    let departments = load_departments_from_file("deps.js").expect("Не удалось загрузить deps.js");
    
    let result = update_user_departments(webhook.as_str(), &id_user, &dept_id, &departments).await;
    
    match result {
        Ok(_) => println!("✓ Тест завершён успешно"),
        Err(e) => println!("✗ Ошибка обновления: {}", e),
    }
}
}

///////////////////////////////////////////    


