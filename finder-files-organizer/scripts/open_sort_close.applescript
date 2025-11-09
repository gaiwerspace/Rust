-- Open Finder window, sort it, and close it
on run argv
	set folderPath to item 1 of argv
	set sortColumn to item 2 of argv
	set sortDirection to item 3 of argv
	
	tell application "Finder"
		-- Open the folder in a new Finder window
		set targetFolder to POSIX file folderPath as alias
		set newWindow to make new Finder window to targetFolder
		
		-- Set view to list view
		set current view of newWindow to list view
		
		-- Sort by specified column using column ID syntax
		try
			set sort column of list view options of newWindow to column id sortColumn
			set sorted of list view options of newWindow to sortDirection
		on error errMsg
			-- Fallback: try using the column name directly
			try
				set sort column of list view options of newWindow to column sortColumn
				set sorted of list view options of newWindow to sortDirection
			on error
				-- Last resort: use UI scripting
				tell application "System Events"
					tell process "Finder"
						tell window 1
							-- Try to find and click the column header
							try
								set columnHeader to button sortColumn of group 1 of scroll area 1
								click columnHeader
								if sortDirection is "reversed" then
									delay 0.2
									click columnHeader
								end if
							on error
								-- Try alternative UI structure
								try
									set columnHeader to button sortColumn of scroll area 1
									click columnHeader
									if sortDirection is "reversed" then
										delay 0.2
										click columnHeader
									end if
								on error
									-- If all else fails, just leave it as is
								end try
							end try
						end tell
					end tell
				end tell
			end try
		end try
		
		-- Wait a moment for the sort to complete
		delay 0.5
		
		-- Close the window
		close newWindow
	end tell
	
	return "Folder sorted and window closed: " & folderPath
end run