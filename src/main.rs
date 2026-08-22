use anyhow::Result;
use clap::Parser;
use qctl::cli::{Cli, Command};
use qctl::ledger;
use qctl::mutate;
use std::io::{self, Write};
use std::process::ExitCode;

const INSTRUCTIONS: &str = include_str!("instructions.md");

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
        Command::Check(args) => qctl::check::run(&args)?,
        Command::Add(args) => mutate::add(&args)?,
        Command::Start(args) => mutate::start(&args)?,
        Command::Archive(args) => mutate::archive(&args)?,
        Command::Park(args) => mutate::park(&args)?,
        Command::Promote(args) => mutate::promote(&args)?,
        Command::Show(args) => ledger::print_show(&args)?,
        Command::Fmt(args) => qctl::format::run(&args)?,
        Command::Schema(args) => qctl::schema::write(&args)?,
        Command::Instructions => {
            io::stdout().write_all(INSTRUCTIONS.as_bytes())?;
        }
    }
    Ok(())
}
