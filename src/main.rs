use std::process::Command;
extern crate butlerd;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate structopt;
pub mod args;
use args::*;
extern crate libeidolon;
use auto::*;
use config::*;
use games::*;
use libeidolon::*;
use structopt::StructOpt;
use std::fs::File;
use std::io::Write;
use std::fs;
fn main() {
    if startup() {
        check_games();
        interpret_args();
    }
}
fn interpret_args() {
    //Matches arguments to their relevant functions
    let a = Eidolon::from_args();
    use Eidolon::*;
    let config = get_config();
    match a {
        Import { path, multi } => {
            if multi {
                imports(&path);
            } else {
                import(&path);
            }
        }
        Add {
            name,
            path,
            wine,
            dolphin,
        } => {
            if !dolphin {
                add_game_p(&name, &path, wine);
            } else {
                let dgame = Game {
                    command: path,
                    pname: name.clone(),
                    name: helper::create_procname(&name),
                    typeg: GameType::Dolphin
                };
                add_game(dgame);
            }
        }
        Rm { game } => rm_game(&game),
        Menu {} => show_menu(&config.menu_command),
        List {} => list_games(),
        Run { name } => {
            run_game(&name);
        },
        Update {} => {
            update_steam(config.steam_dirs);
            update_lutris();
            update_itch();
        }
    }
}
fn show_menu(menu_command: &str) {
    use games::*;
    // Creates a list of all installed games, then pipes them to a dmenu rofi
    let mut entries = get_games();
    entries.sort_by(|a, b| a.cmp(&b));
    let mut game_list = String::new();
    for entry in entries {
        game_list.push_str(&entry);
        game_list.push_str("\n");
    }
    game_list = String::from(game_list.trim());
    if game_list.lines().count() <= 0 {
        println!("No games added. Either run eidolon update or add games manually.");
        notify("No games added. Either run eidolon update or add games manually.");
    } else {
        let output = Command::new("sh")
            .arg("-c")
            .arg(String::from("echo '") + &game_list + "' | " + &menu_command)
            .output()
            .expect("Failed to run menu.");
        let parsed_output = String::from_utf8_lossy(&output.stdout);
        if output.status.success() {
            if parsed_output.trim().chars().count() > 0 {
                let res  = run_game(&String::from_utf8_lossy(&output.stdout).trim());
                if res.is_err(){
                    let stderr = res.err().unwrap();
                    if &stderr.clone() != "Nonexistent"{
                        println!("Game crashed. Stderr: \n{}",stderr);
                        notify("Game crashed. Stderr written to /tmp/crash_eidolon.log.");
                        fs::remove_file("/tmp/crash_eidolon.log");
                        File::create("/tmp/crash_eidolon.log").unwrap().write_all(stderr.as_bytes()).expect("Couldn't write");
                    } else {
                        notify("Could not find game of that name.");
                    }
                }
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
fn notify(notification: &str){
    Command::new("notify-send").arg(String::from(notification)).output().expect("Couldn't send notification");
}
