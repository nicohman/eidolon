use std::fs;
use std::env;
use std::process::Command;
use std::fs::DirEntry;
use std::io;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;
mod eid_lib;
use eid_lib::eidolon;
fn main() {
    eidolon::check_games();
    interpret_args();
}
fn interpret_args() {
    if eidolon::startup() {
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
                let menu_command = config.menu_command;
                let steam_dirs = config.steam_dirs;
                let prefix_command = config.prefix_command;
                match command.as_ref() {
                    "update" => {
                        eidolon::update_steam(steam_dirs);
                        eidolon::update_lutris();
                        eidolon::update_itch();
                    },
                    "version" => print_version(),
                    "add" => eidolon::add_game_p(&args[2], &args[3], false),
                    "rm" => eidolon::rm_game(&args[2]),
                    "help" => print_help(),
                    "menu" => show_menu(menu_command, prefix_command),
                    "import" => eidolon::import(&args[2]),
                    "list" => eidolon::list_games(),
                    "imports" => eidolon::imports(&args[2]),
                    "run" => eidolon::run_game(&args[2]),
                    "wine_add" => eidolon::add_game_p(&args[2], &args[3], true),
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
fn print_version() {
    println!("Eidolon Game Launcher v1.4.0");
    println!("Created by nicohman");
    println!("For support, file an issue at https://github.com/nicohman/eidolon or email nicohman@disroot.org");
}
fn show_menu(menu_command: String, prefix_command:String) {
    //Creates a list of all installed games, then pipes them to a dmenu rofi
    let mut entries = eidolon::get_games();
    entries.sort_by(|a, b| a.cmp(&b));
    let mut game_list = String::new();
    for entry in entries {
        //let entry = proc_path(entry);
        game_list.push_str(&entry);
        game_list.push_str("\n");
    }
    game_list = String::from(game_list.trim());
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
                eidolon::run_game(&String::from_utf8_lossy(&output.stdout).trim());
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
    println!("update : updates registry with installed steam games and lutris wine games");
    println!("add [name] [file] : adds game to registry");
    println!("list : lists installed games");
    println!("rm [name] : removes game from registry");
    println!("menu : shows game menu");
    println!("run [name] : runs named game");
    println!("import [dir] : attempts to import in game directory just from name of location.");
    println!("imports [dir] : imports in all game directories within given directory");
    println!("wine_add [name] [.exe] : adds windows exe to be run under wine to the registry");
    println!("version : displays the current eidolon version and contact info");
    println!("help : show this screen");
}
