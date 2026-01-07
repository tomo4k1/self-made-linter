mod linter;
mod rules;
mod cli;
mod template_parser;

use clap::Parser;
use ignore::WalkBuilder;
use rayon::prelude::*;
use crate::linter::{Linter, LintResult, LinterConfig};
use crate::cli::Args;
use crate::rules::no_console::NoConsole;
use crate::rules::no_process_env::NoProcessEnv;
use crate::rules::no_v_html::NoVHtml;
use std::fs;

fn main() {
    let args = Args::parse();
    
    // Load config
    let config_path = ".linterrc.json";
    let config = if fs::metadata(config_path).is_ok() {
        let content = fs::read_to_string(config_path).expect("Failed to read .linterrc.json");
        serde_json::from_str(&content).expect("Failed to parse .linterrc.json")
    } else {
        LinterConfig::default()
    };

    let mut linter = Linter::new(config);
    
    // Register rules
    linter.add_rule(Box::new(NoConsole));
    linter.add_rule(Box::new(NoProcessEnv));
    linter.add_rule(Box::new(NoVHtml));
    
    // New Rules (Phase 5)
    use crate::rules::vue::require_v_for_key::RequireVForKey;
    use crate::rules::vue::mustache_interpolation_spacing::MustacheInterpolationSpacing;
    use crate::rules::nuxt::prefer_import_meta::PreferImportMeta;
    
    linter.add_rule(Box::new(RequireVForKey));
    linter.add_rule(Box::new(MustacheInterpolationSpacing));
    linter.add_rule(Box::new(PreferImportMeta));

    if !args.json {
        println!("🚀 Starting Speedy Nuxt Linter...");
        if args.fix {
            println!("🔧 Autofix enabled");
        }
    }

    // 1. Collect all target files recursively using ignore
    let mut files_to_lint = Vec::new();
    for input_path in &args.files {
        // WalkBuilder respects .gitignore by default
        let walker = WalkBuilder::new(input_path)
            .hidden(false) // default: ignore hidden files, but we might want them? let's stick to defaults usually
            .git_ignore(true)
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "vue" {
                                files_to_lint.push(path.to_path_buf());
                            }
                        }
                    }
                },
                Err(err) => {
                    if !args.json {
                        eprintln!("Error walking directory: {}", err);
                    }
                }
            }
        }
    }

    let file_count = files_to_lint.len();
    if !args.json {
        println!("📂 Analyzing {} files...", file_count);
    }

    // 2. Parallel Linting with Rayon
    let results: Vec<LintResult> = files_to_lint.par_iter()
        .filter_map(|path| linter.lint_file(path, args.fix))
        .collect();

    if args.json {
        let json_output = serde_json::to_string_pretty(&results).unwrap();
        println!("{}", json_output);
    } else {
        // Text output
        let mut total_issues = 0;
        let mut total_fixed = 0;

        for result in &results {
            if result.diagnostics.is_empty() && result.fixed_count == 0 {
                continue;
            }

            for d in &result.diagnostics {
                println!("❌ {} ({}:{}) - {} [{}]", 
                    result.path, 
                    d.start_line, 
                    d.start_column, 
                    d.message, 
                    if d.fix_available { "🔧" } else { "" }
                );
                total_issues += 1;
            }

            if result.fixed_count > 0 {
                println!("✨ Fixed {} issue(s) in {}", result.fixed_count, result.path);
                total_fixed += result.fixed_count;
            }
        }

        if total_issues == 0 && total_fixed == 0 {
             println!("✨ No issues found!");
        } else {
             println!("✨ Done! Found {} issues. Fixed {}.", total_issues, total_fixed);
        }
    }
}
