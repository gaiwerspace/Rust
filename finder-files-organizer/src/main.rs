use clap::{Parser, ValueEnum};
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

// ============================================================================
// Enums
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
    /// Use column IDs to sort column
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

    /// Sort by: name, modified, created, size, type, tags [default: type]
    #[arg(short, long, value_enum)]
    sort: Option<SortBy>,

    /// Order: asc, desc [default: asc]
    #[arg(short, long, value_enum)]
    order: Option<SortOrder>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Recursively process all nested folders
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

    /// Execute AppleScript with arguments surpassed through stdin (secure from injection)
    fn execute_applescript_with_args(&self, script: &str, args: &[&str]) -> Result<String, String> {
        self.log(" Executing AppleScript...");

        let mut command = Command::new("osascript");
        command.arg("-"); // Read script from stdin
        
        // Add all arguments
        for arg in args {
            command.arg(arg);
        }
        
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command
            .spawn()
            .map_err(|e| format!("Failed to spawn osascript: {e}"))?;

        // Write the script to stdin
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(script.as_bytes())
                .map_err(|e| format!("Could not write script to stdin: {e}"))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to execute AppleScript: {e}"))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("Error of AppleScript: {}", error.trim()))
        }
    }

    /// Run AppleScript and return results (unexpected - now using execute_applescript_with_args)
    fn execute_applescript(&self, script: &str) -> Result<String, String> {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| format!("Failed to execute AppleScript: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(format!(
                "Error of AppleScript: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Find AppleScript file in multiple possible locations
    fn find_applescript_file(&self, filename: &str) -> Result<PathBuf, String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to retrieve executable path:{}", e))?;
        let exe_dir = exe_path.parent()
            .ok_or("Could not determine executable directory")?;
        
        // Try multiple possible locations
        let possible_paths = vec![
            // Try in the same directory as the executable (for release builds)
            exe_dir.join("scripts").join(filename),
            // Try in the parent directory (for debug builds in target/debug)
            exe_dir.parent().unwrap_or(exe_dir).join("scripts").join(filename),
            // Try in the project root (development mode)
            exe_dir.parent().unwrap_or(exe_dir).parent().unwrap_or(exe_dir).join("scripts").join(filename),
            // Try current working directory
            PathBuf::from("scripts").join(filename),
        ];
        
        // Create error message with paths before consuming them
        let error_paths: Vec<String> = possible_paths.iter().map(|p| p.display().to_string()).collect();
        
        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }
        
        Err(format!(
            "Could not find AppleScript file '{}' in any of these locations: {:?}", 
            filename, 
            error_paths
        ))
    }

     /// Recursively fetch all subdirectories, except symlinks, to stop the cycle
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

                // Skip symlinks to prevent cycles
                let file_type = entry.file_type()?;
                if file_type.is_symlink() {
                    continue;
                }

                if path.is_dir() {
                    dirs.push(path.clone());
                    visit_dirs(&path, dirs)?;
                }
            }
            Ok(())
        }

        visit_dirs(root, &mut directories)
            .map_err(|e| format!("Could not traverse directories: {e}"))?;

        Ok(directories)
    }

    /// Set folder sort preferences in background (for recursive mode) - now unused but kept for reference
    /// #[allow(dead_code)] uses in Rust to disables compiler warnings about unused code.
    #[allow(dead_code)]
    fn set_folder_sort_preferences_background(
        &self,
        path: &Path,
        sort_by: &SortBy,
        order: &SortOrder,
    ) -> Result<(), String> {
        self.validate_directory(path)?;

        // Try multiple possible locations for the scripts
        let script_path = self.find_applescript_file("background_sort.applescript")?;
        let script = fs::read_to_string(&script_path)
            .map_err(|e| format!("Failed to read AppleScript file at {}: {}", script_path.display(), e))?;

        let path_str = path.to_string_lossy();
        self.execute_applescript_with_args(
            &script,
            &[&path_str, sort_by.sort_column(), order.direction()],
        )?;

        Ok(())
    }

    /// Open a Finder window and apply sort settings
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

        // Try multiple possible locations for the scripts
        let script_path = self.find_applescript_file("foreground_sort.applescript")?;
        let script = std::fs::read_to_string(&script_path)
            .map_err(|e| format!("Failed to read AppleScript file at {}: {}", script_path.display(), e))?;

        let path_str = path.to_string_lossy();
        self.execute_applescript_with_args(
            &script,
            &[&path_str, sort_by.sort_column(), order.direction()],
        )?;

        eprintln!("Finder window sorted successfully!");
        Ok(())
    }

    /// Open a Finder window, sort it, and close it (for recursive mode)
    fn sort_finder_window_with_close(
        &self,
        path: &Path,
        sort_by: &SortBy,
        order: &SortOrder,
    ) -> Result<(), String> {
        self.validate_directory(path)?;

        eprintln!("Opening folder: {}", path.display());
        eprintln!("Sort by: {sort_by:?}");
        eprintln!("Order: {order:?}");

        // Try multiple possible locations for the scripts
        let script_path = self.find_applescript_file("open_sort_close.applescript")?;
        let script = std::fs::read_to_string(&script_path)
            .map_err(|e| format!("Failed to read AppleScript file at {}: {}", script_path.display(), e))?;

        let path_str = path.to_string_lossy();
        let result = self.execute_applescript_with_args(
            &script,
            &[&path_str, sort_by.sort_column(), order.direction()],
        )?;

        eprintln!("{}", result);
        Ok(())
    }

    /// Sort all subdirectories recursively with open/close windows
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
            "Found {} director{} to sort",
            dir_count,
            if dir_count == 1 { "y" } else { "ies" }
        );

        for (index, dir) in directories.iter().enumerate() {
            eprintln!("[{}/{}] Sorting: {}", index + 1, dir_count, dir.display());
            self.sort_finder_window_with_close(dir, sort_by, order)?;
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

    /// Sort files in all subdirectories recursively
    fn organize_recursive(&self, root: &Path) -> Result<(usize, usize), String> {
        let mut total_moved = 0;
        let mut total_skipped = 0;

        let directories = self.get_all_directories(root)?;
        eprintln!(
            "Processing {} director{}...",
            directories.len(),
            if directories.len() == 1 { "y" } else { "ies" }
        );

        for (index, dir) in directories.iter().enumerate() {
            eprintln!(
                "[{}/{}] Organizing: {}",
                index + 1,
                directories.len(),
                dir.display()
            );
            let (moved, skipped) = self.organize(dir)?;
            total_moved += moved;
            total_skipped += skipped;
        }

        Ok((total_moved, total_skipped))
    }

    /// Get all directories recursively, except symlinks, to stop the cycle
    fn get_all_directories(&self, root: &Path) -> Result<Vec<PathBuf>, String> {
        let mut directories = Vec::with_capacity(16);
        directories.push(root.to_path_buf());

        fn visit_dirs(dir: &Path, dirs: &mut Vec<PathBuf>) -> io::Result<()> {
            if !dir.is_dir() {
                return Ok(());
            }

            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                // Skip symlinks to prevent cycles
                let file_type = entry.file_type()?;
                if file_type.is_symlink() {
                    continue;
                }

                if path.is_dir() {
                    dirs.push(path.clone());
                    visit_dirs(&path, dirs)?;
                }
            }
            Ok(())
        }

        visit_dirs(root, &mut directories)
            .map_err(|e| format!("Could not traverse directories: {e}"))?;

        Ok(directories)
    }

   /// Organize files in the same directory by extension
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

            // Skip directories
            if file_path.is_dir() {
                self.log(format!("Skipping directory: {}", file_path.display()));
                files_skipped += 1;
                continue;
            }

            // Get file extension
            let extension = match file_path.extension().and_then(|e| e.to_str()) {
                Some(ext) => ext.to_lowercase(),
                None => {
                    self.log(format!(
                        "Skipping file without extension: {}",
                        file_path.display()
                    ));
                    files_skipped += 1;
                    continue;
                }
            };

            // Create extension directory
            let extension_dir = dir_path.join(&extension);
            Self::create_dir_if_not_exists(&extension_dir)?;

            // Check existing file and create a unique name if necessary
            let filename = file.file_name();
            let mut destination = extension_dir.join(&filename);
            
            if destination.exists() {
                destination = Self::get_unique_filename(&extension_dir, &filename)?;
                self.log(format!(
                    "File already exists, using unique name: {}",
                    destination.display()
                ));
            }

            Self::move_file(&file_path, &destination)?;
            files_moved += 1;
        }

        Ok((files_moved, files_skipped))
    }

    /// Create a unique filename to prevent overwriting
    fn get_unique_filename(dir: &Path, original_name: &std::ffi::OsStr) -> Result<PathBuf, String> {
        let name_str = original_name.to_string_lossy();
        let path = Path::new(name_str.as_ref());
        
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        
        // Try incrementing counter until we find an available name
        for i in 1..10000 {
            let new_name = if extension.is_empty() {
                format!("{} ({})", stem, i)
            } else {
                format!("{} ({}).{}", stem, i, extension)
            };
            
            let candidate = dir.join(&new_name);
            if !candidate.exists() {
                return Ok(candidate);
            }
        }
        
        Err("Could not generate unique filename after 10000 attempts".to_string())
    }

    /// Create the directory if it does not exist
    fn create_dir_if_not_exists(dir_path: &Path) -> Result<(), String> {
        if !dir_path.exists() {
            fs::create_dir(dir_path).map_err(|e| {
                format!("Error creating directory \"{}\": {}", dir_path.display(), e)
            })?;
        }
        Ok(())
    }

    /// Move the file from source to destination
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
// Main function
// ============================================================================

