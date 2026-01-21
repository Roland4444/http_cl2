use reqwest::Client;
pub mod http_test;

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
        let result = http_test::update_param_(client_reqwest.clone(), item).await?;
        println!("Response body: {}", result);
    };
      
    for item in arr2.iter(){        
        let result =  http_test::update_param_(client_reqwest.clone(), item).await?;
        println!("Response body: {}", result);
    };
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_reqwest: Client = reqwest::Client::new();
    //process(client_reqwest).await
   // proc2();

    
    Ok(())


    
}


pub fn add(a: i32, b:i32) -> i32 {
    a + b
}

pub fn bad_add(a: i32, b: i32) -> i32{
    a - b
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
        let vect = read_lines(filename);
        let line0 = vect[0].clone();
        assert_eq!(etalon, line0);
    }

    #[test]
    fn test_read_webbhook(){
        let test_webhook = http_test::get_webhook_(WEBHOOK_FILENAME_TEST);
        let etalon = "http://google.com";
        assert_eq!(etalon, test_webhook);

    }
}





