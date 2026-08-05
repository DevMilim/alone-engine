use std::{
    fs,
    process::{self, Command},
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "milim")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New { name: String },
    Build,
    Package,
    Update,
}

fn main() -> std::io::Result<()> {
    let args: Args = Args::parse();

    match args.command {
        Commands::New { name } => create_project(&name)?,

        Commands::Build => {
            println!("Compilando...");
        }

        Commands::Package => {
            println!("Empacotando...");
        }
        Commands::Update => update()?,
    }
    Ok(())
}
pub fn update() -> std::io::Result<()> {
    let status = Command::new("cargo")
        .args([
            "install",
            "--git",
            "https://github.com/DevMilim/alone-engine",
            "milim",
            "--force",
        ])
        .status()?;

    if !status.success() {
        eprintln!("Falha ao atualizar");
    }

    Ok(())
}
pub fn create_project(name: &str) -> std::io::Result<()> {
    let root = std::env::current_dir()?.join(name);

    if root.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("O projeto '{}' já existe", name),
        ));
    }

    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("assets"))?;

    let files = [
        ("Cargo.toml", cargo_toml(name)),
        ("src/main.rs", include_str!("template/main.rs").to_string()),
        (
            ".gitignore",
            include_str!("template/.gitignore").to_string(),
        ),
    ];

    for (path, content) in files {
        fs::write(root.join(path), content)?;
    }

    Ok(())
}

fn cargo_toml(name: &str) -> String {
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        eprintln!("Nome de projeto inválido");
        process::exit(1);
    }
    let cargo_toml_template = include_str!("template/Cargo.toml");
    cargo_toml_template.replace("{{name}}", name)
}
