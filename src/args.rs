#[derive(StructOpt, Debug)]
#[structopt(name = "eidolon")]
pub enum Eidolon {
    #[structopt(name = "rm", about = "Remove a game from the registry")]
    Rm { game: String },
    #[structopt(name = "add", about = "Adds selected file to registry")]
    Add {
        name: String,
        path: String,
        #[structopt(short = "w", long = "wine")]
        wine: bool,
    },
    #[structopt(name = "menu", about = "Show game menu")]
    Menu {},
    #[structopt(name = "import", about = "Attempts to import in game directory from dir path")]
    Import {
        path: String,
        #[structopt(short = "m", long = "multi")]
        multi: bool,
    },
    #[structopt(name = "list", about = "Lists installed games")]
    List {},
    #[structopt(name = "run", about = "Runs a game by name")]
    Run { name: String },
    #[structopt(name = "update",
                about = "Updates registry with installed steam, lutris wine, and itch games")]
    Update {},
}
