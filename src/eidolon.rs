extern crate regex;
use butlerd::Butler;
use regex::Regex;
use serde_json;
use std::env;
use std::fs;
use std::fs::DirEntry;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::Read;
use std::io::{Error, ErrorKind};
use std::process::Command;
fn gd() -> String {
    return get_home() + "/.config/eidolon/games/";
}
fn default_blocked() -> Vec<String> {
    vec![
        "steamworks_common_redistributables".to_string(),
        "proton_3.7".to_string(),
        "proton_3.7_beta".to_string(),
    ]
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub steam_dirs: Vec<String>,
    pub menu_command: String,
    pub prefix_command: String,
    #[serde(default = "default_blocked")]
    pub blocked: Vec<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    command: String,
    pname: String,
    name: String,
    typeg: String,
}
pub struct OldConfig {
    pub steam_dirs: Vec<String>,
    pub menu_command: String,
    pub prefix_command: String,
}
pub struct SearchResult {
    pub appid: String,
    pub name :String,
    pub outname: String
}
impl Config {
    /// Default config
    fn default() -> Config {
        Config {
            steam_dirs: vec!["$HOME/.local/share/Steam/steamapps".to_string()],
            menu_command: "rofi -theme sidebar -mesg 'eidolon game:' -p '> ' -dmenu".to_string(),
            prefix_command: "".to_string(),
            blocked: default_blocked(),
        }
    }
}
/// Checks game registry for old-formatted games, and attempts to convert them
pub fn check_games() {
    let games = get_games();
    for game in games {
        let m = fs::metadata(gd() + &game);
        if m.is_ok() {
            if m.unwrap().is_dir() {
                let mut command = String::new();
                fs::File::open(gd() + &game + "/start")
                    .unwrap()
                    .read_to_string(&mut command)
                    .unwrap();
                let mut commandl = command.lines();
                commandl.next().unwrap();
                let mut command = commandl.next().unwrap().to_string();
                let mut typeg = String::from("exe");
                if command.contains("steam://rungameid") {
                    typeg = String::from("steam");
                } else if command.contains("lutris:rungameid") {
                    typeg = String::from("lutris");
                }
                let mut games = Game {
                    name: game.clone(),
                    pname: game.clone(),
                    command: command,
                    typeg: typeg,
                };
                add_game(games);
                println!("Converted {}", game);
                fs::remove_dir_all(gd() + &game).unwrap();
            }
        }
    }
}
/// Adds a given and configured game to registry
pub fn add_game(game: Game) {
    if fs::metadata(gd() + &game.name + ".json").is_ok() {
        println!("  Already made a shortcut for {}", game.pname);
    } else {
        let mut ok = true;
        let blocked = get_config().blocked;
        for block in blocked {
            if game.name == block {
                ok = false;
            }
        }
        if ok {
            OpenOptions::new()
                .create(true)
                .write(true)
                .open(gd() + &game.name + ".json")
                .unwrap()
                .write_all(serde_json::to_string(&game).unwrap().as_bytes())
                .unwrap();
            println!("  Made shortcut for {}", game.pname);
        } else {
            println!("  {} is in your blocked list", game.pname);
        }
    }
}
/// Loads vec of all installed games
pub fn get_games() -> Vec<String> {
    fs::read_dir(gd())
        .expect("Can't read in games")
        .collect::<Vec<io::Result<DirEntry>>>()
        .into_iter()
        .map(|entry| {
            entry
                .unwrap()
                .file_name()
                .into_string()
                .unwrap()
                .replace(".json", "")
        })
        .collect::<Vec<String>>()
}
/// Prints currently installed games
pub fn list_games() {
    println!("Currently registered games:");
    let entries = get_games();
    println!("Name - Procname - Type");
    for entry in entries {
        let game = read_game(entry).unwrap();
        println!("{} - {} - {}", game.pname, game.name, game.typeg);
    }
}
/// Fetches lutris wine games
pub fn get_lutris() -> Result<Vec<(String, String)>, io::Error> {
    let games = Command::new("lutris").arg("-l").output();
    if games.is_ok() {
        let games = games.unwrap();
        if games.status.success() {
            let games_list = String::from_utf8_lossy(&games.stdout);
            Ok(games_list
                .lines()
                .filter(|x| x.find("wine").is_some())
                .map(|x| {
                    let n = x.split("|").collect::<Vec<&str>>();
                    (String::from(n[0].trim()), String::from(n[1].trim()))
                })
                .collect::<Vec<(String, String)>>())
        } else {
            Err(Error::new(ErrorKind::NotFound, "Lutris not installed"))
        }
    } else {
        Err(Error::new(ErrorKind::NotFound, "Lutris not installed"))
    }
}
/// Searches itch.io games and adds them to game registry
pub fn update_itch() {
    let btest = Butler::new();
    if btest.is_ok() {
        let mut already = get_games()
            .iter_mut()
            .filter(|x| {
                let read = read_game(x.to_string()).unwrap();
                &read.typeg == "itch"
            })
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        println!(">> Reading in itch.io games");
        let butler = btest.expect("Couldn't start butler daemon");
        let caves = butler.fetchall().unwrap();
        for cave in caves {
            let game = cave.game;
            let name = game.title;
            let procname = create_procname(&name.clone());
            let g = Game {
                pname: name.clone(),
                name: procname.clone(),
                command: cave.id,
                typeg: "itch".to_string(),
            };
            add_game(g);
            let mut i = 0;
            while i < already.len() {
                if already[i] == procname {
                    already.remove(i);
                }
                i += 1;
            }
        }
        for left in already {
            println!("{} has been uninstalled. Removing from registry.", left);
            rm_game(&left);
        }
    } else {
        println!("Itch.io client not installed!");
    }
}
/// Adds lutris wine games from get_lutris
pub fn update_lutris() {
    let lut = get_lutris();
    if lut.is_err() {
        println!(">> No wine games found in lutris, or lutris not installed");
    } else {
        println!(">> Reading in lutris wine games");
        for game in lut.unwrap() {
            let procname = create_procname(&game.1);
            let pname = game.1.clone();
            let command = String::from("lutris lutris:rungameid/") + &game.0;
            let g = Game {
                pname: pname.clone(),
                name: procname,
                command: command,
                typeg: "lutris".to_string(),
            };
            add_game(g);
        }
    }
}
/// Runs a registered game, given name
pub fn run_game(name: &str) {
    let proced = create_procname(name);
    let g = read_game(proced);
    if g.is_ok() {
        let g = g.unwrap();
        if &g.typeg != "itch" {
            Command::new("sh")
                .arg("-c")
                .arg(g.command)
                .output()
                .expect("Couldn't run selected game!");
        } else {
            let butler = Butler::new().expect("Has butler been uninstalled?");
            butler.launch_game(&g.command);
        }
    } else {
        println!("Couldn't find that game installed. Maybe you misspelled something?");
    }
}
/// Converts pre-v1.2.7 config to JSON config
pub fn convert_config() {
    let old = get_config_old();
    let conf = Config {
        steam_dirs: old
            .steam_dirs
            .into_iter()
            .map(|x| String::from(x))
            .collect::<Vec<String>>(),
        menu_command: String::from(old.menu_command),
        prefix_command: String::from(old.prefix_command),
        blocked: default_blocked(),
    };
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(get_home() + "/.config/eidolon/config.json")
        .unwrap()
        .write_all(serde_json::to_string(&conf).unwrap().as_bytes())
        .unwrap();
    fs::remove_file(get_home() + "/.config/eidolon/config").unwrap();
}
/// Loads in eidolon config file
pub fn get_config() -> Config {
    let mut conf_s = String::new();
    fs::File::open(get_home() + "/.config/eidolon/config.json")
        .expect("Couldn't read config")
        .read_to_string(&mut conf_s)
        .unwrap();
    let mut config: Config = serde_json::from_str(&conf_s).unwrap();
    let fixed = config.steam_dirs.into_iter();
    config.steam_dirs = fixed
        .map(|x| {
            String::from(
                x.as_str()
                    .replace("$HOME", &get_home())
                    .replace("~", &get_home()),
            )
        })
        .collect::<Vec<String>>();
    config
}
/// This parses the config format that eidolon used prior to v1.2.7. This is used to convert the old format into the new JSON-based format when it is detected.
pub fn get_config_old() -> OldConfig {
    let mut conf = String::new();
    fs::File::open(get_home() + "/.config/eidolon/config")
        .expect("Couldn't read config")
        .read_to_string(&mut conf)
        .expect("Couldn't read in config");
    let mut conf = conf.lines();
    let steam_dirs = conf.next().unwrap();
    let steam_vec = Regex::new(r"(?:([^\|\s]+)\|)")
        .expect("Couldn't create regex")
        .captures_iter(steam_dirs)
        .map(|x| String::from(x.get(1).unwrap().as_str().replace("$HOME", &get_home())))
        .collect::<Vec<String>>();
    let menu_command_base = String::from(conf.next().unwrap());
    let prefix_command_bool = conf.next();
    let mut prefix_command: &str;
    if prefix_command_bool.is_some() {
        prefix_command = prefix_command_bool.unwrap();
        prefix_command = prefix_command.split('|').collect::<Vec<&str>>()[1];
    } else {
        prefix_command = " "
    }
    let menu_command = menu_command_base.split('|').collect::<Vec<&str>>()[1];
    OldConfig {
        steam_dirs: steam_vec,
        menu_command: String::from(menu_command),
        prefix_command: String::from(prefix_command),
    }
}
/// Intializes basic directories and config for the first use
pub fn init() {
    println!("Beginning config init");
    if fs::metadata(get_home() + "/.config").is_err() {
        fs::create_dir(get_home() + "/.config").expect("Couldn't create config directory");
    }
    fs::create_dir(get_home() + "/.config/eidolon").expect("Couldn't create eidolon directory");
    fs::create_dir(get_home() + "/.config/eidolon/games").expect("Couldn't create games directory");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(get_home() + "/.config/eidolon/config.json")
        .unwrap();
    file.write_all(
        serde_json::to_string(&Config::default())
            .unwrap()
            .as_bytes(),
    ).unwrap();
    println!("Correctly initialized base config.");
}
/// Iterates through directory and imports each child directory
pub fn imports(dir: &str) {
    let entries = fs::read_dir(&dir).expect("Can't read in folder of games");
    println!("Reading in directory: {}", &dir);
    for entry in entries {
        let entry = proc_path(entry.unwrap());
        println!("Attempting import on {}", &entry);
        import(&entry);
        println!("Finished attempted import on {}", &entry);
    }
}
/// Scans a directory for common game formats and adds them.
pub fn import(dir: &str) {
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
        add_game_p(
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
/// Removes folder of named game
pub fn rm_game(name: &str) {
    let res = fs::remove_file(String::from(gd() + create_procname(name).as_ref()) + ".json");
    if res.is_ok() {
        println!("Game removed!");
    } else {
        println!("Game did not exist. So, removed?");
    }
}
/// Registers executable file as game with given name. Wine argguement indicates whether or not to run this game under wine
pub fn add_game_p(name: &str, exe: &str, wine: bool) {
    let mut path = env::current_dir().unwrap();
    path.push(exe);
    //Adds pwd to exe path
    let pname = name.clone();
    let name = create_procname(name);
    if fs::metadata(gd() + &name + ".json").is_ok() {
        println!("A shortcut has already been made for {}", pname);
    } else {
        println!("Creating shortcut for {:?} with a name of {}", path, name);
        let mut start = String::from("");
        if wine {
            let mut winestr = String::from("wine ");
            if exe.to_lowercase().contains(".lnk") {
                winestr = winestr + "start ";
            }
            start.push_str(&winestr);
        }
        let command = String::from(
            start
                + &(path
                    .into_os_string()
                    .into_string()
                    .unwrap()
                    .replace(" ", "\\ ")),
        );
        let game = Game {
            pname: pname.to_string(),
            name: name,
            command: command,
            typeg: "exe".to_string(),
        };
        add_game(game);
    }
}
// /Iterates through steam directories for installed steam games and creates registrations for all
pub fn update_steam(dirs: Vec<String>) {
    let mut already = get_games();
    for x in &dirs {
        println!(">> Reading in steam library {}", &x);
        let entries = fs::read_dir(x.to_owned() + "/common")
            .expect("Can't read in games")
            .into_iter()
            .map(|x| proc_path(x.unwrap()));
        for entry in entries {
            //Calls search games to get appid and proper name to make the script
            let results = search_games(entry, x.to_owned());
            if results.is_some() {
                let results = results.unwrap();
                let procname = create_procname(&results.name);
                let pname = results.name.clone();
                let command = String::from("steam 'steam://rungameid/") + &results.appid + "'";
                let game = Game {
                    name: procname.clone(),
                    pname: pname.clone(),
                    command: command,
                    typeg: "steam".to_string(),
                };
                add_game(game);
                let mut i = 0;
                while i < already.len() {
                    if already[i] == procname {
                        already.remove(i);
                    }
                    i += 1;
                }
            }
        }
    }
    for al in already {
        let typeg = read_game(al.clone()).unwrap().typeg;
        if typeg == "steam" {
            println!("{} has been uninstalled. Removing game from registry.", al);
            rm_game(&al);
        }
    }
}
/// Reads in a game's info from a name
pub fn read_game(name: String) -> Result<Game, String> {
    if fs::metadata(gd() + &name + ".json").is_ok() {
        let mut stri = String::new();
        fs::File::open(gd() + &name + ".json")
            .unwrap()
            .read_to_string(&mut stri)
            .unwrap();
        let g: Game = serde_json::from_str(&stri).unwrap();
        return Ok(g);
    }
    return Err("No such game".to_string());
}
/// Formats game name into nice-looking underscored name for continuity with other names
pub fn create_procname(rawname: &str) -> String {
    let mut basename = String::from(rawname).to_lowercase();
    basename = String::from(basename.trim());
    let reg_white = Regex::new(r"-|\s").unwrap();
    let reg_special = Regex::new(r"'|™|:").unwrap();
    let white_formatted = reg_white.replace_all(&basename, "_");
    let total_formatted = reg_special.replace_all(&white_formatted, "");
    return String::from(total_formatted);
}
/// Searches given steam game directory for installed game with a directory name of [rawname]
pub fn search_games(rawname: String, steamdir: String) -> Option<SearchResult> {
    let entries = fs::read_dir(&steamdir).expect("Can't read installed steam games");
    for entry in entries {
        let entry = entry.unwrap().path();
        let new_entry = entry.into_os_string().into_string().unwrap();
        if new_entry.find("appmanifest").is_some() {
            let mut f = fs::File::open(&new_entry).expect("Couldn't open appmanifest");
            let mut contents = String::new();
            f.read_to_string(&mut contents)
                .expect("Could not read appmanifest");
            unsafe {
                if contents.find("installdir").is_some() {
                    //Slices out the game data from the appmanifest. Will break the instant steam changes appmanifest formats
                    let outname = contents.slice_unchecked(
                        contents
                            .find("installdir")
                            .expect("Couldn't find install dir") + 14,
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
                        return Some(SearchResult {
                            appid: String::from(appid),
                            name: String::from(name),
                            outname: String::from(outname),
                        });
                    }
                }
            }
        }
    }
    //Return none if nothing can be found
    return None;
}
pub fn startup() -> bool {
    if check_inited() {
        true
    } else {
        init();
        return true;
    }
}
/// Check if eidolon has been initialized prior to this run
pub fn check_inited() -> bool {
    if fs::metadata(get_home() + "/.config/eidolon").is_err() || fs::metadata(gd()).is_err() {
        false
    } else {
        if fs::metadata(get_home() + "/.config/eidolon/config").is_ok() {
            convert_config();
            true
        } else if fs::metadata(get_home() + "/.config/eidolon/config.json").is_ok() {
            true
        } else {
            false
        }
    }
}
/// Converts DirEntry into a fully processed file/directory name
pub fn proc_path(path: DirEntry) -> String {
    let base = path.file_name().into_string().unwrap();
    return base;
}
/// Gets current user's home directory
pub fn get_home() -> String {
    return String::from(env::home_dir().unwrap().to_str().unwrap());
}
