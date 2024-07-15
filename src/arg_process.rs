use clap::{Arg, Command, ArgMatches,ArgAction};
pub fn read_command() -> ArgMatches {
    // deal with command input
    Command::new("oj")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .action(ArgAction::Set)
                .default_value("config.json")
                
        )
        .arg(
            Arg::new("flush-data")
                .short('f')
                .long("flush-data")
                .action(ArgAction::SetTrue)
        )

        .get_matches()
}