tell application "System Events"
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
end tell
