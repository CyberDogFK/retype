use std::error::Error;
use std::path::Path;
use std::process::exit;
use clap::Parser;
use log::error;
use rstype::database::fetch_text_from_id;

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

    let file_text: FileText = if args.version {
        println!("Rstype version 0.1.0");
        exit(0)
    } else if args.history {
        todo!("History not implemented yet");
    } else if let Some(file_path) = args.file {
        load_text_from_file(file_path)
    } else if let Some(id) = args.id {
        load_from_database(id)
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

/// Load given text from database with given id.
/// # Arguments
/// * `text_id` - ID of text to load
/// $ Returns 
/// * `Result<FileText>` containing file contents or error message
fn load_from_database(text_id: u32) -> Result<FileText, String> {
    let row_count = 6000;
    if 1 <= text_id && text_id <= row_count {
        let text = fetch_text_from_id(text_id)
            .map_err(|e| format!("Error fetching text: {}", e))?;
        Ok((text, text_id.to_string()))
    } else { 
        Err(format!("ID out of range: {}", text_id))
    }
}

type FileText = (String, String);

/// Load file contents
/// # Arguments
/// * `file_path` - Path to file
/// # Returns
/// * `Result<FileText>` containing file contents or error message
fn load_text_from_file<P: AsRef<Path>>(file_path: P) -> Result<FileText, String> {
    if std::fs::exists(&file_path)
        .map_err(|e| format!("Error checking file: {}", e))? {
        let text = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Error reading file: {}", e))?;
        Ok((text, file_path.as_ref().display().to_string()))
    } else {
        Err(format!("File not found: {}", file_path.as_ref().display()))
    }
}
