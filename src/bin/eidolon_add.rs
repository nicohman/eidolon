use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
fn main() {
    let args: Vec<_> = env::args().collect();
    let name = &args[1];
    let exe = &args[2];
    let mut path = env::current_dir().expect("hi");
    path.push(exe);
    let name = create_procname(name);
    let entries = fs::read_dir("/home/nicohman/.config/eidolon/games").expect("cant read");
    let mut can_be_used = true;
    for entry in entries {
        let base = entry.expect("unable to get entry").path().into_os_string().into_string().expect("turn to string");
        let entry_format = base.split("/").collect::<Vec<&str>>();
        let total = entry_format.len() - 1;
        let entry:String = String::from(entry_format[total]);
        if entry == name {
            println!("Game already registered with that name. Pick another");
            can_be_used = false;
        }
    }
    if can_be_used == true {
        println!("Creating shortcut for {:?} with a name of {}", path, name);
        let res = fs::create_dir(String::from("/home/nicohman/.config/eidolon/games/")+&name);
        if res.is_ok() {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .mode(0o770)
                .open(String::from("/home/nicohman/.config/eidolon/games/")+&name+"/start")
                .unwrap();
            file.write_all(String::from(String::from("#!/bin/bash\n")+&(path.into_os_string().into_string().expect("cant convert"))).as_bytes()).expect("couldnt write");
        }
    }
}
fn create_procname(rawname:&str) -> (String) {
            let procname:String = String::from(String::from(rawname).chars().map(|x| match x {
            '-' => '_',
            ' ' => '_',
            _ => x}
        ).collect::<String>().to_lowercase().trim());
        let chars = procname.chars();
        let mut procname = String::new();
        for char in chars {
            if char == '\'' {
            
            } else if char == '™'{
            
            } else if char == ':'{
            
            } else {
                procname.push(char);
            }
        }
    return procname;
}
