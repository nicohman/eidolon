use std::fs;
use std::io::prelude::*;
use std::io::Read;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::env;
use std::process::Command;
use std::fs::DirEntry;
use std::io;
extern crate regex;
use regex::Regex;
fn main() {
    interpret_args();
}
fn interpret_args() {
    if fs::metadata(get_home() + "/.config/eidolon").is_err() ||
        fs::metadata(get_home() + "/.config/eidolon/config").is_err() ||
            fs::metadata(get_home() + "/.config/eidolon/games").is_err()
            {
                init();
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
                let config = get_config();
                let menu_command = config.1;
                let steam_dirs = config.0;
                let prefix_command = config.2;
                match command.as_ref() {
                    "update" => update_steam(steam_dirs),
                    "add" => add_game(&args[2], &args[3], false),
                    "rm" => rm_game(&args[2]),
                    "help" => print_help(),
                    "menu" => show_menu(menu_command, prefix_command),
                    "import" => import(&args[2]),
                    "imports" => imports(&args[2]),
                    "run" => run_game(&args[2]),
                    "wine_add" => add_game(&args[2], &args[3], true),
                    _ => println!("Unknown command. eidolon help for commands."),
                }
            }
}
fn run_game(name:&str) {
    let proced = create_procname(name);
    if fs::metadata(get_home()+"/.config/eidolon/games/"+&proced).is_err() {
        println!("Couldn't find that game installed. Maybe you misspelled something?");
    } else {
        Command::new("sh").arg("-c").arg(get_home()+"/.config/eidolon/games/"+&proced+"/start").output().expect("Couldn't run selected game!");
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
fn get_config() -> (Vec<String>, String, String) {
    let mut conf = String::new();
    fs::File::open(get_home() + "/.config/eidolon/config")
        .expect("Couldn't read config")
        .read_to_string(&mut conf)
        .expect("Couldn't read in config");
    let mut conf = conf.lines();
    let steam_dirs = conf.next().unwrap();
    let dirs_pre = Regex::new(r"(\s*steam_dirs *: *\|(?:(?:[^\|\s]+)\|)+)").unwrap().captures(steam_dirs).unwrap().get(0).expect("Config file is incorrectly setup").as_str();
    let steam_vec = Regex::new(r"(?:([^\|\s]+)\|)").expect("Couldn't create regex").captures_iter(dirs_pre).map(|x| String::from(x.get(1).unwrap().as_str().replace("$HOME", &get_home()))).collect::<Vec<String>>();
    let menu_command_base = String::from(conf.next().unwrap());
    let prefix_command_bool = conf.next();
    let mut prefix_command:&str;
    if prefix_command_bool.is_some() {
        prefix_command = prefix_command_bool.unwrap();
        prefix_command = prefix_command.split('|').collect::<Vec<&str>>()[1];
    } else {
        prefix_command = " "
    }
    let menu_command = menu_command_base.split('|').collect::<Vec<&str>>()[1];
    (steam_vec, String::from(menu_command), String::from(prefix_command))
}
fn init() {
    println!("Beginning config init");
    if fs::metadata(get_home()+"/.config").is_err() {
        fs::create_dir(get_home()+"/.config").expect("Couldn't create config directory");
    }
    fs::create_dir(get_home() + "/.config/eidolon").expect("Couldn't create eidolon directory");
    fs::create_dir(get_home() + "/.config/eidolon/games").expect("Couldn't create games directory");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(get_home() + "/.config/eidolon/config")
        .unwrap();
    file.write_all(
        (String::from("steam_dirs: | $HOME/.local/share/Steam/steamapps |\nmenu_command: | rofi -theme sidebar -mesg 'eidolon game:' -p '> ' -dmenu |\nprefix_command: | |")).as_bytes(),
        ).unwrap();
    println!("Correctly initialized base config. Please run again to use eidolon.");
}
fn imports(dir: &str) {
    //Iterates through directory and imports each child directory
    let entries = fs::read_dir(&dir).expect("Can't read in folder of games");
    println!("Reading in directory: {}", &dir);
    for entry in entries {
        let entry = proc_path(entry.unwrap());
        println!("Attempting import on {}", &entry);
        import(&entry);
        println!("Finished attempted import on {}", &entry);
    }
}
fn import(dir: &str) {
    //Scans a directory for common game formats and adds them.
    let mut path = env::current_dir().unwrap();
    let entry_format = &dir.split("/").collect::<Vec<&str>>();
    let real_dir: String = String::from(entry_format[entry_format.len() - 1]);
    let procname = create_procname(&real_dir);
    println!("Creating game registry named {}.", procname);
    path.push(&dir);
    let entries = fs::read_dir(&path).expect("Can't read in game folder");
    let mut found_game = String::new();
    for entry in entries {
        let entry = proc_path(entry.unwrap());

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
            false,
            );
    } else {
        println!(
            "Could not find recognizable game exe. You will have to manually specify using eidolon add [name] [exe]"
            );

    }
}
fn show_menu(menu_command: String, prefix_command:String) {
    //Creates a list of all installed games, then pipes them to a dmenu rofi
    let mut entries = fs::read_dir(get_home() + "/.config/eidolon/games")
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
        if parsed_output.trim().chars().count() > 0 {
            Command::new("sh").arg("-c").arg(prefix_command+"~/.config/eidolon/games/"+&String::from_utf8_lossy(&output.stdout).trim()+"/start").spawn().expect("Failed to start game");
        } else {
            println!("No game selected!");
        }
    }
}
fn print_help() {
    println!("Commands:");
    println!("update : updates registry with installed steam games");
    println!("add [name] [file] : adds game to registry");
    println!("rm [name] : removes game from registry");
    println!("menu : shows game menu");
    println!("run [game] : runs [game] without the menu");
    println!("import [dir] : attempts to import in game directory just from name of location.");
    println!("imports [dir] : imports in all game directories within given directory");
    println!("wine_add [name] [.exe] : adds windows exe to be run under wine to the registry");
    println!("help : show this screen");
}
fn rm_game(name: &str) {
    //Removes folder of named game
    let res = fs::remove_dir_all(String::from(
            get_home() + "/.config/eidolon/games/" +
            create_procname(name).as_ref(),
            ));
    if res.is_ok() {
        println!("Game removed!");

    } else {
        println!("Game did not exist. So, removed?");
    }
}
fn add_game(name: &str, exe: &str, wine: bool) {
    //Registers executable file as game with given name
    let mut path = env::current_dir().unwrap();
    path.push(exe);

    //Adds pwd to exe path
    let name = create_procname(name);
    let entries = fs::read_dir(get_home() + "/.config/eidolon/games").expect("Can't read in games");
    let mut can_be_used = true;
    for entry in entries {
        let entry = proc_path(entry.unwrap());

        //Checks to ensure that game is not already registered with selected name
        if entry == name {
            println!("Game already registered with that name. Pick another");
            can_be_used = false;
        }
    }
    if can_be_used == true {
        println!("Creating shortcut for {:?} with a name of {}", path, name);
        let res = fs::create_dir(String::from(
                String::from(get_home() + "/.config/eidolon/games/") + &name,
                ));
        if res.is_ok() {
            //Writes executable file in correct folder with simple bash script to run the linked executable
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .mode(0o770)
                .open(
                    String::from(get_home() + "/.config/eidolon/games/") + &name + "/start",
                    )
                .unwrap();
            let mut start = String::from("#!/bin/bash\n");
            if wine {
                start.push_str("wine ");
            }
            file.write_all(
                String::from(
                    start +
                    &(path.into_os_string().into_string().unwrap().replace(
                            " ",
                            "\\ ",
                            )),
                            ).as_bytes(),
                            ).expect("Could not write game registry");
        }
    }
}
fn update_steam(dirs: Vec<String>) {
    //Iterates through steam directories for installed steam games and creates registrations for all
    for x in &dirs {
        println!(">> Reading in steam library {}", &x);
        let entries = fs::read_dir(x.to_owned() + "/common").expect("Can't read in games");
        for entry in entries {
            let entry = proc_path(entry.unwrap());
            //Calls search games to get appid and proper name to make the script
            let results = search_games(entry, x.to_owned());
            if results.1 == "name" {
                println!("Could not find game as refrenced by .vdf");
            } else {
                let procname = create_procname(&results.1);
                let res = fs::create_dir(get_home() + "/.config/eidolon/games" + "/" + &procname);
                if res.is_ok() {
                    println!("Made shortcut for {}", &results.1);
                    let mut file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .mode(0o770)
                        .open(
                            get_home() + "/.config/eidolon/games" + "/" + &procname + "/start",
                            )
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
    let mut basename = String::from(rawname).to_lowercase();
    basename = String::from(basename.trim());
    let reg_white = Regex::new(r"-|\s").unwrap();
    let reg_special = Regex::new(r"'|™|:").unwrap();
    let white_formatted = reg_white.replace_all(&basename, "_");
    let total_formatted = reg_special.replace_all(&white_formatted, "");
    return String::from(total_formatted);
}
fn search_games(rawname: String, steamdir: String) -> (String, String, String) {
    //Searches given steam game directory for installed game with a directory name of [rawname]
    let entries = fs::read_dir(&steamdir).expect("Can't read installed steam games");
    for entry in entries {
        let entry = entry.unwrap().path();
        let new_entry = entry.into_os_string().into_string().unwrap();
        if new_entry.find("appmanifest").is_some() {
            let mut f = fs::File::open(&new_entry).expect("Couldn't open appmanifest");
            let mut contents = String::new();
            f.read_to_string(&mut contents).expect(
                "Could not read appmanifest",
                );
            unsafe {
                if contents.find("installdir").is_some() {
                    //Slices out the game data from the appmanifest. Will break the instant steam changes appmanifest formats
                    let outname = contents.slice_unchecked(
                        contents.find("installdir").expect("Couldn't find install dir") + 14,
                        contents.find("LastUpdated").unwrap() - 4,
                        );
                    if outname == rawname {

                        let appid = contents.slice_unchecked(
                            contents.find("appid").unwrap() + 9,
                            contents.find("Universe").unwrap() - 4,
                            );
                        let name = contents.slice_unchecked(
                            contents.find("name").unwrap() + 8,
                            contents.find("StateFlags").unwrap() - 4,
                            );
                        return (
                            String::from(appid),
                            String::from(name),
                            String::from(outname),
                            );
                    }
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
fn proc_path(path: DirEntry) -> String {
    //Converts DirEntry into a fully processed file/directory name
    let base = path.file_name().into_string().unwrap();
    return base;
}
fn get_home() -> String {
    return String::from(env::home_dir().unwrap().to_str().unwrap());
}
