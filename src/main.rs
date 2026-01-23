use reqwest::Client;
use reqwest;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use bincode;
use std::fs::File;
use std::io::{Read, Write};
use std::process::id;
use serde::de::DeserializeOwned;

pub mod http_Test;


fn deserialize_from_file<T: DeserializeOwned>(filename: &str) -> Result<T, Box<dyn std::error::Error>>{
    let mut file: File = File::open(filename)?;
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer)?;
    let decoded: T = bincode::deserialize(&buffer)?;
    Ok(decoded)
}


#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Employee{
    id: i32,
    name: String,
    last_name: String,
    middle_name: String,    
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Pack{
    pack: Vec<Employee>
}

impl Pack{
    fn new(pack: Vec<Employee>) -> Self{
        Pack {pack}
    }

    fn serialize_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>>{
        let encoded: Vec<u8> = bincode::serialize(self)?;
        let mut file = File::create(filename)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    fn deserialize_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>>{
        deserialize_from_file(filename)
    }

    fn push_and_update(&mut self, entry: Employee) -> (){
        if (self.is_contains(&entry)){
            self.remove(&entry);
        }
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

            if cur_id == entry_id {
                return true;
            }

            if (cur_name == entry_name) && (cur_last_name == entry_last_name) {
                return true;
            }
        }
        false
    }

    fn remove(&mut self, entry: &Employee) -> bool {
        let index = self.pack.iter().position(|emp| {
            emp.id == entry.id || 
            (emp.name == entry.name && emp.last_name == entry.last_name)
        });
        
        if let Some(idx) = index {
            self.pack.remove(idx);
            true
        } else {
            false
        }
    }  


    fn to_string(&self, ender: String) -> String{
        let mut result = String::from("");
        for emp in &self.pack {
            result.push_str(&emp._to_string());
            result.push_str(&ender);

        }
        result
    }

    fn get_id_by_fi(&self, fi: String)-> Option<i32>{
        let splitted: Vec<String> = split_to_fi(fi);
        if (splitted.len()<3){
            return None
        }
            
        let last_name  = &splitted[0];
        let first_name = &splitted[1];

        println!("F:{}", last_name.to_string());
        println!("I:{}", first_name.to_string());

        
        for employee in &self.pack {

            println!("EMPLOYEE    F:{}", employee.last_name.to_string());
            println!("EMPLOYEE    I:{}", employee.name.to_string());
            if employee.last_name == last_name.to_string() && employee.name == first_name.to_string() {
                return Some(employee.id);
            }
        }    
        None
    }
}

impl Employee {
     fn new(id:i32, name: String, last_name: String, middle_name: String) -> Self {
         Employee {id, name, last_name, middle_name}
     }

     fn serialize_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>>{
         let encoded: Vec<u8> = bincode::serialize(self)?;
         let mut file = File::create(filename)?;
         file.write_all(&encoded)?;
         Ok(())    
     }

     fn deserialize_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>>{
         deserialize_from_file(filename)
     }

     fn _to_string(&self) -> String{
        format!("{} {} {} {}", self.id, self.name, self.middle_name, self.last_name)
     } 
}

fn get_i32_from_value(value: &Value) -> Option<i32> {
    match value {
        Value::Number(n) => n.as_i64().map(|x| x as i32),
        Value::String(s) => s.parse::<i32>().ok(),
        _ => None,
    }
}

