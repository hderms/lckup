use clap::Parser;
use clap::Subcommand;
use colored::Colorize;
use file_mode::{Mode, ModePath, User};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::fs::set_permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "lckup")]
#[command(about = "Simpler file permission management", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Debug, Subcommand)]
enum Commands {
    /// Only owner can read/write/execute
    #[command(arg_required_else_help = true)]
    Safe {
        /// The file to modify
        file: PathBuf,
    },
    /// Everyone can read, only owner can read/write/execute
    #[command(arg_required_else_help = true)]
    Show {
        /// The file to modify
        file: PathBuf,
    },
    /// Completely wide open
    #[command(arg_required_else_help = true)]
    All {
        /// The file to modify
        file: PathBuf,
    },
    /// Shows info about permissions
    #[command(arg_required_else_help = true)]
    Info {
        /// The file to modify
        file: PathBuf,
    },
}
impl Commands {
    fn file(&self) -> &PathBuf {
        match self {
            Commands::Safe { file } => file,
            Commands::Show { file } => file,
            Commands::All { file } => file,
            Commands::Info { file } => file,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let file = args.command.file();
    let mut perms = fs::metadata(file.clone())?.permissions();
    match args.command {
        Commands::Safe { file } => {
            perms.set_mode(0o700);
            set_permissions(file, perms)?;
        }
        Commands::Show { file } => {
            perms.set_mode(0o744);
            set_permissions(file, perms)?;
        }
        Commands::All { file } => {
            perms.set_mode(0o777);
            set_permissions(file, perms)?;
        }
        Commands::Info { file } => {
            let perms = WrappedPermissions::new(file.mode()?);
            println!("{}", perms);
        }
    }
    Ok(())
}
struct WrappedPermissions {
    mode: Mode,
}

impl WrappedPermissions {
    pub fn new(mode: Mode) -> Self {
        Self { mode }
    }
}

impl Display for WrappedPermissions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for g in [User::Owner, User::Group, User::Other] {
            //heading
            let user_string = format!("{:?}:", g);
            writeln!(f, "{}", user_string.bold())?;

            let r = self.mode.user_protection(g).is_read_set();
            let w = self.mode.user_protection(g).is_write_set();
            let e = self.mode.user_protection(g).is_execute_set();
            //rows
            writeln!(
                f,
                "\t{: <5}: {: >6}",
                permission_colorize("read", r),
                to_check(r)
            )?;
            writeln!(
                f,
                "\t{: <5}: {: >6}",
                permission_colorize("write", w),
                to_check(w)
            )?;
            writeln!(
                f,
                "\t{: <5}: {: >6}",
                permission_colorize("exec", e),
                to_check(e)
            )?;
        }
        Ok(())
    }
}

fn permission_colorize(s: &str, boolean: bool) -> String {
    if boolean {
        s.green().to_string()
    } else {
        s.red().to_string()
    }
}

fn to_check(b: bool) -> char {
    if b {
        '✓'
    } else {
        '✕'
    }
}
