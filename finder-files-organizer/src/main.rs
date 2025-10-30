use clap::{Parser, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, ValueEnum)]
#[value(rename_all = "lowercase")]
enum SortBy {
    Name,
    Modified,
    Created,
    Size,
    Type,
    Tags,
}

impl SortBy {
    fn key_code(&self) -> &'static str {
        match self {
            SortBy::Name => "18",     // 1 key
            SortBy::Type => "19",     // 2 key
            SortBy::Modified => "20", // 3 key
            SortBy::Created => "21",  // 4 key
            SortBy::Size => "23",     // 5 key
            SortBy::Tags => "22",     // 6 key
        }
    }

    fn sort_column(&self) -> &'static str {
        match self {
            SortBy::Name => "name column",
            SortBy::Type => "kind column",
            SortBy::Modified => "modification date column",
            SortBy::Created => "creation date column",
            SortBy::Size => "size column",
            SortBy::Tags => "label column",
        }
    }
}

impl From<&SortBy> for &'static str {
    fn from(sort_by: &SortBy) -> Self {
        sort_by.key_code()
    }
}

#[derive(Debug, Clone, ValueEnum)]
#[value(rename_all = "lowercase")]
enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    fn direction(&self) -> &'static str {
        match self {
            SortOrder::Asc => "normal",
            SortOrder::Desc => "reversed",
        }
    }
}

impl From<&SortOrder> for &'static str {
    fn from(order: &SortOrder) -> Self {
        order.direction()
    }
}

/// Sort files in macOS Finder
#[derive(Parser, Debug)]
#[command(name = "finder-sorter")]
#[command(about = "Finder Sorter - Sort files in macOS Finder", long_about = None)]
struct Args {
    /// Directory to open and sort
    #[arg(value_parser = parse_path)]
    path: PathBuf,

    /// Sort by: name, modified, created, size, type, tags
    #[arg(short, long, value_enum, default_value_t = SortBy::Type)]
    sort: SortBy,

    /// Order: asc, desc
    #[arg(short, long, value_enum, default_value_t = SortOrder::Asc)]
    order: SortOrder,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Recursively sort all nested folders
    #[arg(short, long)]
    recursive: bool,
}

fn parse_path(s: &str) -> Result<PathBuf, String> {
    let path = if s.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(s.replacen("~", &home, 1))
        } else {
            PathBuf::from(s)
        }
    } else {
        PathBuf::from(s)
    };
    Ok(path)
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
            println!(" Executing AppleScript...");
            println!("{script}");
        }

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| format!("Failed to execute AppleScript: {e}"))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(format!("AppleScript error: {error}"))
        }
    }

    fn get_all_subdirectories(&self, root: &Path) -> Result<Vec<PathBuf>, String> {
        let mut directories = vec![root.to_path_buf()];

        fn visit_dirs(dir: &Path, dirs: &mut Vec<PathBuf>) -> std::io::Result<()> {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        dirs.push(path.clone());
                        visit_dirs(&path, dirs)?;
                    }
                }
            }
            Ok(())
        }

        visit_dirs(root, &mut directories)
            .map_err(|e| format!("Failed to traverse directories: {e}"))?;

        Ok(directories)
    }

    fn set_folder_sort_preferences_background(
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
        let sort_column = sort_by.sort_column();
        let sort_direction = order.direction();

        let script = include_str!("../scripts/background_sort.applescript")
            .replace("{FOLDER_PATH}", &dir_str)
            .replace("{SORT_COLUMN}", sort_column)
            .replace("{SORT_DIRECTION}", sort_direction);

        self.execute_applescript(&script)?;
        Ok(())
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

        println!("Opening folder: {}", path.display());
        println!("Sort by: {sort_by:?}");
        println!("Order: {order:?}");

        // Step 1: Open folder
        let open_script =
            include_str!("../scripts/open_folder.applescript").replace("{FOLDER_PATH}", &dir_str);

        self.execute_applescript(&open_script)?;
        thread::sleep(Duration::from_millis(300));

        // Step 2: Use keyboard shortcut to sort
        let key_code: &str = sort_by.into();

        let sort_script =
            include_str!("../scripts/sort_keyboard.applescript").replace("{KEY_CODE}", key_code);

        self.execute_applescript(&sort_script)?;
        thread::sleep(Duration::from_millis(500));

        // Step 3: Reversing sort order
        if matches!(order, SortOrder::Desc) {
            println!("Reversing sort order...");

            let reverse_script = include_str!("../scripts/reverse_sort.applescript");

            let _ = self.execute_applescript(reverse_script);
        }

        println!("Finder window sorted successfully!");
        Ok(())
    }

    pub fn sort_recursively(
        &self,
        path: &Path,
        sort_by: &SortBy,
        order: &SortOrder,
    ) -> Result<(), String> {
        println!("Finding all subdirectories...");
        let directories = self.get_all_subdirectories(path)?;

        println!(
            "Found {} director{} to sort",
            directories.len(),
            if directories.len() == 1 { "y" } else { "ies" }
        );

        for (index, dir) in directories.iter().enumerate() {
            println!(
                "[{}/{}] Processing: {}",
                index + 1,
                directories.len(),
                dir.display()
            );

            self.set_folder_sort_preferences_background(dir, sort_by, order)?;
        }

        println!("All folders sorted!");
        Ok(())
    }
}

fn main() {
    let args = Args::parse();

    let sorter = FinderSorter::new(args.verbose);

    let result = if args.recursive {
        println!("Recursive mode enabled - sorting all nested folders\n");
        sorter.sort_recursively(&args.path, &args.sort, &args.order)
    } else {
        sorter.sort_finder_window(&args.path, &args.sort, &args.order)
    };

    match result {
        Ok(_) => println!("\nDone!!!"),
        Err(e) => {
            eprintln!("\nError: {e}");
            std::process::exit(1);
        }
    }
}
