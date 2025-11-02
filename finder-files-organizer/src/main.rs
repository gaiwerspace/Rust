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

        let open_script = self.build_applescript(
            include_str!("../scripts/open_folder.applescript"),
            &[("{FOLDER_PATH}", &dir_str)],
        );
        self.execute_applescript(&open_script)?;

        let sort_script = self.build_applescript(
            include_str!("../scripts/sort_keyboard.applescript"),
            &[("{KEY_CODE}", sort_by.key_code())],
        );
        self.execute_applescript(&sort_script)?;

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
            "Found {} director{} to sort",
            dir_count,
            if dir_count == 1 { "y" } else { "ies" }
        );

        for (index, dir) in directories.iter().enumerate() {
            eprintln!("[{}/{}] Sorting: {}", index + 1, dir_count, dir.display());
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
                    self.log(format!(
                        "Skipping file without extension: {}",
                        file_path.display()
                    ));
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
// Main function
// ============================================================================

fn main() -> Result<(), String> {
    let args = Args::parse();

    // Determine sort options (with defaults)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    // ========================================================================
    // Helper Functions
    // ========================================================================

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    fn create_test_dir_structure() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create test files with different extensions
        create_test_file(temp_dir.path(), "file1.txt", "content1");
        create_test_file(temp_dir.path(), "file2.txt", "content2");
        create_test_file(temp_dir.path(), "file3.md", "markdown");
        create_test_file(temp_dir.path(), "file4.rs", "rust code");
        create_test_file(temp_dir.path(), "noext", "no extension");

        // Create a subdirectory
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        create_test_file(&sub_dir, "nested.txt", "nested content");

        temp_dir
    }

    // ========================================================================
    // Path Parsing Tests
    // ========================================================================

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

    // ========================================================================
    // SortBy Enum Tests
    // ========================================================================

    #[test]
    fn test_sortby_key_codes() {
        assert_eq!(SortBy::Name.key_code(), "18");
        assert_eq!(SortBy::Type.key_code(), "19");
        assert_eq!(SortBy::Modified.key_code(), "20");
        assert_eq!(SortBy::Created.key_code(), "21");
        assert_eq!(SortBy::Size.key_code(), "23");
        assert_eq!(SortBy::Tags.key_code(), "22");
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

    // ========================================================================
    // SortOrder Enum Tests
    // ========================================================================

    #[test]
    fn test_sortorder_direction() {
        assert_eq!(SortOrder::Asc.direction(), "normal");
        assert_eq!(SortOrder::Desc.direction(), "reversed");
    }

    // ========================================================================
    // FinderSorter Tests
    // ========================================================================

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
        assert!(result.unwrap_err().contains("Path does not exist"));
    }

    #[test]
    fn test_validate_directory_is_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file(temp_dir.path(), "test.txt", "content");
        let sorter = FinderSorter::new(false);

        let result = sorter.validate_directory(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a directory"));
    }

    #[test]
    fn test_build_applescript() {
        let sorter = FinderSorter::new(false);
        let template = "set folder to {FOLDER_PATH} and sort by {SORT_COLUMN}";
        let replacements = [
            ("{FOLDER_PATH}", "/test/path"),
            ("{SORT_COLUMN}", "name column"),
        ];

        let result = sorter.build_applescript(template, &replacements);
        assert_eq!(result, "set folder to /test/path and sort by name column");
    }

    #[test]
    fn test_get_all_subdirectories() {
        let temp_dir = create_test_dir_structure();
        let sorter = FinderSorter::new(false);

        let result = sorter.get_all_subdirectories(temp_dir.path());
        assert!(result.is_ok());

        let dirs = result.unwrap();
        assert_eq!(dirs.len(), 2); // root + subdir
        assert!(dirs.contains(&temp_dir.path().to_path_buf()));
        assert!(dirs.contains(&temp_dir.path().join("subdir")));
    }

    #[test]
    fn test_get_all_subdirectories_nested() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested directory structure
        let level1 = temp_dir.path().join("level1");
        fs::create_dir(&level1).unwrap();
        let level2 = level1.join("level2");
        fs::create_dir(&level2).unwrap();
        let level3 = level2.join("level3");
        fs::create_dir(&level3).unwrap();

        let sorter = FinderSorter::new(false);
        let result = sorter.get_all_subdirectories(temp_dir.path());
        assert!(result.is_ok());

        let dirs = result.unwrap();
        assert_eq!(dirs.len(), 4); // root + 3 levels
    }

    // ========================================================================
    // FileOrganizer Tests
    // ========================================================================

    #[test]
    fn test_file_organizer_new() {
        let organizer = FileOrganizer::new(true);
        assert!(organizer.verbose);

        let organizer = FileOrganizer::new(false);
        assert!(!organizer.verbose);
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

        // Verify extension folders were created
        assert!(temp_dir.path().join("txt").is_dir());
        assert!(temp_dir.path().join("md").is_dir());
        assert!(temp_dir.path().join("rs").is_dir());

        // Verify files were moved
        assert!(temp_dir.path().join("txt/file1.txt").exists());
        assert!(temp_dir.path().join("txt/file2.txt").exists());
        assert!(temp_dir.path().join("md/file3.md").exists());
        assert!(temp_dir.path().join("rs/file4.rs").exists());
    }

    #[test]
    fn test_organize_nonexistent_directory() {
        let organizer = FileOrganizer::new(false);
        let non_existent = PathBuf::from("/path/that/does/not/exist");

        let result = organizer.organize(&non_existent);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("doesn't exist"));
    }

    #[test]
    fn test_organize_file_not_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file(temp_dir.path(), "test.txt", "content");
        let organizer = FileOrganizer::new(false);

        let result = organizer.organize(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a directory"));
    }

    #[test]
    fn test_organize_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let organizer = FileOrganizer::new(false);

        let result = organizer.organize(temp_dir.path());
        assert!(result.is_ok());

        let (moved, skipped) = result.unwrap();
        assert_eq!(moved, 0);
        assert_eq!(skipped, 0);
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

        // Test idempotency - should not error if dir already exists
        let result = FileOrganizer::create_dir_if_not_exists(&new_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_move_file() {
        let temp_dir = TempDir::new().unwrap();
        let source = create_test_file(temp_dir.path(), "source.txt", "content");
        let dest = temp_dir.path().join("destination.txt");

        assert!(source.exists());
        assert!(!dest.exists());

        let result = FileOrganizer::move_file(&source, &dest);
        assert!(result.is_ok());

        assert!(!source.exists());
        assert!(dest.exists());
    }

    #[test]
    fn test_move_file_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("nonexistent.txt");
        let dest = temp_dir.path().join("destination.txt");

        let result = FileOrganizer::move_file(&source, &dest);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_directories() {
        let temp_dir = create_test_dir_structure();
        let organizer = FileOrganizer::new(false);

        let result = organizer.get_all_directories(temp_dir.path());
        assert!(result.is_ok());

        let dirs = result.unwrap();
        assert_eq!(dirs.len(), 2); // root + subdir
    }
}
