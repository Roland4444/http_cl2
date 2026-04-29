use axum::extract::State;
use axum::{    Router,    body::Bytes,    http::StatusCode,    response::IntoResponse,    routing::{get, post},};
use futures_util::{SinkExt, StreamExt};
use anyhow::{Context, Result};
use futures::stream::{self,  TryStreamExt}; // в начало файла
use reqwest::Client;
use serde_json::{Value, json};
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
use crate::http_Test::{read_lines, read_lines_utf8, update_param_};
use tokio::net::TcpListener;
use common::*;
const WEBHOOK_FILENAME: &str = "webhook";
use once_cell::sync::Lazy;
use crate::thread;
use parking_lot::Mutex as Mutex2;

static IS_PENDING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, PartialEq)]
pub struct KeyValueMessage {
    pub id: i32,
    pub key: String,
    pub value: String,
}


#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
pub struct ConfigProcess {    pub switch_mode: SwitchIDMode,   pub from: u32,    pub to: u32, pub enabled_collabs: Vec<common::Collab>}


pub static CONFIG: ConfigProcess = ConfigProcess{switch_mode:common::FROM_CURRENT, from:0, to:1000000, enabled_collabs:vec![OKLAND]};

pub fn process_msg(entry: ExtractedMessage)-> bool{    
    let author = entry.author_name;    
    let collab = entry.chat_id;    
    true   
}


pub static ENABLED_COLLABS: Mutex2<Vec<common::Collab>> = Mutex2::new(Vec::new());
pub static ID_FROM_TO: Mutex2<Vec<u32>> = Mutex2::new(Vec::new());

pub fn process_message<F, R>(msg: ExtractedMessage, processor: F) -> R
where
    F: Fn(ExtractedMessage) -> R,
{
    processor(msg)
}


pub fn init_processing_config
//pub static QUEUE: Lazy< std::sync::Mutex<Vec<ExtractedMessage>>> = Lazy::new(||  std::sync::Mutex::new(Vec::new()));

pub static QUEUE: Mutex2<Vec<ExtractedMessage>> = Mutex2::new(Vec::new());
pub static QUEUE2: Mutex2<Vec<ExtractedMessage>> = Mutex2::new(Vec::new());

//pub static QUEUE__: Lazy<Mutex<Vec<ExtractedMessage>>> = Lazy::new(|| Mutex::new(Vec::new()));

//   add::             QUEUE.lock().unwrap().push(msg);
//   remove::             QUEUE.lock().unwrap().remove(index);

//  iteration
//  let mut queue = QUEUE.lock().unwrap();
//  for item in queue.iter() { ... }
pub fn copy_queue_to_queue2() {    
    let queue = QUEUE.lock();    
    let mut queue2 = QUEUE2.lock();    
    queue2.extend(queue.clone()); 
}

pub fn move_queue_to_queue2() -> usize {   
     let mut queue = QUEUE.lock();    
     let mut queue2 = QUEUE2.lock();    
     let len = queue.len();    
     queue2.extend(queue.drain(..));    len
}


pub fn adding_to_Pack(msg: ExtractedMessage) -> bool {    QUEUE.lock().push(msg);    true}

// Синхронный просмотр очереди
pub fn watch() {
    let queue = QUEUE.lock();
    for item in queue.iter() {println!("{}", item.to_string());}
}

fn process_atom_queue(){
    let mut queue: parking_lot::lock_api::MutexGuard<'_, parking_lot::RawMutex, Vec<ExtractedMessage>> = QUEUE.lock();
    let elem = queue.first();
    match elem{
        Some(el__) => {
            println!("DROPPED:{}", el__.to_string());
            queue.remove(0);
        },
        None => {  }
    }        
}







pub fn __func1(mut counter1: u64){
        loop {
            counter1+=1;
            thread::sleep(Duration::from_secs(3));
            process_atom_queue();
            println!("THREAD1::{}\n\n\n", counter1);
            {
                  let mut queue = QUEUE.lock();
                  println!("SIZE QUEUE: {}", queue.len());
                  if queue.len()==0 {break;}
            }
        }
}

pub fn __func2(mut counter2: u64){
        let queue2 = QUEUE2.lock();
        let mut counter = 0;
        let finish_index = queue2.len(); 
        loop {
            if counter < finish_index{
               if let Some(item) = queue2.get(counter) {
                    let mut queue = QUEUE.lock();
                    queue.push(item.clone());
                    println!("SIZE QUEUE: {}", queue.len());
                    println!("SIZE QUE:{}, size", queue.len());
                } 
            }
            else {
                println!("\n\n\nFUNC2 stopped\n\n\n\n");
                break;
            }

            thread::sleep(Duration::from_millis(200));
            println!("THREAD2::{}\n\n\n", counter2);
            counter+=1;

        }
}


pub fn process_function() {
    let  counter1 = 0;
    let  counter2 = 0;
    let __hndl1 = thread::spawn(move || {__func1(counter1);});
    let __hndl2 = thread::spawn(move || {__func2(counter2);});
    __hndl1.join();
    __hndl2.join();
   // loop {thread::sleep(Duration::from_secs(3));}

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

const SYSTEM_MESSAGE: &str = "0";

const CREATE_QUEUE: &str = "CREATE_QUEUE";

const RUN_QUEUE: &str = "RUN_QUEUE";

const CONTENT_TYPE: &str = "Content-type";

const APPROVED: &str = "СОГЛАСОВАНО";

fn process_message2(msg: ExtractedMessage) -> u64 {
    if !msg.text.to_uppercase().contains(APPROVED) {
        println!("SKIPPED");
        msg.id
    } else {
        println!("PROCESSED!");
        msg.id
    }
}

pub async fn get_text_via_chat_id_and_id(chat_name: String, message_id: u64) -> Result<String> {
    let (mut ws_stream, _) = connect_async("ws://127.0.0.1:3000/proc").await.context("Не удалось подключиться к WebSocket")?;

    let request = json!({"collab": chat_name,"message_id": message_id});

    let request_bytes = serde_json::to_vec(&request)?;
    ws_stream.send(Message::Binary(request_bytes.into())).await?;

    if let Some(Ok(Message::Text(resp_text))) = ws_stream.next().await {
        let resp: ExtractResp = serde_json::from_str(&resp_text)?;
        if resp.success {return Ok(resp.quoted_text.unwrap_or_default());} 
        else {            anyhow::bail!("Ошибка сервера: {}", resp.error.unwrap_or_default());        }
    }

    anyhow::bail!("Не получен ответ от сервера");
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

            let items: Vec<(i32, String, String)> = vec
                .iter()
                .map(|kv| (kv.id, kv.key.clone(), kv.value.clone()))
                .collect();

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {    spawn().await}

#[cfg(test)]
mod tests {
    use crate::http_Proc;

    use super::*;
    use common::*;

    #[test]
    fn test_process_msg() {
        let msg = ExtractedMessage {author_name: "Сергей Браташов".to_string(),text: "Согласовано".to_string(),uuid: Some("851fba1b-35d9-4b61-aaaa-0258fc093efd".to_string()),
            id: 100822,chat_id: 9796,};
        assert_eq!(100822, http_Proc::process_message2(msg));
    }


    #[tokio::test]
    async fn test_websocket_extract_quote() {
        let resp = get_text_via_chat_id_and_id(OKLAND.to_string(), 118782).await;
        match resp {
            Ok(text) => {println!("EXTRACTED::{}", text)}
            Err(_) => {println!("FAILED!")}
        }
    }
}
