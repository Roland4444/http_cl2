// use axum::{
//     routing::{get, post},
//     Router,
//     response::IntoResponse,
//     http::StatusCode,
//     extract::Request,
//     body::Bytes,
// };
// use std::net::SocketAddr;
// use tokio::net::TcpListener;
// use prost::Message;

// #[derive(Message, Clone, PartialEq)]  // <- Debug убран из derives
// pub struct KeyValueMessage {
//     #[prost(int32, tag = "1")]
//     pub id: i32,

//     #[prost(string, tag = "2")]
//     pub key: String,

//     #[prost(string, tag = "3")]
//     pub value: String,
// }

// // Ручная реализация Debug — без конфликтов
// impl std::fmt::Debug for KeyValueMessage {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("KeyValueMessage")
//             .field("id", &self.id)
//             .field("key", &self.key)
//             .field("value", &self.value)
//             .finish()
//     }
// }

// pub async fn spawn() -> anyhow::Result<()> {
//     let app = Router::new()
//         .route("/test", get(hello_handler))
//         .route("/test", post(post_handler)) // ← добавили
//         .fallback(fallback_handler);

//     let PORT = 3000;

//     let addr = SocketAddr::from(([0, 0, 0, 0], PORT));
//     println!(          "*********************************************************");
//     println!(          "*********************************************************");
//     println!(          "***  ***************************************    ****  ***");
//     println!(          "***  *******                           *****  ** ***  ***");
//     println!(          "***  *******STARTUP SERVER AT PORT {}*****  *** **  ***", PORT);
//     println!(          "***  *******                           *****  **** *  ***");
//     println!(          "***  ***************************************  *****   ***");
//     println!(          "***       **********************************  ******  ***");
//     println!(          "*********************************************************");
//     println!(" Опрос доступен на русском и фарси");

//     let listener = TcpListener::bind(addr).await?;
//     axum::serve(listener, app).await?;
//     print!("WORKING!!!!");
//     Ok(())
// }

// async fn hello_handler() -> &'static str {
//     "hello world"
// }

// async fn post_handler(body: Bytes) -> impl IntoResponse {
//     match KeyValueMessage::decode(body) {
//         Ok(msg) => {
//             println!(
//                 " Получено сообщение: id = {}, key = {}, value = {}",
//                 msg.id, msg.key, msg.value
//             );
//             (StatusCode::OK, "OK")
//         }
//         Err(e) => {
//             eprintln!(" Ошибка декодирования protobuf: {}", e);
//             (StatusCode::BAD_REQUEST, "Invalid protobuf")
//         }
//     }
// }

// async fn fallback_handler() -> impl IntoResponse {
//     (StatusCode::NOT_FOUND, "Страница не найдена")
// }
use futures::stream::{self, StreamExt, TryStreamExt}; // в начало файла

use axum::{
    Router,
    body::Bytes,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::Client;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc; // для Arc
use tokio::sync::Mutex; // для асинхронного Mutex

use axum::extract::State;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::http_Test::{read_lines, read_lines_utf8, update_param_};
use tokio::net::TcpListener;

const WEBHOOK_FILENAME: &str = "webhook";

static IS_PENDING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, PartialEq)]
pub struct KeyValueMessage {
    pub id: i32,
    pub key: String,
    pub value: String,
}

fn decode_key_value_message(buf: &[u8]) -> Result<KeyValueMessage, String> {
    let mut bytes = buf;
    let mut id = None;
    let mut key = None;
    let mut value = None;

    while !bytes.is_empty() {
        let (key_val, rest) =
            read_varint(bytes).map_err(|e| format!("Ошибка чтения ключа: {}", e))?;
        bytes = rest;

        let tag = key_val >> 3;
        let wire_type = key_val & 0x07;

        match (tag, wire_type) {
            (1, 0) => {
                let (val, rest) =
                    read_varint(bytes).map_err(|e| format!("Ошибка чтения id: {}", e))?;
                id = Some(val as i32);
                bytes = rest;
            }
            (2, 2) => {
                let (len, rest) =
                    read_varint(bytes).map_err(|e| format!("Ошибка чтения длины key: {}", e))?;
                if rest.len() < len as usize {
                    return Err("Недостаточно байт для key".into());
                }
                let str_bytes = &rest[..len as usize];
                let s = String::from_utf8(str_bytes.to_vec())
                    .map_err(|e| format!("key не валидный UTF-8: {}", e))?;
                key = Some(s);
                bytes = &rest[len as usize..];
            }
            (3, 2) => {
                let (len, rest) =
                    read_varint(bytes).map_err(|e| format!("Ошибка чтения длины value: {}", e))?;
                if rest.len() < len as usize {
                    return Err("Недостаточно байт для value".into());
                }
                let str_bytes = &rest[..len as usize];
                let s = String::from_utf8(str_bytes.to_vec())
                    .map_err(|e| format!("value не валидный UTF-8: {}", e))?;
                value = Some(s);
                bytes = &rest[len as usize..];
            }
            _ => match wire_type {
                0 => {
                    let (_, rest) =
                        read_varint(bytes).map_err(|e| format!("Ошибка пропуска varint: {}", e))?;
                    bytes = rest;
                }
                2 => {
                    let (len, rest) = read_varint(bytes)
                        .map_err(|e| format!("Ошибка чтения длины для пропуска: {}", e))?;
                    if rest.len() < len as usize {
                        return Err("Недостаточно байт для пропуска поля".into());
                    }
                    bytes = &rest[len as usize..];
                }
                _ => {
                    return Err(format!(
                        "Неподдерживаемый wire type {} для пропуска",
                        wire_type
                    ));
                }
            },
        }
    }

    Ok(KeyValueMessage {
        id: id.ok_or("Отсутствует поле id")?,
        key: key.ok_or("Отсутствует поле key")?,
        value: value.ok_or("Отсутствует поле value")?,
    })
}

