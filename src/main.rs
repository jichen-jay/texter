use rand::distributions::{Alphanumeric, DistString};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

fn generate_random_string(length: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), length)[0..length].to_string()
}

fn walk_directory(directory: &Path, recursive: bool) -> io::Result<Vec<String>> {
    let canonical_path = directory.canonicalize().map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("Failed to resolve path '{}': {}", directory.display(), e),
        )
    })?;

    if !canonical_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Directory '{}' does not exist", canonical_path.display()),
        ));
    }

    let dir = fs::read_dir(&canonical_path)?;
    let mut all_files = Vec::new();

    for entry in dir {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && recursive {
            let sub_files = walk_directory(&path, recursive)?;
            all_files.extend(sub_files);
        } else if path.is_file() {
            all_files.push(path.to_string_lossy().into_owned());
        }
    }

    Ok(all_files)
}

fn main() {
    std::env::set_current_dir(std::path::Path::new(".")).expect("current dir failed");

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut recursive = false;
    let mut processed_args = Vec::new();

    for arg in &args {
        if arg == "-r" {
            recursive = true;
        } else {
            processed_args.push(arg.clone());
        }
    }

    if processed_args.len() != 2 {
        eprintln!("Usage: texter [-r] <directory> <extension>");
        eprintln!("Example: texter -r ./src .rs");
        std::process::exit(1);
    }

    let directory = Path::new(&processed_args[0]);
    let extension = &processed_args[1];

    match combine_text_files(&directory.to_string_lossy(), extension, recursive) {
        Ok(()) => println!("Files combined successfully"),
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    }
}

fn copy_to_clipboard(text: &str) -> io::Result<()> {
    // Try common Ubuntu clipboard utilities in order of preference
    let clipboard_commands = [
        ("xclip", vec!["-selection", "clipboard"]),
        ("xsel", vec!["--clipboard", "--input"]),
        ("wl-copy", vec![]), // For Wayland systems
    ];

    for (cmd, args) in &clipboard_commands {
        match Command::new(cmd)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(text.as_bytes())?;
                    drop(stdin); // Close stdin to signal end of input
                }

                match child.wait() {
                    Ok(status) if status.success() => {
                        println!("Content copied to clipboard using {}", cmd);
                        return Ok(());
                    }
                    Ok(_) => {
                        eprintln!("Warning: {} exited with non-zero status", cmd);
                        continue; // Try next command
                    }
                    Err(e) => {
                        eprintln!("Warning: Error waiting for {}: {}", cmd, e);
                        continue; // Try next command
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                continue; // Try next command
            }
            Err(e) => {
                eprintln!("Warning: Error spawning {}: {}", cmd, e);
                continue; // Try next command
            }
        }
    }

    println!("Warning: No clipboard utility found. Install xclip with: sudo apt install xclip");
    Ok(())
}

fn combine_text_files(directory: &str, extension: &str, recursive: bool) -> io::Result<()> {
    let files = walk_directory(Path::new(directory), recursive)?;
    let mut combined_content = String::new();

    for file_path in files {
        if file_path.ends_with(extension) {
            let content = fs::read_to_string(&file_path)?;
            let file_name = Path::new(&file_path).file_name().unwrap().to_string_lossy();

            combined_content.push_str(&format!("\n{}\n", file_name));
            combined_content.push_str(&"=".repeat(file_name.len()));
            combined_content.push_str(&format!("\n\n{}\n", content));
        }
    }

    // Copy to clipboard
    copy_to_clipboard(&combined_content)?;

    let random_string = generate_random_string(8);
    let output_filename = format!("__{}.txt", random_string);
    let mut file = File::create(output_filename)?;
    file.write_all(combined_content.as_bytes())?;

    Ok(())
}
