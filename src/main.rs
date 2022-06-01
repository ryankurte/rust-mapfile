use bloatr::MapFile;
use clap::Parser;

use log::{debug, info, warn, LevelFilter};
use simplelog::SimpleLogger;

#[derive(PartialEq, Debug, Parser)]
#[clap(author, version, about)]
pub struct Args {

    /// Map file to parse
    pub file: String,

    #[clap(long, default_value="debug")]
	/// Application log level
	pub log_level: LevelFilter,
}

fn main() -> anyhow::Result<()> {
    // Parse command line arguments
	let args = Args::parse();
	println!("Args: {:?}", args);

	// Setup logging
	let _ = SimpleLogger::init(args.log_level, Default::default());

    debug!("Loading map file: '{}'", args.file);

    // Load map file
    let raw = std::fs::read_to_string(&args.file)?;

    // Parse map
    let m = match MapFile::parse(&raw) {
        Ok(v) => v,
        // TODO: reshape errors to give -useful- context
        Err(_e) => return Err(anyhow::anyhow!("Failed to parse .map")),
    };

    //debug!("Map object: {:#?}", m);

    Ok(())
}
