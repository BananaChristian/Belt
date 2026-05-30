mod build;
mod config;
mod graph;
mod leather;
mod project;
mod scaffold;

use std::env;
use std::process::Command;

use crate::scaffold::scaffold;
use leather::analyzer::Semantics;
use leather::lexer::Lexer;
use leather::parser::Parser;

use crate::build::Builder;
use crate::config::Config;
use crate::project::Workspace;

fn load_config() -> Result<Config, String> {
    let mut lexer = Lexer::new();
    lexer.load(&String::from("belt.lethr"));
    lexer.generate();

    let mut parser = Parser::new();
    parser.load(lexer.get_tokens());
    let ast = parser.parse()?;

    let analyzer = Semantics::new(ast);
    analyzer.analyze()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("new") => match args.get(2) {
            Some(name) => match scaffold(name) {
                Ok(_) => println!("Created project {}", name),
                Err(_) => eprintln!("Failed to create project {}", name),
            },
            None => eprintln!("error: belt new requires a project name"),
        },
        Some("build") => match load_config() {
            Ok(config) => match Workspace::build(&config) {
                Ok(workspace) => match Builder::new(workspace) {
                    Ok(builder) => match builder.build() {
                        Ok(_) => println!("build complete"),
                        Err(e) => eprintln!("{}", e),
                    },
                    Err(e) => eprintln!("{}", e),
                },
                Err(e) => eprintln!("{}", e),
            },
            Err(e) => eprintln!("{}", e),
        },
        Some("run") => match load_config() {
            Ok(config) => match Workspace::build(&config) {
                Ok(workspace) => {
                    let build_dir = &config.layout.build;
                    let name = &config.project.name;
                    let output = format!("{}/{}", build_dir, name);
                    match Builder::new(workspace) {
                        Ok(builder) => match builder.build() {
                            Ok(_) => {
                                let status = Command::new(&output)
                                    .status()
                                    .map_err(|e| eprintln!("error: failed to run: {}", e));
                            }
                            Err(e) => eprintln!("{}", e),
                        },
                        Err(e) => eprintln!("{}", e),
                    }
                }
                Err(e) => eprintln!("{}", e),
            },
            Err(e) => eprintln!("{}", e),
        },
        Some("clean") => match load_config() {
            Ok(config) => {
                let layout = &config.layout;
                let dirs = [&layout.stubs, &layout.objs, &layout.build];
                let mut failed = false;
                for dir in &dirs {
                    match std::fs::read_dir(dir) {
                        Ok(entries) => {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.is_file() {
                                    if let Err(e) = std::fs::remove_file(&path) {
                                        eprintln!(
                                            "error: failed to remove {}:{}",
                                            path.display(),
                                            e
                                        );
                                        failed = true;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("error: could not read {}: {}", dir, e);
                            failed = true;
                        }
                    }
                }
                if !failed {
                    println!("clean complete");
                }
            }
            Err(e) => eprintln!("{}", e),
        },
        Some("check") => match load_config() {
            Ok(config) => {
                println!("{:#?}", config);
            }
            Err(e) => eprintln!("{}", e),
        },
        Some("version") => {
            println!("belt v0.1.0");
        }
        Some("help") => {
            println!("Usage: belt <command> [options]");
            println!();
            println!("Commands:");
            println!("  new <name>      Create a new project");
            println!("  build           Build the project");
            println!("  run             Run the generated executable");
            println!("  clean           Delete the generated objects,executable and stubs");
            println!("  check           Run frontend checks only");
            println!("  help            Show this message");
            println!("  version         Show belt's version");
            println!();
            println!("belt.lethr options:");
            println!();
            println!("  [project]");
            println!("    name                  Project name");
            println!("    version               Project version");
            println!("    entry                 Entry point file");
            println!("    mode                  executable | static | obj");
            println!("    target                Target triple (e.g. x86_64-unknown-linux-gnu)");
            println!("    freestanding          true | false, do not link standard library");
            println!("    script                Path to custom linker script");
            println!();
            println!("  [layout]                Optional, overrides defaults");
            println!("    src                   Source directory (default: src)");
            println!("    stubs                 Stubs directory (default: stubs)");
            println!("    objs                  Object directory (default: objs)");
            println!("    build                 Build directory (default: build)");
            println!();
            println!("  [dependencies]");
            println!("    <name> = \"<path>\"      Local Unnameable project");
            println!();
            println!("  [link]");
            println!("    <name> = [\"lib1\"]      C libraries to link");
            println!();
            println!("Examples:");
            println!("  belt new myproject");
            println!("  belt build");
            println!(
                "  belt build  # with target in belt.lethr: target = x86_64-unknown-linux-gnu"
            );
            println!("  belt build  # kernel: freestanding = true, script = linker.ld");
        }
        None => eprintln!("error: no command provided"),
        _ => eprintln!("error: unknown command"),
    }
}
