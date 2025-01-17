use std::process::Command;
use std::env;
use clap::{App, Arg};

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate ureq;

mod commands {
  pub(crate) mod config;
  pub(crate) mod init;
  pub(crate) mod server;
  pub(crate) mod apply;
  pub(crate) mod deployment;
  pub(crate) mod login;
  pub(crate) mod user;
}

mod scheduler {
  pub(crate) mod scheduler;
}

mod runtime {
  pub(crate) mod docker;
}

mod models {
  pub(crate) mod deployments;
  pub(crate) mod users;
}

mod api;

mod config {
    pub(crate) mod api;
    pub(crate) mod config;
    pub(crate) mod user;
}

mod database;

use crate::database::get_database_connection;

#[tokio::main]
async fn main() {
    env_logger::init();

    let commands = vec![
        crate::commands::config::command_config(),
        crate::commands::init::command_config(),
        crate::commands::server::command_config(),
        crate::commands::apply::command_config(),
        crate::commands::login::command_config(),
        crate::commands::deployment::list::command_config(),
        crate::commands::deployment::inspect::command_config(),
        crate::commands::deployment::delete::command_config(),
        crate::commands::user::list::command_config(),
        crate::commands::user::create::command_config(),
        crate::commands::user::update::command_config(),
        crate::commands::user::delete::command_config(),
    ];

    let app = App::new("ring")
      .version("0.1.0")
      .author("Mlanawo Mbechezi <mlanawo.mbechezi@kemeter.io>")
      .about("The ring to rule them all")
      .arg(
          Arg::with_name("context")
              .required(false)
              .help("Sets the context to use")
              .long("context")
              .short("c")
      )
      .subcommands(commands);

    let matches = app.get_matches();
    let subcommand_name = matches.subcommand_name();
    let storage = get_database_connection();
    let config = config::config::load_config();

    match subcommand_name {
        Some("config") => {
            crate::commands::config::execute(
                matches.subcommand_matches("config").unwrap(),
                config,
            );
        }
        Some("init") => {
            crate::commands::init::init(
                matches.subcommand_matches("init").unwrap(),
                storage
            );
        }
        Some("server:start") => {
            crate::commands::server::execute(
                matches.subcommand_matches("server:start").unwrap(),
                config,
                storage
            ).await
        }
        Some("apply") => {
          crate::commands::apply::apply(
              matches.subcommand_matches("apply").unwrap(),
              config,
          );
        }
        Some("deployment:list") => {
            crate::commands::deployment::list::execute(
                matches.subcommand_matches("deployment:list").unwrap(),
                config,
            );
        }
        Some("deployment:inspect") => {
            crate::commands::deployment::inspect::execute(
                matches.subcommand_matches("deployment:inspect").unwrap(),
                config
            ).await
        }
        Some("deployment:delete") => {
            crate::commands::deployment::delete::execute(
                matches.subcommand_matches("deployment:delete").unwrap(),
                config
            ).await
        }
        Some("login") => {
            crate::commands::login::execute(
                matches.subcommand_matches("login").unwrap(),
                config,
            );
        }
        Some("user:list") => {
            crate::commands::user::list::execute(
                matches.subcommand_matches("user:list").unwrap(),
                config
            );
        }
        Some("user:create") => {
            crate::commands::user::create::execute(
                matches.subcommand_matches("user:create").unwrap(),
                config
            );
        }
        Some("user:update") => {
            crate::commands::user::update::execute(
                matches.subcommand_matches("user:update").unwrap(),
                config
            );
        }
        Some("user:delete") => {
            crate::commands::user::delete::execute(
                matches.subcommand_matches("user:delete").unwrap(),
                config
            );
        }
        _ => {
            let process_args: Vec<String> = env::args().collect();
            let process_name = process_args[0].as_str().to_owned();

            let mut subprocess = Command::new(process_name.as_str())
                .arg("--help")
                .spawn()
                .expect("failed to execute process");

            subprocess
                .wait()
                .expect("failed to wait for process");
        }
    }
}

