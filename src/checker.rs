
use std::println;
use crate::normalize_tel;
use crate::ADDITIONAL_FIELDS;
use crate::{EFES, Employee, http_test, Pack};
use crate::ADDITIONAL_FIELDS::WORK_POSITION;
use crate::ADD_DUMP;
pub fn get_work_pos(input: String) -> String {
    if let Some(pos) = input.find('=') {
        input[..pos].to_string()
    } else {
        EFES.to_string()
    }
}


pub fn get_f_(input: String) -> String {
    if let Some(pos) = input.find('=') {
        let after = &input[pos + 1..];
        return after.split_whitespace().next().unwrap_or("").to_string();
    } else
    {
        "" .to_string()
    }
}

pub fn check_work_pos_atom (input : &Employee, atom_string: String ) -> bool {
    let curr_work = input.map_add.get(&WORK_POSITION).expect("").to_string().replace("\"", "");
 //   println!("CURRENT WORKPOS ::+{}+\n",  curr_work);

    let res = input.map_add.get(&WORK_POSITION).expect("").to_string().replace("\"", "")  == get_work_pos(atom_string);
    if !res {
        println!("CURRENT WORKPOS ::+{}+\n",  curr_work);
    }
    res

}

//http_test::read_lines(filename: &str) -> Vec<String> 
pub fn get_info_str___(input: Vec<String>, f: String) -> String {
    for str in input  {
        if str.contains(&f){

            return str;
        }
    }
    return "".to_string();
}


pub fn get_info_str(input: &mut Vec<String>, f: &str) -> String {
    // Ищем индекс первого элемента, содержащего подстроку f
    if let Some(pos) = input.iter().position(|s| s.contains(f)) {
        // Удаляем элемент по индексу и возвращаем его
        // swap_remove — быстрее, но меняет порядок; remove — сохраняет порядок, но дороже
        input.swap_remove(pos)
    } else {
        "".to_string()
    }    
}



pub fn check_pack(pack: Pack, filename: &str, supress_sucess: bool) {
    let mut info_strs = http_test::read_lines_utf8(filename);


    for info_str in info_strs{
        println!("\n\n================================================================================"); 
        let f = get_f_(info_str.clone());
        let mass: Vec<&Employee> = pack.pack.iter().filter(|&q|  q.last_name == f ).collect();
        if mass.is_empty()  {
            println!("RECORD FOR {} not found in pack", info_str.clone());
            continue;
        }  
        let emp = mass.get(0).unwrap();
        let compared = check_work_pos_atom(&emp, info_str.to_string());
        if compared {
            if supress_sucess {
                continue;
            }
            println!("PROCESS ::{} {} {} {}", emp.name, emp.last_name, emp.middle_name,  f.clone()); 

            println!("ALLES GUTTE!");
        }
        else {
            println!("\n\nPROCESS ::{} {}", emp.name, f.clone()); 
            println!("INFO STR-> {}", info_str);
            println!("REQUIRED WORKPOS ::+{}+\n",  get_work_pos(info_str) );
        }

    }

}


pub fn check_phone(emp: &Employee, info_str: String)  -> bool {
    let tel_from_info_str = normalize_tel(extract_tel_n(info_str.clone()).unwrap(), EFES.to_string());
    let e_tel = emp.map_add.get(&crate::ADDITIONAL_FIELDS::PERSONAL_MOBILE).unwrap().to_string();
    let e_tel2 = normalize_tel(e_tel, EFES.to_string());
   // let emp_tel = normalize_tel(emp.map_add.get(&crate::ADDITIONAL_FIELDS::PERSONAL_MOBILE).unwrap().to_string(), EFES.to_string());
 //   println!("first::{}\nsecond::{}", tel_from_info_str, e_tel);

    let res = tel_from_info_str == e_tel2 ;//emp_tel
    if !res {
        println!("PROCESS::{}\n, from binary::{}\n, from csv::{}", info_str, e_tel2, tel_from_info_str);

        println!("STRING WRONG::{}", info_str);
    }
    res
}

pub fn get_f_4_phone_check(input: String ) ->  String {
    let first_delim = input.find(";").unwrap();  
    let slice = input.as_str()[first_delim + 1..].to_string();
    slice.split_whitespace().next().unwrap_or("").to_string()

}

pub fn get_n_4_phone_check(input: String) -> String {
    let first_delim = input.find(";").unwrap();  
    let slice = input.as_str()[first_delim + 1..].to_string();
    slice.split_whitespace().nth(1).unwrap_or("").to_string()
}


pub fn check_pack_phones(pack: Pack, filename: &str) {
    let mut info_strs = http_test::read_lines_utf8(filename);


    for info_str in info_strs{
        println!("=========================================================================================");
        let f =  get_f_4_phone_check(info_str.clone());
        let n =  get_n_4_phone_check(info_str.clone());
        let mass: Vec<&Employee> = pack.pack.iter().filter(|&q| 
            q.last_name == f && 
            q.name == n &&
            q.map_add.get(&ADDITIONAL_FIELDS::ACTIVE).unwrap().to_string() == "true".to_string() 
        ).collect();

        if mass.is_empty(){
            println!("NOT FOUND IN BINARY::{}", info_str);
            continue;
        }
        let emp = mass.get(0).unwrap();
        let f =  get_f_4_phone_check(info_str.to_string());
    //    assert_eq!("Абрамов".to_string(), f);
        let mass: Vec<&Employee> = pack.pack.iter().filter(|&q|  q.last_name == f ).collect();
        let emp = mass.get(0).unwrap();
        check_phone(&emp, info_str);       
        
    }

}


