tell application "Finder"
    set theFolder to POSIX file "{FOLDER_PATH}" as alias
    set theWindow to make new Finder window
    set target of theWindow to theFolder
    set current view of theWindow to list view
    tell list view options of theWindow
        set sort column to {SORT_COLUMN}
        set sort direction of sort column to {SORT_DIRECTION}
    end tell
    close theWindow
end tell