fn read_varint(buf: &[u8]) -> Result<(u64, &[u8]), &'static str> {
    let mut result = 0u64;
    let mut shift = 0;
    let mut pos = 0;

    while pos < buf.len() {
        let byte = buf[pos];
        result |= ((byte & 0x7F) as u64) << shift;
        pos += 1;
        if byte & 0x80 == 0 {
            return Ok((result, &buf[pos..]));
        }
        shift += 7;
        if shift > 63 {
            return Err("varint слишком длинный");
        }
    }
    Err("неожиданный конец varint")
}

async fn hello_handler() -> &'static str {
    "hello world"
}

pub fn get_webhook() -> String {
    get_webhook_(WEBHOOK_FILENAME)
}

pub fn get_webhook_(filename: &str) -> String {
    let vec = read_lines(filename);
    let elem = vec[0].clone();
    elem
}

pub async fn update_param_2(
    client_reqwest: Client,
    params: &[(&str, &str)],
) -> Result<String, Box<dyn Error>> {
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

async fn processmsg(
    msg: KeyValueMessage,
    vec: &mut Vec<KeyValueMessage>,
) -> Result<(), Box<dyn Error>> {
    println!("✅ Получено сообщение: {:?}", msg);

    let client_reqwest = Client::new();

    let id_str = msg.id.to_string();
    let key_str = msg.key.as_str();
    let value_str = msg.value.as_str();

    let item = [("ID", id_str.as_str()), (key_str, value_str)];

    if id_str == SYSTEM_MESSAGE.to_string() {
        println!("SYSTEM MESSAGE: {}", id_str);
        println!("VALUE MESSAGE: {}", key_str);
        if key_str == CREATE_QUEUE {
            IS_PENDING.store(true, Ordering::SeqCst);
        }

        if key_str == RUN_QUEUE {
            if vec.len() == 0{
                println!("\n\n\n\nאפס\n\n\n\n!!");
                return Ok(())
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

        // for item3 in vec.iter() {

        //     let id_str2 = item3.id.to_string();
        //     let key_str2 = item3.key.as_str();
        //     let value_str2 = item3.value.as_str();

        //     let item2 = [
        //     ("ID", id_str2.as_str()),
        //     (key_str2, value_str2),
        //     ];

        //     let result = update_param_2(client_reqwest.clone(), &item2).await?;
        //     println!("Response body: {}", result);
        // }
        //END PARALLEL

        //}
        return Ok(())

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
            let mut vec: tokio::sync::MutexGuard<'_, Vec<KeyValueMessage>> = state.lock().await; // теперь .await корректен
            processmsg(msg, &mut *vec).await;

            (StatusCode::OK, "OK")
        }
        Err(e) => {
            eprintln!("❌ Ошибка декодирования: {}", e);
            (StatusCode::BAD_REQUEST, "Invalid protobuf")
        }
    }
}

// async fn post_handler(
//     body: Bytes,                       // тело запроса
// ) -> impl IntoResponse {
//     match decode_key_value_message(&body) {
//         Ok(msg) => {
//             println!("✅ Получено сообщение: {:?}", msg);

//             processmsg(msg).await;
//             (StatusCode::OK, "OK")
//         }
//         Err(e) => {
//             eprintln!("❌ Ошибка декодирования protobuf: {}", e);
//             (StatusCode::BAD_REQUEST, "Invalid protobuf")
//         }
//     }
// }

async fn fallback_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Страница не найдена")
}

pub async fn spawn() -> anyhow::Result<()> {
    let shared_state = Arc::new(Mutex::new(Vec::<KeyValueMessage>::new()));

    let app = Router::new()
        .route("/test", get(hello_handler))
        .route("/test", post(post_handler))
        .with_state(shared_state)
        .fallback(fallback_handler);

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
async fn main() -> anyhow::Result<()> {
    spawn().await
}
