tell application "Finder"
    activate
    set theFolder to POSIX file "{FOLDER_PATH}" as alias
    open theFolder
    delay 0.8
    tell front window
        set current view to list view
    end tell
end tell
