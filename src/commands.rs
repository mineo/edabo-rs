use clap::{Arg, ArgMatches, App, SubCommand};
use empd;
use serde_json;
use std::convert::From;
use std::path::PathBuf;
use types::*;
use xdg::BaseDirectories;

pub fn get_playlist_dir() -> Result<PathBuf, EdaboError> {
    BaseDirectories::with_prefix("edabo").
        map_err(|e| From::from(e)).
        and_then(|dirs| dirs.place_data_file("playlists").map_err(|e| From::from(e)))
}

fn get_playlist_filenames() -> Result<Vec<PathBuf>, EdaboError> {
    get_playlist_dir().
        and_then(|dir| dir.read_dir().map_err(|e| From::from(e))).
        and_then(|files|
                 files.map(|file|
                           file.map(|f|
                                    f.path()).
                           map_err(|e| From::from(e))
                 ).collect())
}

pub struct ListCommand;

impl Command for ListCommand {
    fn build_subcommand<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name("list").about("List all available playlists")
    }

    fn name(&self) -> &str{
        "list"
    }

    fn run(&self, _: ArgMatches) -> Result<(), EdaboError>{
        get_playlist_filenames().
            and_then(|filenames| {
                for filename in filenames {
                    println!("{}", filename.display())
                };
                Ok(())
            })
    }
}

pub struct PrintCommand;

impl Command for PrintCommand {
    fn build_subcommand<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name()).about("Print the current playlist as JSON")
    }

    fn name(&self) -> &str{
        "print"
    }

    fn run(&self, _: ArgMatches) -> Result<(), EdaboError> {
        empd::current_playlist().
            and_then(|playlist| serde_json::to_string_pretty(&playlist).map_err(|e| From::from(e))).
            and_then(|s| Ok(println!("{}", s))
            )
    }
}

    }
}
