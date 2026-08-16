mod check;
mod cli;
mod ledger;
mod mutate;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("qctl: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init(args) => mutate::init(&args)?,
        Command::Status(args) => ledger::print_status(&args)?,
        Command::Check(args) => check::run(&args)?,
        Command::Add(args) => mutate::add(&args)?,
        Command::Start(args) => mutate::start(&args)?,
        Command::Archive(args) => mutate::archive(&args)?,
        Command::Show(args) => ledger::print_show(&args)?,
    }
    Ok(())
}
