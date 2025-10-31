# finder-files-organizer

Simple Finder files CLI organizer for macOS

## Getting Started

### Install

Ensure you have `Rust` installed via [rustup](https://rustup.rs).

Use `cargo` to install `finder-files-organizer` from the Git repository:

```
cargo install --git https://github.com/gaiwerspace/Rust/finder-files-organizer
```

Or you can clone the Git repository and build from the source:

```
git clone https://github.com/gaiwerspace/Rust/finder-files-organizer
cd finder-files-organizer
cargo build --release
```

### Preparation

After running the command cargo build --release, you will have a binary file that you can use it from the folder:
./target/release/finder-finder-files-organizer

### Use

Select the folder in which you want to sort the folders and files and execute the command.

USAGE:
  finder-sorter <PATH> [OPTIONS]

    ARGUMENTS:
      <PATH>    Directory to open and sort

      OPTIONS:
        -s, --sort <SORT>     Sort by: name, modified, created, size, type, tags [default: type]
        -o, --order <ORDER>   Order: asc, desc [default: asc]
        -r, --recursive       Recursively sort all nested folders
        -v, --verbose         Verbose output

Examples of available commands:

Test with the basic sorting options first.
Sort by name (no tags needed)

./target/release/finder-files-organizer /YOUR_SELECTED_FOLDER -s name

Sort by modification date

./target/release/finder-files-organizer /YOUR_SELECTED_FOLDER -s modified -o desc

Sort by size

./target/release/finder-files-organizer /YOUR_SELECTED_FOLDER -s size -o desc

Sort by type

./target/release/finder-files-organizer /YOUR_SELECTED_FOLDER -s type

Recursively sort all folders in YOUR_SELECTED_FOLDER by type

./target/release/finder-files-organizer /YOUR_SELECTED_FOLDER -r

Examples (sorting the folders and files in the folder Downloads):

Sort by name (no tags needed)

./target/release/finder-files-organizer /Downloads -s name

Sort by modification date

./target/release/finder-files-organizer /Downloads -s modified -o desc

Sort by size

./target/release/finder-files-organizer /Downloads -s size -o desc

Sort by type

./target/release/finder-files-organizer /Downloads -s type

After the first launch, macOS will ask you to grant the program permission to perform the relevant actions.
Please grant it the following rights:

1. “Terminal“ wants access to control “Finder“. Allowing control will provide access to documents and data in “Finder“, and to perform actions within that app. -> сlick on button "Allow"

2. “Terminal“ wants access to control “System Events“. Allowing control will provide access to documents and data in “System Events“, and to perform actions within that app. -> сlick on button "Allow"

3. Accessibility Access (Events)

"Terminal" would like to control this computer using accessibility features.
Grant access to this application in Privacy & Security settings, located in System Settings. -> click on button "Open System Settings"

4. Please grant Accessibility Access to the application that runs this program (e.g., Terminal).
