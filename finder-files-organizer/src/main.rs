use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
enum SortBy {
    Name,
    Modified,
    Created,
    Size,
    Type,
    Tags,
}

#[derive(Debug, Clone)]
enum SortOrder {
    Asc,
    Desc,
}

struct FinderSorter {
    verbose: bool,
}

impl FinderSorter {
    fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    fn execute_applescript(&self, script: &str) -> Result<String, String> {
        if self.verbose {
            println!(" Executing AppleScript...");
            println!("{}", script);
        }

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| format!("Failed to execute AppleScript: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(format!("AppleScript error: {}", error))
        }
    }

    pub fn sort_finder_window(
        &self,
        path: &Path,
        sort_by: &SortBy,
        order: &SortOrder,
    ) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }

        if !path.is_dir() {
            return Err(format!("Path is not a directory: {}", path.display()));
        }

        let dir_str = path.to_string_lossy();

        println!("📂 Opening folder: {}", path.display());
        println!("📊 Sort by: {:?}", sort_by);
        println!("📈 Order: {:?}", order);

        // Step 1: Open folder
        let open_script = format!(
            r#"tell application "Finder"
                activate
                set theFolder to POSIX file "{}" as alias
                open theFolder
                delay 0.8
                tell front window
                    set current view to list view
                end tell
            end tell"#,
            dir_str
        );

        self.execute_applescript(&open_script)?;
        thread::sleep(Duration::from_millis(500));

        // Step 2: Use keyboard shortcut to sort
        // Ctrl+Option+Cmd+1 = Name, 2 = Type, 3 = Date Modified, etc.
        let key_code = match sort_by {
            SortBy::Name => "18",     // 1 key
            SortBy::Type => "19",     // 2 key
            SortBy::Modified => "20", // 3 key
            SortBy::Created => "21",  // 4 key
            SortBy::Size => "23",     // 5 key
            SortBy::Tags => "22",     // 6 key
        };

        let sort_script = format!(
            r#"tell application "System Events"
                tell process "Finder"
                    set frontmost to true
                    delay 0.2
                    key code {} using {{control down, option down, command down}}
                    delay 0.3
                end tell
            end tell"#,
            key_code
        );

        self.execute_applescript(&sort_script)?;
        thread::sleep(Duration::from_millis(500));

        // Step 3: Reversing sort order
        if matches!(order, SortOrder::Desc) {
            println!("🔄 Reversing sort order...");

            let reverse_script = r#"tell application "System Events"
                tell process "Finder"
                    tell front window
                        try
                            set allButtons to buttons of group 1 of splitter group 1 of splitter group 1
                            repeat with aButton in allButtons
                                try
                                    set sortDir to value of attribute "AXSortDirection" of aButton
                                    if sortDir is not missing value then
                                        click aButton
                                        exit repeat
                                    end if
                                end try
                            end repeat
                        end try
                    end tell
                end tell
            end tell"#;

            let _ = self.execute_applescript(reverse_script);
        }

        println!("✅ Finder window sorted successfully!");
        Ok(())
    }
}

fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(path.replacen("~", &home, 1));
        }
    }
    PathBuf::from(path)
}

fn print_usage() {
    println!(
        r#"
        Finder Sorter - Sort files in macOS Finder

        USAGE:
            finder-sorter <PATH> [OPTIONS]

        ARGUMENTS:
            <PATH>    Directory to open and sort

        OPTIONS:
            -s, --sort <SORT>     Sort by: name, modified, created, size, type, tags [default: name]
            -o, --order <ORDER>   Order: asc, desc [default: asc]
            -v, --verbose         Verbose output
            -h, --help           Show help

        KEYBOARD SHORTCUTS USED:
            ⌃⌥⌘1  Sort by Name
            ⌃⌥⌘2  Sort by Kind/Type
            ⌃⌥⌘3  Sort by Date Modified
            ⌃⌥⌘4  Sort by Date Created
            ⌃⌥⌘5  Sort by Size
            ⌃⌥⌘6  Sort by Tags

        EXAMPLES:
            finder-sorter ~/Downloads
            finder-sorter ~/Documents -s modified -o desc
            finder-sorter ~/Desktop -s size -o desc

        SETUP:
            Enable Accessibility in System Settings:
            Settings → Privacy → Accessibility
            Add Terminal to allowed apps
        "#
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_usage();
        if args.len() < 2 {
            std::process::exit(1);
        }
        return;
    }
    let path = expand_path(&args[1]);
    let mut sort_by = SortBy::Name;
    let mut order = SortOrder::Asc;
    let mut verbose = false;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--sort" => {
                if i + 1 < args.len() {
                    sort_by = match args[i + 1].as_str() {
                        "name" => SortBy::Name,
                        "modified" => SortBy::Modified,
                        "created" => SortBy::Created,
                        "size" => SortBy::Size,
                        "type" => SortBy::Type,
                        "tags" => SortBy::Tags,
                        _ => {
                            eprintln!("❌ Invalid: {}", args[i + 1]);
                            std::process::exit(1);
                        }
                    };
                    i += 2;
                } else {
                    eprintln!("❌ Missing value for --sort");
                    std::process::exit(1);
                }
            }
            "-o" | "--order" => {
                if i + 1 < args.len() {
                    order = match args[i + 1].as_str() {
                        "asc" => SortOrder::Asc,
                        "desc" => SortOrder::Desc,
                        _ => {
                            eprintln!("❌ Invalid: {}", args[i + 1]);
                            std::process::exit(1);
                        }
                    };
                    i += 2;
                } else {
                    eprintln!("❌ Missing value for --order");
                    std::process::exit(1);
                }
            }
            "-v" | "--verbose" => {
                verbose = true;
                i += 1;
            }
            _ => {
                eprintln!("❌ Unknown option: {}", args[i]);
                std::process::exit(1);
            }
        }
    }
    let sorter = FinderSorter::new(verbose);
    match sorter.sort_finder_window(&path, &sort_by, &order) {
        Ok(_) => println!("\n✨ Done!"),
        Err(e) => {
            eprintln!("\n❌ Error: {}", e);
            std::process::exit(1);
        }
    }
}
