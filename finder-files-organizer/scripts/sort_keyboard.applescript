tell application "System Events"
    tell process "Finder"
        set frontmost to true
        delay 0.2
        key code {KEY_CODE} using {control down, option down, command down}
        delay 0.3
    end tell
end tell