async fn grub_data(index_start: i32, index_stop: i32, filename_to_dump: &str)->   Result<(), Box<dyn std::error::Error>> {
    let mut init_buffer: Vec<Employee> = Vec::new();
    let mut pack = Pack::new(init_buffer);
    let client_reqwest: Client = reqwest::Client::new();
    for _i in index_start..index_stop{
        match http_Test::get_user_by_id(client_reqwest.clone(), _i).await {
        Ok(data) => {
            if let Some(users) = data.get("result") {
                if users.is_array() {
                    for user in users.as_array().unwrap() {
                        let id =  user.get("ID").unwrap_or(&Value::Null);
                        let name = user.get("NAME").unwrap_or(&Value::Null);
                        let last_name =   user.get("LAST_NAME").unwrap_or(&Value::Null);
                        let second_name = user.get("SECOND_NAME").unwrap_or(&Value::Null);
                        println!("ID: {}", id);
                        println!("Имя: {}", name);
                        println!("Фамилия: {}", last_name);
                        println!("Отчество: {}", second_name);

                        println!("---");
                        let number_id = get_i32_from_value(id).expect("shit");
                        let emp = Employee::new(number_id, name.to_string().replace("\"", ""), 
                        last_name.to_string().replace("\"", ""), second_name.to_string().replace("\"", ""));
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


async fn try_grub() ->   Result<(), Box<dyn std::error::Error>> {
    grub_data(1, 400, "all_dump.bin").await
}

async fn process(client_reqwest: Client) ->   Result<(), Box<dyn std::error::Error>> {
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


    let z20 = [ ("ID", "283"), ("PERSONAL_MOBILE", "+7927581-68-51")]; 
    let z22 = [ ("ID", "283"), ("WORK_POSITION", "Ведущий инженер-конструктор")];   
    let z21 = [ ("ID", "285"), ("WORK_POSITION", "Специалист по корпоративной безопасности")]; 

   
    let arr = vec! [z1, z11, z12, z13, z14, z15, z16, z17, z18, z19];

    let arr2: Vec<[(&str, &str); 2]> = vec! [z20, z21, z22];

    // for item in arr.iter(){        
    //     let result = update_param_(item).await?;
    //     println!("Response body: {}", result);
    // };
        for item in arr.iter(){        
        let result = http_Test::update_param_(client_reqwest.clone(), item).await?;
        println!("Response body: {}", result);
    };
      
    for item in arr2.iter(){        
        let result =  http_Test::update_param_(client_reqwest.clone(), item).await?;
        println!("Response body: {}", result);
    };
    Ok(())
}


async fn getting_users(client_reqwest: Client) -> (){
    println!("=== Получение пользователя по ID 111111111111111111111===");
    match http_Test::get_user_by_id(client_reqwest.clone(), 1).await {
        Ok(data) => {
            if let Some(users) = data.get("result") {
                if users.is_array() {
                    for user in users.as_array().unwrap() {
                        println!("ID: {}", user.get("ID").unwrap_or(&Value::Null));
                        println!("Имя: {}", user.get("NAME").unwrap_or(&Value::Null));
                        println!("Фамилия: {}", user.get("LAST_NAME").unwrap_or(&Value::Null));
                        println!("Отчество: {}", user.get("SECOND_NAME").unwrap_or(&Value::Null));

                       /// println!("Email: {}", user.get("EMAIL").unwrap_or(&Value::Null));
                        println!("---");
                    }
                }
            }
        }
        Err(e) => println!("Ошибка: {}", e),
    }
    
    // Пример 2: Получаем пользователя с определенными полями
    println!("\n=== Получение пользователя с определенными полями    222222222222222222 ===");
    let fields = vec!["ID", "NAME", "LAST_NAME", "EMAIL", "PERSONAL_MOBILE"];
    match http_Test::get_user_with_fields(client_reqwest.clone(), 1, &fields).await {
        Ok(data) => println!("{:#?}", data),
        Err(e) => println!("Ошибка: {}", e),
    }
    
    // Пример 3: Получаем несколько пользователей
    println!("\n=== Получение нескольких пользователей 333333333333333333333333333333===");
    let user_ids = vec![1, 2, 3];
    match http_Test::get_multiple_users(client_reqwest.clone(), &user_ids).await {
        Ok(data) => {
            if let Some(result) = data.get("result") {
                println!("Найдено пользователей: {}", result.as_array().unwrap().len());
            }
        }
        Err(e) => println!("Ошибка: {}", e),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_reqwest: Client = reqwest::Client::new();
    //process(client_reqwest).await
   // proc2();
   // getting_users(client_reqwest).await;
   try_grub().await
    //Ok(())


    
}


pub fn add(a: i32, b:i32) -> i32 {
    a + b
}

pub fn bad_add(a: i32, b: i32) -> i32{
    a - b
}



pub fn split_to_fi(input: String)->Vec<String>{
    let mut res: Vec<String> = Vec::new();
    let splitted = input.split_whitespace();
    for _i in splitted{
        res.push(_i.to_string());
    }
    res.push(input);
    res
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_add(){
        assert_eq!(add(1, 2), 3);
    }

    #[test]
    fn test_bad_add(){
        assert_eq!(bad_add(1, 2), -1);
    }

    #[test]
    fn test_read_str(){
        let filename = "lstdata.csv";
        let etalon = ";ФИО;Должность;;;;телефон;юр. лицо";
        let vect = http_Test::read_lines(filename);
        let line0 = vect[0].clone();
        assert_eq!(etalon, line0);
    }

    #[test]
    fn test_read_webbhook(){
        const WEBHOOK_FILENAME_TEST: &str = "webhook_test";
        let test_webhook = http_Test::get_webhook_(WEBHOOK_FILENAME_TEST);
        let etalon = "http://google.com";
        assert_eq!(etalon, test_webhook);

    }


    #[test]
    fn test_ser(){
        let emp = Employee::new(1, "John Doe".to_string(), "DOE".to_string(), "DOE".to_string());
        emp.serialize_to_file("employee.bin");
        println!("Data serialized to employee.bin");
        let emp2 = Employee::deserialize_from_file("employee.bin").expect("shit");
        println!("Deserialized data: {:?}", emp2);
        assert_eq!(emp, emp2);
    }

    #[test]
    fn test_sr_dsr(){
        let emp1 = Employee::new(1, "John Doe".to_string(), "DOE".to_string(), "DOE".to_string());
        let emp2 = Employee::new(1, "John Doe".to_string(), "DOE".to_string(), "DOE".to_string());
        let pack_filename = "pack.bin";
        let emp_Vecs = vec![emp1, emp2];
        let pack = Pack::new(emp_Vecs);
        pack.serialize_to_file( pack_filename);
        let restored = Pack::deserialize_from_file(pack_filename).expect("shit");
        assert_eq!(pack, restored);

        assert_eq!(2, 2);
    }

    #[test]
    fn test_contains_pack(){
        let emp1 = Employee::new(1, "Michael".to_string(), "Snoyman".to_string(), "".to_string());
        let emp2 = Employee::new(2, "Roman".to_string(), "Pastushkov".to_string(), "DOE".to_string());
        let pack_filename = "pack.bin";
        let emp_Vecs = vec![emp1, emp2];
        let mut pack = Pack::new(emp_Vecs);
        let emp3 = Employee::new(1, "Michael".to_string(), "Snoyman".to_string(), "".to_string());
        let emp4 = Employee::new(99, "Michael".to_string(), "Snoyman".to_string(), "".to_string());
        let emp5 = Employee::new(99, "Michael2".to_string(), "Snoyman".to_string(), "".to_string());
        let emp6 = Employee::new(1, "Michael2".to_string(), "Snoyman".to_string(), "".to_string());

        let res = pack.is_contains(&emp3);
        assert_eq!(true,  res);

        
        let res2 = pack.is_contains(&emp4);
        assert_eq!(true,  res2);

        let res3 = pack.is_contains(&emp5);
        assert_eq!(false,  res3);

        let res4 = pack.is_contains(&emp6);
        assert_eq!(true,  res4);

        println!("{}", pack.to_string("\n".to_string()))


    }


     #[test]
    fn test_delete_pack(){
        let emp1 = Employee::new(1, "Michael".to_string(), "Snoyman".to_string(), "".to_string());
        let emp2 = Employee::new(2, "Roman".to_string(), "Pastushkov".to_string(), "DOE".to_string());
        let emp_Vecs = vec![emp1, emp2];
        let mut pack = Pack::new(emp_Vecs);
        let emp3 = Employee::new(1, "Michael".to_string(), "Snoyman".to_string(), "".to_string());
      
        let res = pack.is_contains(&emp3);
        assert_eq!(true,  res);

        pack.remove(&emp3);

         let res2 = pack.is_contains(&emp3);
        assert_eq!(false,  res2);     
    }

    #[test]
    fn test_get_id(){
        let emp1 = Employee::new(1, "Michaelen".to_string(), "Snoyman".to_string(), "".to_string());
        let emp2 = Employee::new(2, "Роман".to_string(), "Пастушков".to_string(), "DOE".to_string());
        let emp3 = Employee::new(1, "Michael".to_string(), "Snoyman".to_string(), "".to_string());
        let emp_Vecs = vec![emp1, emp2, emp3];
        let mut pack = Pack::new(emp_Vecs);
        assert_eq!(2, pack.get_id_by_fi("Пастушков Роман".to_string()).expect("shit"));
    }




    #[test]
    fn test_get_id2(){
        let mut pack = Pack::deserialize_from_file("all_dump.bin").expect("msg");
        println!("{}", pack.to_string("\n".to_string()));
        assert_eq!(1, pack.get_id_by_fi("Цыбульский Сергей".to_string()).expect("shit"));
    }

    #[test]
    fn test_fi(){
        let mut fi = "Roman Pastushkov";
        assert_eq!(3, split_to_fi(fi.to_string()).len());

    }
}





