
use std::println;

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

    // for emp in pack.pack {
    //     if info_strs.is_empty(){
    //         break;
    //     }
    //     println!("\n\n================================================================================"); 

    //     let fio_target  = &emp.last_name;
    //     let info_str = get_info_str(&mut info_strs, fio_target);
    //     if info_str.len() == 0 { 
    //        // println!("RECORD NOT FOUND in TEXT FILE     {} ", filename);
    //         continue;
    //     }

    //     let compared = check_work_pos_atom(&emp, info_str.to_string());


    //     if compared {
    //         if supress_sucess {
    //             continue;
    //         }
    //         println!("PROCESS ::{} {} {} {}", emp.name, emp.last_name, emp.middle_name,  fio_target.clone()); 

    //         println!("ALLES GUTTE!");
    //     }
    //     else {
    //         println!("\n\nPROCESS ::{} {}", emp.name, fio_target.clone()); 
    //         println!("INFO STR-> {}", info_str);
    //         println!("REQUIRED WORKPOS ::+{}+\n",  get_work_pos(info_str) );
    //     }
    // }

}

#[cfg(test)]
mod tests {
use crate::ADD_DUMP;

use std::assert_eq;

use super::*;

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

    
}
