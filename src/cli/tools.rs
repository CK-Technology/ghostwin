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
    
    // Detect individual tools
    let mut total_tools = 0;
    
    for tool_dir in &tool_dirs {
        let tools = detector.detect_tools(tool_dir)?;
        
        if !tools.is_empty() {
            println!("\n📂 Tools in {}:", tool_dir.display());
            
            for tool in &tools {
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
            
            total_tools += tools.len();
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