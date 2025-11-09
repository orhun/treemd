use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

/// Detects the current shell from environment variables
pub fn detect_shell() -> Option<String> {
    env::var("SHELL").ok().and_then(|shell| {
        let shell_name = shell.split('/').next_back()?;
        Some(shell_name.to_string())
    })
}

/// Gets the appropriate shell config file path
pub fn get_shell_config_path(shell: &str) -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;
    let config_file = match shell {
        "bash" => ".bashrc",
        "zsh" => ".zshrc",
        "fish" => ".config/fish/config.fish",
        _ => return None,
    };
    Some(PathBuf::from(home).join(config_file))
}

/// Generates the completion setup line for a given shell
pub fn get_completion_setup_line(shell: &str, bin_name: &str) -> Option<String> {
    match shell {
        "bash" => Some(format!("source <(COMPLETE=bash {})", bin_name)),
        "zsh" => Some(format!("source <(COMPLETE=zsh {})", bin_name)),
        "fish" => Some(format!("COMPLETE=fish {} | source", bin_name)),
        _ => None,
    }
}

/// Checks if completion is already set up in the config file
pub fn is_completion_setup(config_path: &PathBuf, bin_name: &str) -> io::Result<bool> {
    let content = fs::read_to_string(config_path)?;
    Ok(content.contains("COMPLETE=") && content.contains(bin_name))
}

/// Adds completion setup to the shell config file
pub fn add_completion_to_config(config_path: &PathBuf, setup_line: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(config_path)?;

    writeln!(file, "\n# treemd shell completion")?;
    writeln!(file, "{}", setup_line)?;
    Ok(())
}

/// Interactive setup for shell completions
pub fn setup_completions_interactive(bin_name: &str) -> io::Result<()> {
    let shell = detect_shell()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not detect shell"))?;

    println!("Detected shell: {}", shell);

    let config_path = get_shell_config_path(&shell).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Unsupported shell: {}", shell),
        )
    })?;

    if config_path.exists() && is_completion_setup(&config_path, bin_name)? {
        println!(
            "✓ Shell completions are already set up in {:?}",
            config_path
        );
        return Ok(());
    }

    let setup_line = get_completion_setup_line(&shell, bin_name).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("No completion setup available for {}", shell),
        )
    })?;

    println!(
        "\nTo enable shell completions, add this line to {:?}:",
        config_path
    );
    println!("  {}", setup_line);
    println!("\nWould you like to add it automatically? (y/N): ");

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if response.trim().eq_ignore_ascii_case("y") {
        add_completion_to_config(&config_path, &setup_line)?;
        println!("✓ Added completions to {:?}", config_path);
        println!("\nRestart your shell or run:");
        println!("  source {:?}", config_path);
    } else {
        println!("\nManually add the line above to enable completions.");
    }

    Ok(())
}

/// Print completion setup instructions for all supported shells
pub fn print_completion_instructions(bin_name: &str) {
    println!("Shell Completion Setup Instructions\n");
    println!("Add the appropriate line to your shell config:\n");

    println!("Bash (~/.bashrc):");
    println!("  source <(COMPLETE=bash {})\n", bin_name);

    println!("Zsh (~/.zshrc):");
    println!("  source <(COMPLETE=zsh {})\n", bin_name);

    println!("Fish (~/.config/fish/config.fish):");
    println!("  COMPLETE=fish {} | source\n", bin_name);

    println!("After adding the line, restart your shell or source the config file.");
}
