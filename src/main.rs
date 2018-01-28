use std::fs;
use std::env;
use std::process::Command;
use std::fs::DirEntry;
use std::io;
extern crate regex;
mod eid_lib;
use eid_lib::eidolon;
fn main() {
    interpret_args();
}
fn interpret_args() {
    if fs::metadata(eidolon::get_home() + "/.config/eidolon").is_err() ||
        fs::metadata(eidolon::get_home() + "/.config/eidolon/config").is_err() ||
            fs::metadata(eidolon::get_home() + "/.config/eidolon/games").is_err()
            {
                eidolon::init();
            } else {
                //Matches arguments to their relevant functions
                let args: Vec<String> = env::args().collect();
                let command: &str;
                if args.len() < 2 {
                    command = "help";
                } else {
                    command = &args[1];
                    if !check_args_num(args.len() - 2, command.as_ref()){
                        println!("Not enough arguments for {}", &command);
                        ::std::process::exit(64);
                    }
                }
                let config = eidolon::get_config();
                let menu_command = config.1;
                let steam_dirs = config.0;
                let prefix_command = config.2;
                match command.as_ref() {
                    "update" => eidolon::update_steam(steam_dirs),
                    "add" => eidolon::add_game(&args[2], &args[3], false),
                    "rm" => eidolon::rm_game(&args[2]),
                    "help" => print_help(),
                    "menu" => show_menu(menu_command, prefix_command),
                    "import" => eidolon::import(&args[2]),
                    "list" => eidolon::list_games(),
                    "imports" => eidolon::imports(&args[2]),
                    "run" => eidolon::run_game(&args[2]),
                    "wine_add" => eidolon::add_game(&args[2], &args[3], true),
                    _ => println!("Unknown command. eidolon help for commands."),
                }
            }
}
fn check_args_num(num:usize, command:&str) -> bool {
    let need  = match command {
        "add" => 2,
        "rm" => 1,
        "import" => 1,
        "imports" => 1,
        "run" => 1,
        "wine_add" => 2,
        _ => 0,
    };
    if num < need {
        false
    } else {
        true
    }
}
fn show_menu(menu_command: String, prefix_command:String) {
    //Creates a list of all installed games, then pipes them to a dmenu rofi
    let mut entries = fs::read_dir(eidolon::get_home() + "/.config/eidolon/games")
        .expect("Can't read in games")
        .collect::<Vec<io::Result<DirEntry>>>()
        .into_iter()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<String>>();
    entries.sort_by(|a, b| a.cmp(&b));
    let mut game_list = String::new();
    for entry in entries {
        //let entry = proc_path(entry);
        game_list.push_str(&entry);
        game_list.push_str("\n");
    }
    if game_list.lines().count() <= 0 {
        println!("No games added. Either run eidolon update or add games manually.");
        Command::new("sh").arg("-c").arg("notify-send").arg(String::from("'No games added. Either run eidolon update or add games manually.'")).output().expect("Couldn't send notification");
    } else {
        let output = Command::new("sh")
            .arg("-c")
            .arg(String::from("echo '") + &game_list + "' | " + &menu_command)
            .output()
            .expect("Failed to run menu.");
        let parsed_output = String::from_utf8_lossy(&output.stdout);
        if output.status.success() {
            if parsed_output.trim().chars().count() > 0 {
                Command::new("sh").arg("-c").arg(prefix_command+"~/.config/eidolon/games/"+&String::from_utf8_lossy(&output.stdout).trim()+"/start").spawn().expect("Failed to start game");
            } else {
                println!("No game selected!");
            }
        } else {
            if parsed_output.trim().chars().count() > 0 {

                println!("Okay, something went wrong. Your menu command:\n{}\n doesn't work. If you're using the default, have you installed rofi?", &menu_command);
            } else {
                println!("No game selected!");
            }
        }
    }
}
fn print_help() {
    println!("Commands:");
    println!("update : updates registry with installed steam games");
    println!("add [name] [file] : adds game to registry");
    println!("list : lists installed games");
    println!("rm [name] : removes game from registry");
    println!("menu : shows game menu");
    println!("run [name] : runs named game");
    println!("import [dir] : attempts to import in game directory just from name of location.");
    println!("imports [dir] : imports in all game directories within given directory");
    println!("wine_add [name] [.exe] : adds windows exe to be run under wine to the registry");
    println!("help : show this screen");
}

