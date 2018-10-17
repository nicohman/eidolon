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
mod args;
use args::*;
mod eidolon;
use structopt::StructOpt;
use eidolon::*;
fn main() {
    check_games();
    if startup() {
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
                    },
                    Add { name, path, wine } => add_game_p(&name, &path, wine),
                    Rm { game } => rm_game(&game),
                    Menu {} => show_menu(&config.menu_command),
                    List {} => list_games(),
                    Run { name } => run_game(&name),
                    Update {} => {
                        update_steam(config.steam_dirs);
                        update_lutris();
                        update_itch();
                    }
                }
}
fn show_menu(menu_command: &str) {
    //Creates a list of all installed games, then pipes them to a dmenu rofi
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
                run_game(&String::from_utf8_lossy(&output.stdout).trim());
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
