use std::fs;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "migration",
    about = "Generate SQL migration files from SeaQuery definitions"
)]
struct Args {
    #[arg(long, help = "Output directory for generated .sql files")]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();
    let migrations = migration::migrations();

    for m in &migrations {
        let dir = args.output.join(m.name);
        fs::create_dir_all(&dir).expect("failed to create migration directory");

        let up_sql = (m.up)().join(";\n") + ";\n";
        let down_sql = (m.down)().join(";\n") + ";\n";

        fs::write(dir.join("up.sql"), &up_sql).expect("failed to write up.sql");
        fs::write(dir.join("down.sql"), &down_sql).expect("failed to write down.sql");

        println!("Generated: {}/up.sql", dir.display());
        println!("Generated: {}/down.sql", dir.display());
    }

    println!(
        "\n{} migration(s) generated to {}",
        migrations.len(),
        args.output.display()
    );
}
