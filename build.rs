use clap_complete::{generate_to, shells};
use clap::CommandFactory;
use std::env;
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir_string = std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?;
    // generate cli
    let mut cmd = Cli::command();

    // generate completions
    generate_to(
        shells::Bash,
        &mut cmd, // We need to specify what generator to use
        env!("CARGO_PKG_NAME"),  // We need to specify the bin name manually
        &outdir_string,   // We need to specify where to write to
    )?;
    generate_to(
        shells::Zsh,
        &mut cmd, // We need to specify what generator to use
        env!("CARGO_PKG_NAME"),  // We need to specify the bin name manually
        &outdir_string,   // We need to specify where to write to
    )?;
    generate_to(
        shells::Fish,
        &mut cmd, // We need to specify what generator to use
        env!("CARGO_PKG_NAME"),  // We need to specify the bin name manually
        &outdir_string,   // We need to specify where to write to
    )?;

    // generate man
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;
    let outdir_path = std::path::PathBuf::from(outdir_string);
    std::fs::write(outdir_path.join("mybin.1"), buffer)?;

    Ok(())
}
