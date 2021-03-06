extern crate clap;
extern crate edabo;
extern crate mpd;
extern crate xdg;

use clap::App;
use edabo::types::*;
use edabo::commands::*;

fn make_clap_parser<'a, 'b>() -> App<'a, 'b> {
    // TODO move this into something global so we don't construct it here and
    // when evaluating the arguments
    let allcommands: Vec<Box<Command>> = vec![
        Box::new(ListCommand {}),
        Box::new(PrintCommand {}),
        Box::new(AddCommand {}),
        Box::new(LoadCommand {}),
    ];
    let mut app = App::new("Edabo").version("1.0").author("Wieland Hoffmann");
    for command in allcommands {
        let commandapp = command.build_subcommand();
        app = app.subcommand(commandapp);
    }
    app
}

fn main() {
    let matches = make_clap_parser().get_matches();
    let allcommands: Vec<Box<Command>> = vec![
        Box::new(ListCommand {}),
        Box::new(PrintCommand {}),
        Box::new(AddCommand {}),
        Box::new(LoadCommand {}),
    ];
    for command in allcommands {
        if let Some(args) = matches.subcommand_matches(command.name()) {
            match command.run(args) {
                Ok(_) => (),
                Err(e) => println!("{:?}", e),
            }
            break;
        }
    }
}
