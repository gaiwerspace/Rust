use clap::{Parser, ValueEnum};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

// ============================================================================
// Enums and Type Definitions
// ============================================================================

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
    const fn key_code(&self) -> &'static str {
        match self {
            Self::Name => "18",
            Self::Type => "19",
            Self::Modified => "20",
            Self::Created => "21",
            Self::Size => "23",
            Self::Tags => "22",
        }
    }

    const fn sort_column(&self) -> &'static str {
        match self {
            Self::Name => "name column",
            Self::Type => "kind column",
            Self::Modified => "modification date column",
            Self::Created => "creation date column",
            Self::Size => "size column",
            Self::Tags => "label column",
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
#[value(rename_all = "lowercase")]
enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    const fn direction(&self) -> &'static str {
        match self {
            Self::Asc => "normal",
            Self::Desc => "reversed",
        }
    }
}

// ============================================================================
// CLI Arguments
// ============================================================================

#[derive(Parser, Debug)]
#[command(name = "finder-sorter")]
#[command(about = "Finder Sorter - Sort and organize files in macOS Finder")]
#[command(version)]
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

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Recursively sort all nested folders
    #[arg(short, long)]
    recursive: bool,

    /// WARNING: This changes the folder structure! Organize files into folders by extension
    #[arg(long)]
    pack_to_folders: bool,
}

fn parse_path(s: &str) -> Result<PathBuf, String> {
    let path = if let Some(rest) = s.strip_prefix("~/") {
        std::env::var("HOME")
            .map(|home| PathBuf::from(home).join(rest))
            .unwrap_or_else(|_| PathBuf::from(s))
    } else {
        PathBuf::from(s)
    };
    Ok(path)
}

// ============================================================================
// Finder Sorter
// ============================================================================

struct FinderSorter {
    verbose: bool,
}

impl FinderSorter {
    const fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    fn log(&self, message: impl AsRef<str>) {
        if self.verbose {
            eprintln!("{}", message.as_ref());
        }
    }

    fn validate_directory(&self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }
        if !path.is_dir() {
            return Err(format!("Path is not a directory: {}", path.display()));
        }
        Ok(())
    }

    fn execute_applescript(&self, script: &str) -> Result<String, String> {
        self.log(" Executing AppleScript...");
        self.log(script);

        let output = Command::new("osascript")
            .args(["-e", script])
            .output()
            .map_err(|e| format!("Failed to execute AppleScript: {e}"))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("AppleScript error: {}", error.trim()))
        }
    }

    fn get_all_subdirectories(&self, root: &Path) -> Result<Vec<PathBuf>, String> {
        let mut directories = Vec::with_capacity(16);
        directories.push(root.to_path_buf());

        fn visit_dirs(dir: &Path, dirs: &mut Vec<PathBuf>) -> io::Result<()> {
            if !dir.is_dir() {
                return Ok(());
            }

            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path.clone());
                    visit_dirs(&path, dirs)?;
                }
            }
            Ok(())
        }

        visit_dirs(root, &mut directories)
            .map_err(|e| format!("Failed to traverse directories: {e}"))?;

        Ok(directories)
    }

    fn build_applescript(&self, template: &str, replacements: &[(&str, &str)]) -> String {
        let mut script = template.to_string();
        for (placeholder, value) in replacements {
            script = script.replace(placeholder, value);
        }
        script
    }

    fn set_folder_sort_preferences_background(
        &self,
        path: &Path,
        sort_by: &SortBy,
        order: &SortOrder,
    ) -> Result<(), String> {
        self.validate_directory(path)?;

        let script = self.build_applescript(
            include_str!("../scripts/background_sort.applescript"),
            &[
                ("{FOLDER_PATH}", &path.to_string_lossy()),
                ("{SORT_COLUMN}", sort_by.sort_column()),
                ("{SORT_DIRECTION}", order.direction()),
            ],
        );

        self.execute_applescript(&script)?;
        Ok(())
    }

    fn sort_finder_window(
        &self,
        path: &Path,
        sort_by: &SortBy,
        order: &SortOrder,
    ) -> Result<(), String> {
        self.validate_directory(path)?;

        eprintln!("Opening folder: {}", path.display());
        eprintln!("Sort by: {sort_by:?}");
        eprintln!("Order: {order:?}");

        let dir_str = path.to_string_lossy();

        // Step 1: Open folder
        let open_script = self.build_applescript(
            include_str!("../scripts/open_folder.applescript"),
            &[("{FOLDER_PATH}", &dir_str)],
        );
        self.execute_applescript(&open_script)?;

        // Step 2: Sort by key code
        let sort_script = self.build_applescript(
            include_str!("../scripts/sort_keyboard.applescript"),
            &[("{KEY_CODE}", sort_by.key_code())],
        );
        self.execute_applescript(&sort_script)?;

        // Step 3: Reverse sort order if needed
        if matches!(order, SortOrder::Desc) {
            eprintln!("Reversing sort order...");
            let reverse_script = include_str!("../scripts/reverse_sort.applescript");
            let _ = self.execute_applescript(reverse_script);
        }

        eprintln!("Finder window sorted successfully!");
        Ok(())
    }

    fn sort_recursively(
        &self,
        path: &Path,
        sort_by: &SortBy,
        order: &SortOrder,
    ) -> Result<(), String> {
        eprintln!("Finding all subdirectories...");
        let directories = self.get_all_subdirectories(path)?;

        let dir_count = directories.len();
        eprintln!(
            "Found {} director{}",
            dir_count,
            if dir_count == 1 { "y" } else { "ies" }
        );

        for (index, dir) in directories.iter().enumerate() {
            eprintln!(
                "[{}/{}] Processing: {}",
                index + 1,
                dir_count,
                dir.display()
            );
            self.set_folder_sort_preferences_background(dir, sort_by, order)?;
        }

        eprintln!("All folders sorted!");
        Ok(())
    }
}

