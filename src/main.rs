use std::error::Error;
use std::path::Path;
use std::process::exit;
use clap::Parser;
use log::error;
use rstype::database::fetch_text_with_id;
use rstype::{load_text_from_database, load_text_from_file, PreparedText};

#[derive(Parser, Debug)]
struct Arguments {
    #[clap(short, long, action)]
    /// Show rstype version
    version: bool,
    #[clap(short, long, value_name="FILENAME")]
    /// File to use text from as sample text
    file: Option<String>,
    #[clap(short, long, value_name="id")]
    /// ID to retrieve text from database
    id: Option<u32>,
    #[clap(short, long, value_name="N", default_value="2")]
    /// Choose difficulty withing range 1-5
    difficulty: Option<u8>,
    #[clap(short='H', long, action)]
    /// Show rstype score history
    history: bool
}

fn main() {
    let args = Arguments::parse();
    println!("Hello, world!");

    let file_text: PreparedText = if args.version {
        println!("Rstype version 0.1.0");
        exit(0)
    } else if args.history {
        todo!("History not implemented yet");
    } else if let Some(file_path) = args.file {
        load_text_from_file(file_path)
    } else if let Some(id) = args.id {
        load_text_from_database(id)
    } else if let Some(difficulty) = args.difficulty {
        todo!("Load from database based on difficulty not implemented yet");
    } else {
        todo!("Load bases on random dificulty not implemented yet")
    }.unwrap_or_else(|e| {
        error!("{}", e);
        exit(1)
    });
    todo!();
}


