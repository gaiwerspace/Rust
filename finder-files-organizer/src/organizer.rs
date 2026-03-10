use std::path::PathBuf;
use crate::{SortBy, SortOrder};

pub struct FileOrganizer {
    pub path: PathBuf,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub recursive: bool,
    pub pack_to_folders: bool,
}

impl FileOrganizer {
    pub fn new(
        path: PathBuf,
        sort_by: SortBy,
        sort_order: SortOrder,
        recursive: bool,
        pack_to_folders: bool,
    ) -> Self {
        Self {
            path,
            sort_by,
            sort_order,
            recursive,
            pack_to_folders,
        }
    }

    pub async fn organize(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut args = vec![self.path.to_string_lossy().to_string()];

        // Add sort option
        args.push("--sort".to_string());
        args.push(match self.sort_by {
            SortBy::Name => "name",
            SortBy::Modified => "modified",
            SortBy::Created => "created",
            SortBy::Size => "size",
            SortBy::Type => "type",
            SortBy::Tags => "tags",
        }.to_string());

        // Add order option
        args.push("--order".to_string());
        args.push(match self.sort_order {
            SortOrder::Asc => "asc",
            SortOrder::Desc => "desc",
        }.to_string());

        // Add recursive option if needed
        if self.recursive {
            args.push("--recursive".to_string());
        }

        // Add pack to folders option if needed
        if self.pack_to_folders {
            args.push("--pack-to-folders".to_string());
        }

        // Execute the organization logic
        self.execute_organization(&args).await
    }

    async fn execute_organization(&self, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        // This would integrate with the existing organization logic
        // For now, we'll simulate the process
        
        println!("Organizing files with args: {:?}", args);
        
        // Simulate organization process
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        println!("Organization completed for: {:?}", self.path);
        
        Ok(())
    }

    // Getters
    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn get_sort_by(&self) -> &SortBy {
        &self.sort_by
    }

    pub fn get_sort_order(&self) -> &SortOrder {
        &self.sort_order
    }

    pub fn get_recursive(&self) -> bool {
        self.recursive
    }

    pub fn get_pack_to_folders(&self) -> bool {
        self.pack_to_folders
    }
}
