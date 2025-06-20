use anyhow::Result;
use tracing::info;
use crate::config::ConfigManager;
use crate::tools::ToolDetector;

pub async fn execute() -> Result<()> {
    info!("🔍 Scanning for tools and scripts");
    
    let config = ConfigManager::load_default()?;
    let detector = ToolDetector::new(&config.tools);
    
    // Scan for tool directories
    let tool_dirs = detector.scan_tools()?;
    
    if tool_dirs.is_empty() {
        println!("⚠️  No tool directories found");
        println!("Expected directories:");
        for folder in &config.tools.folders {
            println!("  - {}", folder);
        }
        return Ok(());
    }
    
    println!("📁 Found {} tool directories:", tool_dirs.len());
    for dir in &tool_dirs {
        println!("  - {}", dir.display());
    }
    
    // Detect individual tools from current directory
    let tools = detector.detect_tools(".")?;
    let total_tools = tools.len();
    
    if !tools.is_empty() {
        // Group tools by their containing directory for display
        use std::collections::HashMap;
        let mut tools_by_dir: HashMap<std::path::PathBuf, Vec<_>> = HashMap::new();
        
        for tool in &tools {
            let parent_dir = tool.path.parent().unwrap_or_else(|| std::path::Path::new("."));
            tools_by_dir.entry(parent_dir.to_path_buf()).or_default().push(tool);
        }
        
        for (dir, dir_tools) in tools_by_dir {
            println!("\n📂 Tools in {}:", dir.display());
            
            for tool in dir_tools {
                let category_icon = match tool.category {
                    crate::tools::ToolCategory::Tool => "🔧",
                    crate::tools::ToolCategory::PEAutoRun => "⚡",
                    crate::tools::ToolCategory::Logon => "🏁",
                };
                
                let visibility = if tool.hidden { " (hidden)" } else { "" };
                let auto_run = if tool.auto_run { " (auto-run)" } else { "" };
                
                println!("  {} {} {}{}{}", 
                    category_icon, 
                    tool.name, 
                    if tool.executable { "📋" } else { "📄" },
                    visibility,
                    auto_run
                );
            }
        }
    }
    
    // Load options files
    for tool_dir in &tool_dirs {
        if let Ok(options) = detector.load_options_file(tool_dir) {
            if options.check_all || options.collapse_tree || !options.default_checked.is_empty() {
                println!("\n⚙️  Options for {}:", tool_dir.display());
                
                if options.check_all {
                    println!("  - Check all items by default");
                }
                if options.collapse_tree {
                    println!("  - Collapse tree view by default");
                }
                if !options.default_checked.is_empty() {
                    println!("  - Default checked items: {}", options.default_checked.join(", "));
                }
            }
        }
    }
    
    println!("\n📊 Summary: {} tools found across {} directories", total_tools, tool_dirs.len());
    
    Ok(())
}