// ============================================================================
// File Organizer
// ============================================================================

struct FileOrganizer {
    verbose: bool,
}

impl FileOrganizer {
    const fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    fn log(&self, message: impl AsRef<str>) {
        if self.verbose {
            eprintln!("{}", message.as_ref());
        }
    }

    fn organize(&self, dir_path: &Path) -> Result<(usize, usize), String> {
        if !dir_path.exists() {
            return Err(format!(
                "Directory \"{}\" doesn't exist",
                dir_path.display()
            ));
        }

        if !dir_path.is_dir() {
            return Err(format!(
                "Path \"{}\" is not a directory",
                dir_path.display()
            ));
        }

        let entries = fs::read_dir(dir_path)
            .map_err(|e| format!("Error opening directory \"{}\": {}", dir_path.display(), e))?;

        let mut files_moved = 0;
        let mut files_skipped = 0;

        for entry in entries {
            let file = entry.map_err(|e| format!("Error reading directory entry: {}", e))?;
            let file_path = file.path();

            if file_path.is_dir() {
                self.log(format!("Skipping directory: {}", file_path.display()));
                files_skipped += 1;
                continue;
            }

            let extension = match file_path.extension().and_then(|e| e.to_str()) {
                Some(ext) => ext.to_lowercase(),
                None => {
                    eprintln!("Skipping file without extension: {}", file_path.display());
                    files_skipped += 1;
                    continue;
                }
            };

            let extension_dir = dir_path.join(&extension);
            Self::create_dir_if_not_exists(&extension_dir)?;

            let destination = extension_dir.join(file.file_name());
            Self::move_file(&file_path, &destination)?;
            files_moved += 1;
        }

        Ok((files_moved, files_skipped))
    }

    fn create_dir_if_not_exists(dir_path: &Path) -> Result<(), String> {
        if !dir_path.exists() {
            fs::create_dir(dir_path).map_err(|e| {
                format!("Error creating directory \"{}\": {}", dir_path.display(), e)
            })?;
        }
        Ok(())
    }

    fn move_file(from: &Path, to: &Path) -> Result<(), String> {
        fs::rename(from, to).map_err(|e| {
            format!(
                "Error moving \"{}\" to \"{}\": {}",
                from.display(),
                to.display(),
                e
            )
        })
    }
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Result<(), String> {
    let args = Args::parse();

    if args.pack_to_folders {
        eprintln!("WARNING: This operation will reorganize your directory structure!");
        eprintln!("Organizing files in: {}\n", args.path.display());

        let start = Instant::now();
        let organizer = FileOrganizer::new(args.verbose);
        let (moved, skipped) = organizer.organize(&args.path)?;

        eprintln!("\nFiles moved: {}, skipped: {}", moved, skipped);
        eprintln!("Completed in {:.3}s", start.elapsed().as_secs_f64());
    } else {
        let sorter = FinderSorter::new(args.verbose);

        if args.recursive {
            eprintln!("Recursive mode enabled - sorting all nested folders\n");
            sorter.sort_recursively(&args.path, &args.sort, &args.order)?;
        } else {
            sorter.sort_finder_window(&args.path, &args.sort, &args.order)?;
        }

        eprintln!("\nTask successfully completed!");
    }

    Ok(())
}
