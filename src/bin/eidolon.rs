use std::fs;
use std::io::prelude::*;
use std::io::Read;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::env;
use std::process::Command;
fn main() {
    interpet_args();
}
fn interpet_args() {
    //Matches arguments to their relevant functions
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    match command.as_ref() {
        "update" => update_steam(),
        "add" => add_game(&args[2], &args[3]),
        "rm" => rm_game(&args[2]),
        "help" => print_help(),
        "menu" => show_menu(),
        "import" => import(&args[2]),
        _ => println!("Unknown command. eidolon help for commands."),
    }
}
fn import(dir: &str) {
    let mut path = env::current_dir().unwrap();
    let entry_format = &dir.split("/").collect::<Vec<&str>>();
    let real_dir: String = String::from(entry_format[entry_format.len() - 1]);
    let procname = create_procname(&real_dir);
    println!("Creating game registry named {}.", procname);
    path.push(&dir);
    let entries = fs::read_dir(&path).expect("Can't read in game folder");
    let mut found_game = String::new();
    for entry in entries {
        let base = entry
            .unwrap()
            .path()
            .into_os_string()
            .into_string()
            .unwrap();
        let entry_format = base.split("/").collect::<Vec<&str>>();
        let total = entry_format.len() - 1;
        let entry: String = String::from(entry_format[total]);
        let mut found = true;
        if entry.find(".x86_64").is_some() {
            println!("Found a unity exe. Assuming it is game");
        } else if entry.find("start.sh").is_some() {
            println!("Found a GOG game. Assuming it is game");
        } else if entry.find("start").is_some() {
            println!("Found older nicohman game exe. Assuming it is game");
        } else {
            found = false;
        }
        if found == true {
            found_game = entry.to_owned();
        }
    }
    if found_game.len() > 0 {
        add_game(
            &procname,
            &(path.into_os_string().into_string().unwrap() + "/" + &found_game),
        );
    } else {
        println!(
            "Could not find recognizable game exe. You will have to manually specify using eidolon add [name] [exe]"
        );

    }
}
fn show_menu() {
    //Creates a list of all installed games, then pipes them to a dmenu rofi
    Command::new("sh")
        .arg("-c")
        .arg("~/.config/eidolon/games/$(ls  --format=single-column ~/.config/eidolon/games | tr -d / | rofi -theme sidebar -mesg 'eidolon game:' -p '> ' -dmenu)/start")
        .spawn()
        .expect("Failed to run rofi.");
}
fn print_help() {
    println!("Commands:");
    println!("update : updates registry with installed steam games");
    println!("add [name] [file] : adds game to registry");
    println!("rm [name] : removes game from registry");
    println!("menu : shows game menu");
    println!("import [dir] : attempts to import in game directory just from name of location.");
    println!("help : show this screen");
}
fn rm_game(name: &str) {
    //Removes folder of named game
    let res = fs::remove_dir_all(
        String::from("/home/nicohman/.config/eidolon/games/") +
            create_procname(name).as_ref(),
    );
    if res.is_ok() {
        println!("Game removed!");

    } else {
        println!("Game did not exist. So, removed?");
    }
}
fn add_game(name: &str, exe: &str) {
    //Registers executable file as game with given name
    let mut path = env::current_dir().unwrap();
    path.push(exe);
    //Adds pwd to exe path
    let name = create_procname(name);
    let entries = fs::read_dir("/home/nicohman/.config/eidolon/games")
        .expect("Can't read in games");
    let mut can_be_used = true;
    for entry in entries {
        let base = entry
            .unwrap()
            .path()
            .into_os_string()
            .into_string()
            .unwrap();
        let entry_format = base.split("/").collect::<Vec<&str>>();
        let total = entry_format.len() - 1;
        let entry: String = String::from(entry_format[total]);
        //Checks to ensure that game is not already registered with selected name
        if entry == name {
            println!("Game already registered with that name. Pick another");
            can_be_used = false;
        }
    }
    if can_be_used == true {
        println!("Creating shortcut for {:?} with a name of {}", path, name);
        let res = fs::create_dir(
            String::from("/home/nicohman/.config/eidolon/games/") + &name,
        );
        if res.is_ok() {
            //Writes executable file in correct folder with simple bash script to run the linked executable
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .mode(0o770)
                .open(
                    String::from("/home/nicohman/.config/eidolon/games/") + &name + "/start",
                )
                .unwrap();
            file.write_all(
                String::from(
                    String::from("#!/bin/bash\n") + &(path.into_os_string().into_string().unwrap()),
                ).as_bytes(),
            ).expect("Could not write game registry");
        }
    }
}
fn update_steam() {
    //Iterates through steam directories for installed steam games and creates registrations for all
    let dirs: [String; 2] = [
        String::from("/home/nicohman/steam_games/steamapps/steamapps"),
        String::from("/games/steam/steamapps"),
    ];
    let dir = "/home/nicohman/.config/eidolon/games";
    for x in &dirs {
        let entries = fs::read_dir(x.to_owned() + "/common").expect("Can't read in games");
        for entry in entries {
            let base = entry
                .expect("unable to get entry")
                .path()
                .into_os_string()
                .into_string()
                .unwrap();
            let entry_format = base.split("/").collect::<Vec<&str>>();
            let total = entry_format.len() - 1;
            let entry: String = String::from(entry_format[total]);
            //Calls search games to get appid and proper name to make the script
            let results = search_games(entry, x.to_owned());
            if results.1 == "name" {
                println!("Could not find game as refrenced by .vdf");
            } else {
                let procname = create_procname(&results.1);
                let res = fs::create_dir(String::from(dir) + "/" + &procname);
                if res.is_ok() {
                    println!("Made shortcut for {}", &results.1);
                    let mut file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .mode(0o770)
                        .open(String::from(dir) + "/" + &procname + "/start")
                        .unwrap();
                    file.write_all(
                        (String::from("#!/bin/bash\nsteam 'steam://rungameid/") + &results.0 + "'")
                            .as_bytes(),
                    ).expect("Couldn't create game registration");
                } else {
                    println!(
                        "{}",
                        String::from("A shortcut has already been made for ") + &results.1
                    );
                }
            }
        }
    }

}
fn create_procname(rawname: &str) -> (String) {
    //Formats game name into nice-looking underscored name
    let procname: String = String::from(
        String::from(rawname)
            .chars()
            .map(|x| match x {
                '-' => '_',
                ' ' => '_',
                _ => x,
            })
            .collect::<String>()
            .to_lowercase()
            .trim(),
    );
    let chars = procname.chars();
    let mut procname = String::new();
    for char in chars {
        if char == '\'' {
        } else if char == '™' {
        } else if char == ':' {
        } else {
            procname.push(char);
        }
    }
    return procname;
}
fn search_games(rawname: String, steamdir: String) -> (String, String, String) {
    //Searches given steam game directory for installed game with a directory name of [rawname]
    let entries = fs::read_dir(steamdir).expect("Can't read installed steam games");
    for entry in entries {
        let entry = entry.unwrap().path();
        let new_entry = entry.into_os_string().into_string().unwrap();
        if new_entry.find("appmanifest").is_some() {
            let mut f = fs::File::open(&new_entry).expect("Could open appmanifest");
            let mut contents = String::new();
            f.read_to_string(&mut contents).expect(
                "Could not read appmanifest",
            );
            unsafe {
                //Slices out the game data from the appmanifest. Will break the instant steam changes appmanifest formats
                let outname = contents.slice_unchecked(
                    contents.find("installdir").unwrap() + 14,
                    contents.find("LastUpdated").unwrap() - 4,
                );
                let appid = contents.slice_unchecked(
                    contents.find("appid").unwrap() + 9,
                    contents.find("Universe").unwrap() - 4,
                );
                let name = contents.slice_unchecked(
                    contents.find("name").unwrap() + 8,
                    contents.find("StateFlags").unwrap() - 4,
                );
                if outname == rawname {
                    return (
                        String::from(appid),
                        String::from(name),
                        String::from(outname),
                    );
                }
            }
        }
    }
    //Return generic defaults
    return (
        String::from("appid"),
        String::from("name"),
        String::from("outname"),
    );
}
