use std::fs;
use std::io::prelude::*;
use std::io::Read;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::env;
fn main() {
	interpet_args();
}
fn interpet_args() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    match command.as_ref() {
        "update" => update_steam(),
        "add" => add_game(&args[2], &args[3]),
        _ => println!("Unknown command"),
    }
}
fn add_game (name:&str, exe: &str) {
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
fn update_steam() {
    let dirs: [String;2] = [String::from("/home/nicohman/steam_games/steamapps/steamapps"), String::from("/games/steam/steamapps")];
    let dir = "/home/nicohman/.config/eidolon/games";
    for x in &dirs {
   // println!("{}",x.to_owned()+"/common");
    let entries = fs::read_dir(x.to_owned()+"/common").expect("cant read");
    for entry in entries {
        //println!("{:?}",entry);
        let base = entry.expect("unable to get entry").path().into_os_string().into_string().expect("turn to string");
        let entry_format = base.split("/").collect::<Vec<&str>>();
        let total = entry_format.len() - 1;
        let entry:String = String::from(entry_format[total]);
       // println!("{}",entry);
        let results = search_games(entry, x.to_owned());
        if results.1 == "name" {
        } else {
        let procname = create_procname(&results.1);
        //println!("{}",procname);
        let res = fs::create_dir(String::from(dir)+"/"+&procname);
        if res.is_ok() {
            //println!("Made shorcut for {}", &results.1);
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .mode(0o770)
                .open(String::from(dir)+"/"+&procname+"/start")
                .unwrap();
            file.write_all((String::from("#!/bin/bash\nsteam 'steam://rungameid/")+&results.0+"'").as_bytes()).expect("didn't write");
        } else {
            //println!("{}",res.err().unwrap());
            println!("{}",String::from("A shortcut has already been made for ") + &results.1);
        }
    }}
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
fn search_games(rawname: String, steamdir:String) -> (String, String, String) {
    let entries = fs::read_dir(steamdir).expect("cant read");
    for entry in entries {
        let entry = entry.expect("unable to get entry").path();
        let new_entry = entry.into_os_string().into_string().expect("turn to string");
        //println!("{:?}", new_entry);
        if new_entry.find("appmanifest").is_some() {
            //println!("APP"); 
            let mut f = fs::File::open( & new_entry).expect("no file");
            let mut contents = String::new();
            f.read_to_string( & mut contents).expect("wrong");
            //println!("Text:\n{} {}", contents, new_entry);
            unsafe {
                let outname = contents.slice_unchecked(contents.find("installdir").expect("pos1") + 14, contents.find("LastUpdated").unwrap() - 4);
                let appid = contents.slice_unchecked(contents.find("appid").unwrap() + 9, contents.find("Universe").unwrap() - 4);
                let name = contents.slice_unchecked(contents.find("name").unwrap() + 8, contents.find("StateFlags").unwrap() - 4);
                //println!("OUTNAME:{} APPID:{} NAME:{}", &outname, &appid, &name);
                if outname == rawname {
                    return (String::from(appid), String::from(name), String::from(outname));
                }
            }
        }
    }
    return (String::from("appid"), String::from("name"), String::from("outname"))
}
