use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tray_icon::{TrayIconBuilder, TrayIconEvent, Icon, menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem}};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{SortBy, SortOrder, FileOrganizer};

pub struct TrayApp {
    pub folder_path: Arc<Mutex<Option<PathBuf>>>,
    pub sort_by: Arc<Mutex<SortBy>>,
    pub sort_order: Arc<Mutex<SortOrder>>,
    pub recursive: Arc<Mutex<bool>>,
    pub pack_to_folders: Arc<Mutex<bool>>,
}

impl TrayApp {
    pub fn new() -> Self {
        Self {
            folder_path: Arc::new(Mutex::new(None)),
            sort_by: Arc::new(Mutex::new(SortBy::Type)),
            sort_order: Arc::new(Mutex::new(SortOrder::Asc)),
            recursive: Arc::new(Mutex::new(false)),
            pack_to_folders: Arc::new(Mutex::new(false)),
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;

        // Load icon from assets
        let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.rgba");
        let icon_data = std::fs::read(icon_path)?;
        let icon = Icon::from_rgba(icon_data, 16, 16)?;

        // Create menu
        let menu = Menu::new();
        let sort_by_name = MenuItem::new("  Sort by Name", true, None);
        let sort_by_modified = MenuItem::new("  Sort by Modified Date", true, None);
        let sort_by_created = MenuItem::new("  Sort by Created Date", true, None);
        let sort_by_size = MenuItem::new("  Sort by Size", true, None);
        let sort_by_type = MenuItem::new("  Sort by Type", true, None);
        let sort_by_tags = MenuItem::new("  Sort by Tags", true, None);
        let set_folder_path = MenuItem::new("Set Folder Path...", true, None);
        let organize_files = MenuItem::new("Organize Files", true, None);
        
        // Handle tray events
        let tray_event_receiver = TrayIconEvent::receiver();
        let menu_event_receiver = MenuEvent::receiver();
        let folder_path = self.folder_path.clone();
        let sort_by = self.sort_by.clone();
        let sort_order = self.sort_order.clone();
        let recursive = self.recursive.clone();
        let pack_to_folders = self.pack_to_folders.clone();
        
        // Set initial checkbox state based on current sort_by setting
        let current_sort = sort_by.lock().unwrap().clone();
        update_menu_checkboxes(&sort_by_name, &sort_by_modified, &sort_by_created, &sort_by_size, &sort_by_type, &sort_by_tags, current_sort);
        
        menu.append(&sort_by_name)?;
        menu.append(&sort_by_modified)?;
        menu.append(&sort_by_created)?;
        menu.append(&sort_by_size)?;
        menu.append(&sort_by_type)?;
        menu.append(&sort_by_tags)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&set_folder_path)?;
        menu.append(&organize_files)?;

        // Create tray icon
        let _tray_icon = TrayIconBuilder::new()
            .with_tooltip("Finder Files Organizer - Right click for options")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build()?;

        event_loop.run(move |event, event_loop| {
            event_loop.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    event_loop.exit();
                }
                Event::NewEvents(_) => {
                    // Handle tray events
                    while let Ok(tray_event) = tray_event_receiver.try_recv() {
                        match tray_event {
                            TrayIconEvent::Click { .. } => {
                                handle_tray_click(
                                    &folder_path,
                                    &sort_by,
                                    &sort_order,
                                    &recursive,
                                    &pack_to_folders,
                                );
                            }
                            _ => {}
                        }
                    }
                    
                    // Handle menu events
                    while let Ok(menu_event) = menu_event_receiver.try_recv() {
                        match menu_event.id {
                            id if id == sort_by_name.id() => {
                                *sort_by.lock().unwrap() = SortBy::Name;
                                update_menu_checkboxes(&sort_by_name, &sort_by_modified, &sort_by_created, &sort_by_size, &sort_by_type, &sort_by_tags, SortBy::Name);
                            }
                            id if id == sort_by_modified.id() => {
                                *sort_by.lock().unwrap() = SortBy::Modified;
                                update_menu_checkboxes(&sort_by_name, &sort_by_modified, &sort_by_created, &sort_by_size, &sort_by_type, &sort_by_tags, SortBy::Modified);
                            }
                            id if id == sort_by_created.id() => {
                                *sort_by.lock().unwrap() = SortBy::Created;
                                update_menu_checkboxes(&sort_by_name, &sort_by_modified, &sort_by_created, &sort_by_size, &sort_by_type, &sort_by_tags, SortBy::Created);
                            }
                            id if id == sort_by_size.id() => {
                                *sort_by.lock().unwrap() = SortBy::Size;
                                update_menu_checkboxes(&sort_by_name, &sort_by_modified, &sort_by_created, &sort_by_size, &sort_by_type, &sort_by_tags, SortBy::Size);
                            }
                            id if id == sort_by_type.id() => {
                                *sort_by.lock().unwrap() = SortBy::Type;
                                update_menu_checkboxes(&sort_by_name, &sort_by_modified, &sort_by_created, &sort_by_size, &sort_by_type, &sort_by_tags, SortBy::Type);
                            }
                            id if id == sort_by_tags.id() => {
                                *sort_by.lock().unwrap() = SortBy::Tags;
                                update_menu_checkboxes(&sort_by_name, &sort_by_modified, &sort_by_created, &sort_by_size, &sort_by_type, &sort_by_tags, SortBy::Tags);
                            }
                            id if id == set_folder_path.id() => {
                                handle_set_folder_path(&folder_path);
                            }
                            id if id == organize_files.id() => {
                                handle_tray_click(
                                    &folder_path,
                                    &sort_by,
                                    &sort_order,
                                    &recursive,
                                    &pack_to_folders,
                                );
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        })?;

        Ok(())
    }
}

fn handle_tray_click(
    folder_path: &Arc<Mutex<Option<PathBuf>>>,
    sort_by: &Arc<Mutex<SortBy>>,
    sort_order: &Arc<Mutex<SortOrder>>,
    recursive: &Arc<Mutex<bool>>,
    pack_to_folders: &Arc<Mutex<bool>>,
) {
    // Show folder selection dialog
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Select folder to organize")
        .pick_folder()
    {
        *folder_path.lock().unwrap() = Some(path.clone());
        
        // Show settings dialog
        show_settings_dialog(sort_by, sort_order, recursive, pack_to_folders);
        
        // Create organizer using existing logic
        let organizer = FileOrganizer::new(true); // verbose=true
        
        // Show confirmation and organize
        if show_confirmation_dialog(&path, &*sort_by.lock().unwrap(), &*sort_order.lock().unwrap(), &*recursive.lock().unwrap(), &*pack_to_folders.lock().unwrap()) {
            std::thread::spawn(move || {
                if let Err(e) = organizer.organize(&path) {
                    eprintln!("Organization failed: {}", e);
                } else {
                    println!("✅ Organization completed successfully!");
                }
            });
        }
    }
}

fn show_confirmation_dialog(
    path: &PathBuf,
    sort_by: &SortBy,
    sort_order: &SortOrder,
    recursive: &bool,
    pack_to_folders: &bool,
) -> bool {
    println!("\n📁 Finder Files Organizer");
    println!("═════════════════════════════");
    println!("Folder: {:?}", path);
    println!("Sort by: {:?}", sort_by);
    println!("Order: {:?}", sort_order);
    println!("Recursive: {}", recursive);
    println!("Pack to folders: {}", pack_to_folders);
    println!("═════════════════════════════");
    print!("Continue with organization? (y/N): ");
    
    use std::io;
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let response = input.trim().to_lowercase();
    response == "y" || response == "yes"
}

fn show_settings_dialog(
    sort_by: &Arc<Mutex<SortBy>>,
    sort_order: &Arc<Mutex<SortOrder>>,
    recursive: &Arc<Mutex<bool>>,
    pack_to_folders: &Arc<Mutex<bool>>,
) {
    println!("\n⚙️  Organization Settings");
    println!("═════════════════════════════");
    
    println!("1. Sort by (name/modified/created/size/type/tags) [default: type]:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let sort_choice = input.trim().to_lowercase();
    if !sort_choice.is_empty() {
        *sort_by.lock().unwrap() = match sort_choice.as_str() {
            "name" => SortBy::Name,
            "modified" => SortBy::Modified,
            "created" => SortBy::Created,
            "size" => SortBy::Size,
            "type" => SortBy::Type,
            "tags" => SortBy::Tags,
            _ => SortBy::Type,
        };
    }
    
    println!("2. Sort order (asc/desc) [default: asc]:");
    input.clear();
    std::io::stdin().read_line(&mut input).unwrap();
    let order_choice = input.trim().to_lowercase();
    if !order_choice.is_empty() {
        *sort_order.lock().unwrap() = match order_choice.as_str() {
            "desc" => SortOrder::Desc,
            _ => SortOrder::Asc,
        };
    }
    
    println!("3. Recursive organization (y/N) [default: N]:");
    input.clear();
    std::io::stdin().read_line(&mut input).unwrap();
    let recursive_choice = input.trim().to_lowercase();
    *recursive.lock().unwrap() = recursive_choice == "y" || recursive_choice == "yes";
    
    println!("4. Pack files into folders by type (y/N) [default: N]:");
    input.clear();
    std::io::stdin().read_line(&mut input).unwrap();
    let pack_choice = input.trim().to_lowercase();
    *pack_to_folders.lock().unwrap() = pack_choice == "y" || pack_choice == "yes";
    
    println!("✅ Settings configured!");
}

fn update_menu_checkboxes(
    sort_by_name: &MenuItem,
    sort_by_modified: &MenuItem,
    sort_by_created: &MenuItem,
    sort_by_size: &MenuItem,
    sort_by_type: &MenuItem,
    sort_by_tags: &MenuItem,
    selected: SortBy,
) {
    // Reset all items to unchecked state
    let _ = sort_by_name.set_text("  Sort by Name");
    let _ = sort_by_modified.set_text("  Sort by Modified Date");
    let _ = sort_by_created.set_text("  Sort by Created Date");
    let _ = sort_by_size.set_text("  Sort by Size");
    let _ = sort_by_type.set_text("  Sort by Type");
    let _ = sort_by_tags.set_text("  Sort by Tags");
    
    // Set the selected item to checked state
    match selected {
        SortBy::Name => { let _ = sort_by_name.set_text("✓ Sort by Name"); }
        SortBy::Modified => { let _ = sort_by_modified.set_text("✓ Sort by Modified Date"); }
        SortBy::Created => { let _ = sort_by_created.set_text("✓ Sort by Created Date"); }
        SortBy::Size => { let _ = sort_by_size.set_text("✓ Sort by Size"); }
        SortBy::Type => { let _ = sort_by_type.set_text("✓ Sort by Type"); }
        SortBy::Tags => { let _ = sort_by_tags.set_text("✓ Sort by Tags"); }
    }
}

fn handle_set_folder_path(folder_path: &Arc<Mutex<Option<PathBuf>>>) {
    // Show folder selection dialog
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Select folder to organize")
        .pick_folder()
    {
        *folder_path.lock().unwrap() = Some(path.clone());
        println!("✅ Folder path set to: {:?}", path);
    }
}
