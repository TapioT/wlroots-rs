extern crate clap;
extern crate ini;
#[macro_use]
extern crate wlroots;

mod config;

const ROOSTON_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const ROOSTON_AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const ROOSTON_DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");

use clap::{App, Arg};

fn main() {
    let app = App::new("rooston").version(ROOSTON_VERSION)
                                 .author(ROOSTON_AUTHORS)
                                 .about(ROOSTON_DESCRIPTION)
                                 .arg(Arg::with_name("config").short("C")
                                                              .value_name("FILE")
                                                              .help("Path to the configuration \
                                                                     file (default: rooston.ini). \
                                                                     See `rooston.ini.example` \
                                                                     for config file \
                                                                     documentation.")
                                                              .takes_value(true))
                                 .arg(Arg::with_name("command").short("E")
                                                               .value_name("COMMAND")
                                                               .help("Command that will be ran \
                                                                      at startup."));
    let config = config::roots_config_from_args(app);
    wlr_log!(L_DEBUG, "Config: {:#?}", config);
}