fn main() -> Result<(), String> {
    let args = Args::parse();

    /// Set sorting options (with defaults)
    let sort_by = args.sort.as_ref().unwrap_or(&SortBy::Type);
    let order = args.order.as_ref().unwrap_or(&SortOrder::Asc);

    if args.pack_to_folders {
        eprintln!("WARNING: This operation will reorganize your directory structure!");
        eprintln!("Organizing files in: {}\n", args.path.display());

        let start = Instant::now();
        let organizer = FileOrganizer::new(args.verbose);

        let (moved, skipped) = if args.recursive {
            eprintln!("Recursive mode enabled - organizing all nested folders\n");
            organizer.organize_recursive(&args.path)?
        } else {
            organizer.organize(&args.path)?
        };

        eprintln!("\nFiles moved: {}, skipped: {}", moved, skipped);
        eprintln!("Completed in {:.3}s", start.elapsed().as_secs_f64());

        // After organizing, apply sorting if sort/order flags were provided
        if args.sort.is_some() || args.order.is_some() {
            eprintln!("\nApplying sort preferences...");
            let sorter = FinderSorter::new(args.verbose);

            if args.recursive {
                sorter.sort_recursively(&args.path, sort_by, order)?;
            } else {
                sorter.sort_finder_window(&args.path, sort_by, order)?;
            }
        }
    } else {
        let sorter = FinderSorter::new(args.verbose);

        if args.recursive {
            eprintln!("Recursive mode enabled - sorting all nested folders\n");
            sorter.sort_recursively(&args.path, sort_by, order)?;
        } else {
            sorter.sort_finder_window(&args.path, sort_by, order)?;
        }
    }

    eprintln!("\nTask successfully completed!");
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    fn create_test_dir_structure() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        create_test_file(temp_dir.path(), "file1.txt", "content1");
        create_test_file(temp_dir.path(), "file2.txt", "content2");
        create_test_file(temp_dir.path(), "file3.md", "markdown");
        create_test_file(temp_dir.path(), "file4.rs", "rust code");
        create_test_file(temp_dir.path(), "noext", "no extension");

        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        create_test_file(&sub_dir, "nested.txt", "nested content");

        temp_dir
    }

    #[test]
    fn test_parse_path_regular() {
        let result = parse_path("/tmp/test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_parse_path_relative() {
        let result = parse_path("relative/path");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("relative/path"));
    }

    #[test]
    fn test_sortby_sort_columns() {
        assert_eq!(SortBy::Name.sort_column(), "name column");
        assert_eq!(SortBy::Type.sort_column(), "kind column");
        assert_eq!(SortBy::Modified.sort_column(), "modification date column");
        assert_eq!(SortBy::Created.sort_column(), "creation date column");
        assert_eq!(SortBy::Size.sort_column(), "size column");
        assert_eq!(SortBy::Tags.sort_column(), "label column");
    }

    #[test]
    fn test_sortorder_direction() {
        assert_eq!(SortOrder::Asc.direction(), "normal");
        assert_eq!(SortOrder::Desc.direction(), "reversed");
    }

    #[test]
    fn test_finder_sorter_new() {
        let sorter = FinderSorter::new(true);
        assert!(sorter.verbose);

        let sorter = FinderSorter::new(false);
        assert!(!sorter.verbose);
    }

    #[test]
    fn test_validate_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let sorter = FinderSorter::new(false);
        let result = sorter.validate_directory(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_directory_not_exists() {
        let sorter = FinderSorter::new(false);
        let non_existent = PathBuf::from("/path/that/does/not/exist");
        let result = sorter.validate_directory(&non_existent);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_subdirectories() {
        let temp_dir = create_test_dir_structure();
        let sorter = FinderSorter::new(false);
        let result = sorter.get_all_subdirectories(temp_dir.path());
        assert!(result.is_ok());
        let dirs = result.unwrap();
        assert_eq!(dirs.len(), 2); // root + subdir
    }

    #[test]
    fn test_organize_basic() {
        let temp_dir = create_test_dir_structure();
        let organizer = FileOrganizer::new(false);
        let result = organizer.organize(temp_dir.path());
        assert!(result.is_ok());
        let (moved, skipped) = result.unwrap();
        assert_eq!(moved, 4); // 4 files with extensions
        assert_eq!(skipped, 2); // 1 file without extension + 1 subdirectory
    }

    #[test]
    fn test_organize_recursive() {
        let temp_dir = create_test_dir_structure();
        let organizer = FileOrganizer::new(false);
        let result = organizer.organize_recursive(temp_dir.path());
        assert!(result.is_ok());
        let (total_moved, total_skipped) = result.unwrap();
        assert!(total_moved > 0);
        assert!(total_skipped > 0);
    }

    #[test]
    fn test_create_dir_if_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("newdir");
        assert!(!new_dir.exists());
        let result = FileOrganizer::create_dir_if_not_exists(&new_dir);
        assert!(result.is_ok());
        assert!(new_dir.exists());
    }

    #[test]
    fn test_get_unique_filename() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(temp_dir.path(), "test.txt", "content");
        let result = FileOrganizer::get_unique_filename(
            temp_dir.path(),
            std::ffi::OsStr::new("test.txt")
        );
        assert!(result.is_ok());
        let unique_name = result.unwrap();
        assert_eq!(unique_name.file_name().unwrap(), "test (1).txt");
    }
}