fn extract_tel_n(input: String) -> Option<String>  {
  input
        .split(';')
        .nth(7)
        .map(|s: &str| s.trim().to_string())
    }

#[cfg(test)]
mod tests {
use crate::ADD_DUMP;

use std::assert_eq;

use super::*;

use crate::normalize_tel;
    #[test]
    fn test_one_4_phone(){
        let mut pack = Pack::deserialize_from_file(ADD_DUMP).expect("msg");
        let info: String = "1;Абрамов Антон Александрович;инженер ПТО;Операционный департпмент ;08;январь;1990;+7 988 591 84 07;Стройпро".to_string();
        let f =  get_f_4_phone_check(info.to_string());
        let n =  get_n_4_phone_check(info.to_string());
        assert_eq!("Абрамов".to_string(), f);
        assert_eq!("Антон".to_string(), n);
        let mass: Vec<&Employee> = pack.pack.iter().filter(|&q|  q.last_name == f ).collect();
        let emp = mass.get(0).unwrap();
        assert_eq!(normalize_tel(extract_tel_n(info.clone()).unwrap(), EFES.to_string()), "9885918407");
        assert_eq!(true, check_phone(&emp, info));

    }


    #[test]
    fn test_packs_4_phone(){
        let mut pack = Pack::deserialize_from_file(ADD_DUMP).expect("msg");
        let filename_info =  "ph_dump.csv";
        check_pack_phones(pack, filename_info);
    }


    #[test]
    fn test_ocr() {
        let input = "Руководитель отдела  по внутренней отделке=Шагинян А.	";
        assert_eq!("Руководитель отдела  по внутренней отделке", get_work_pos(input.to_string()));
        assert_eq!("Шагинян", get_f_(input.to_string()));
      
    }

    #[test]
    fn process_one_record() {
        let mut pack = Pack::deserialize_from_file(ADD_DUMP).expect("msg");
        let filename_info =  "workpos_test.info";
        check_pack(pack, filename_info, true);

    }

    #[test]
    fn process_pack_record() {
        let mut pack = Pack::deserialize_from_file(ADD_DUMP).expect("msg");


    //             let mass: Vec<&Employee> = pack.pack.iter().filter(|&q|  q.last_name == "Паршина" ).collect();

    // //    let data_pack = pack.pack.iter().filter(|&a| a.last_name != "" ).collect();
        let filename_info =  "workpos.info";
        check_pack(pack, filename_info, true);
    }


    use crate::web_accelerator::open_browser_and_wait;
    // #[tokio::test]
    // async fn force_login_glpi_test(){
    //     let mut pack = Pack::deserialize_from_file(ADD_DUMP).expect("msg");
    //     let mut counter = 1;
    //     let base_url =  "https://glpi.relits.ru/chat";
    //     pack.pack.iter().filter(|&q|  q.map_add.get(&ADDITIONAL_FIELDS::ACTIVE).unwrap() == "true" && 
    //                                              q.map_add.get(&ADDITIONAL_FIELDS::UF_DEPARTMENT).unwrap()!="null" &&
    //                                              q.map_add.get(&ADDITIONAL_FIELDS::EMAIL).unwrap().contains("@")).for_each(|f|  {
    //                                                 counter += 1;                                                    
    //                                                 println!("{}      EMPS->{}", counter, f._to_string());
    //                                                 open_browser_and_wait(base_url, f.map_add.get(&ADDITIONAL_FIELDS::EMAIL)).await;
    //                                             });

    // }

   // #[tokio::test]
    async fn force_login_glpi_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut pack = Pack::deserialize_from_file(ADD_DUMP).expect("Ошибка загрузки данных");
    let mut counter = 1;
    let base_url = "https://glpi.relits.ru";////chat";
 //https://glpi.relits.ru?user=nmaksimova%40relits.ru
    for emp in pack.pack.iter() {
        // Фильтруем по условиям
        let active = emp.map_add.get(&ADDITIONAL_FIELDS::ACTIVE).unwrap();
        if active != "true" {
            continue;
        }

        let dept = emp.map_add.get(&ADDITIONAL_FIELDS::UF_DEPARTMENT).unwrap();
        if dept == "null" {
            continue;
        }

        let email = emp.map_add.get(&ADDITIONAL_FIELDS::EMAIL).unwrap();
        if !email.contains('@') {
            continue;
        }

        counter += 1;
        println!("{}      EMPS->{}", counter, emp._to_string());

        // Асинхронный вызов с обработкой ошибок
        if let Err(e) = open_browser_and_wait(base_url, email).await {
            eprintln!("Ошибка для {}: {}", email, e);
            // Если нужно прервать тест при первой ошибке – используйте ?
            // return Err(e.into());
        }
    }

    Ok(())
}

    
}
