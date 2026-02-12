on run argv
    set folderPath to item 1 of argv
    set sortColumnName to item 2 of argv
    set sortDir to item 3 of argv
    
    tell application "Finder"
        try
            set targetFolder to POSIX file folderPath as alias
            
            -- Open the folder in the background (no activate)
            open targetFolder
            delay 0.3
            
            set targetWindow to front window
            set target of targetWindow to targetFolder
            set current view of targetWindow to list view
            
            delay 0.2
            
            -- Workaround for Finder AppleScript bugs: use column id syntax
            tell list view options of targetWindow
                if sortColumnName is "name column" then
                    set sort column to column id name column
                    tell column id name column
                        if sortDir is "normal" then
                            set sort direction to normal
                        else
                            set sort direction to reversed
                        end if
                    end tell
                else if sortColumnName is "kind column" then
                    set sort column to column id kind column
                    tell column id kind column
                        if sortDir is "normal" then
                            set sort direction to normal
                        else
                            set sort direction to reversed
                        end if
                    end tell
                else if sortColumnName is "modification date column" then
                    set sort column to column id modification date column
                    tell column id modification date column
                        if sortDir is "normal" then
                            set sort direction to normal
                        else
                            set sort direction to reversed
                        end if
                    end tell
                else if sortColumnName is "creation date column" then
                    set sort column to column id creation date column
                    tell column id creation date column
                        if sortDir is "normal" then
                            set sort direction to normal
                        else
                            set sort direction to reversed
                        end if
                    end tell
                else if sortColumnName is "size column" then
                    set sort column to column id size column
                    tell column id size column
                        if sortDir is "normal" then
                            set sort direction to normal
                        else
                            set sort direction to reversed
                        end if
                    end tell
                else if sortColumnName is "label column" then
                    set sort column to column id label column
                    tell column id label column
                        if sortDir is "normal" then
                            set sort direction to normal
                        else
                            set sort direction to reversed
                        end if
                    end tell
                else
                    set sort column to column id name column
                    tell column id name column
                        if sortDir is "normal" then
                            set sort direction to normal
                        else
                            set sort direction to reversed
                        end if
                    end tell
                end if
            end tell
            
            -- Force window refresh to apply changes
            set bounds of targetWindow to bounds of targetWindow
            
        on error errMsg
            error "Failed to set sort preferences: " & errMsg
        end try
    end tell
end run