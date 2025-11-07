on run argv
    set folderPath to item 1 of argv

    tell application "Finder"
        activate
        open POSIX file folderPath as alias
        set target of front window to POSIX file folderPath as alias
    end tell
end run
