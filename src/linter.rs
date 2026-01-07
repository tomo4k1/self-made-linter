use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use std::path::{Path, PathBuf};
use std::fs; // fix 1: import fs
use html5gum::Tokenizer;
use crate::template_parser::{SpannedEmitter, TemplateToken};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// --- Data Structures ---

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LinterConfig {
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RuleConfig {
    State(String), // "off", "warn", "error"
    // Future: Options(Vec<...>) or Struct
}

impl Default for RuleConfig {
    fn default() -> Self {
        RuleConfig::State("error".to_string())
    }
}

impl RuleConfig {
    pub fn is_enabled(&self) -> bool {
        match self {
            RuleConfig::State(s) => s != "off",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub original_content: String,
    #[allow(dead_code)]
    pub script_content: String,
    pub script_start_offset: usize, // Start position of script in original file
    pub template_content: String,       // New
    pub template_start_offset: usize,   // New
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub span: Span,         // Span relative to script_content OR template_content
    pub fix: Option<Fix>,
}

#[derive(Debug, Clone)]
pub struct Fix {
    pub span: Span,         // Span
    pub replacement: String,
}


pub struct Context<'a> {
    pub source_file: &'a SourceFile,
    pub program: &'a oxc_ast::ast::Program<'a>,
    pub template_tokens: Option<&'a Vec<TemplateToken>>, // Updated
}

// Output structure
#[derive(Debug, Serialize)]
pub struct LintResult {
    pub path: String,
    pub diagnostics: Vec<DiagnosticWithLocation>,
    pub fixed_count: usize,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticWithLocation {
    pub message: String,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub fix_available: bool,
}

pub trait Rule: Send + Sync { // fix 2: Add Send + Sync
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    fn run(&self, ctx: &Context) -> Vec<Diagnostic>;
}

// --- Linter Engine ---

pub struct Linter {
    rules: Vec<Box<dyn Rule>>,
    config: LinterConfig,
}

impl Linter {
    pub fn new(config: LinterConfig) -> Self {
        Self { rules: Vec::new(), config }
    }

    pub fn add_rule(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    pub fn lint_file(&self, path: &Path, fix: bool) -> Option<LintResult> {
        let allocator = Allocator::default();
        
        // Read file
        let original_content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read {}: {}", path.display(), e);
                return None;
            }
        };
            
        // 1. SFC Parsing (Extract Script & Template)
        // We use html5gum to scan tokens and find script/template blocks.
        let mut script_content = String::new();
        let mut script_start_offset = 0;
        let mut template_content = String::new();
        let mut template_start_offset = 0;

        // Script extraction
        if let Some(start_tag_idx) = original_content.find("<script") {
             if let Some(script_content_start) = original_content[start_tag_idx..].find('>') {
                 let actual_start = start_tag_idx + script_content_start + 1;
                 // Find closing tag strictly after start
                 if let Some(end_tag_relative) = original_content[actual_start..].find("</script>") {
                     script_start_offset = actual_start;
                     script_content = original_content[actual_start .. actual_start + end_tag_relative].to_string();
                 }
             }
        } else {
             // Fallback: entire file is script (if no template/style)? No, SFC requires <script>.
             // If no script tag, try to parse as script? No, assume empty script.
        }

        // Template extraction (simple)
        if let Some(start_tag_idx) = original_content.find("<template") {
            if let Some(content_start) = original_content[start_tag_idx..].find('>') {
                 let actual_start = start_tag_idx + content_start + 1;
                 if let Some(end_tag_relative) = original_content[actual_start..].find("</template>") {
                    template_start_offset = actual_start;
                    template_content = original_content[actual_start .. actual_start + end_tag_relative].to_string();
                 }
            }
        }

        let source_file = SourceFile {
            path: path.to_path_buf(),
            original_content: original_content.clone(),
            script_content: script_content.clone(),
            script_start_offset,
            template_content,
            template_start_offset,
        };

        // 2. Parse Script
        let source_type = SourceType::from_path(path).unwrap_or_default().with_typescript(true);
        let ret = Parser::new(&allocator, &script_content, source_type).parse();

        // 3. Parse Template (Tokenize)
        let template_tokens = if !source_file.template_content.is_empty() {
             let (emitter, _) = SpannedEmitter::new(&source_file.template_content);
             let tokenizer = Tokenizer::new_with_emitter(&source_file.template_content, emitter);
             // We need to collect tokens AND update offsets.
             // Our SpannedEmitter produces TemplateToken with spans relative to *template_content*.
             // The Linter (or Rule) adds `template_start_offset` later.
             // Wait, SpannedEmitter is used inside `Tokenizer`.
             // Collect tokens from the iterator (this drains the emitter via pop_token)
             let tokens: Vec<TemplateToken> = tokenizer.filter_map(|res| res.ok()).collect();
             Some(tokens)
        } else {
            None
        };

        let ctx = Context {
            source_file: &source_file,
            program: &ret.program,
            template_tokens: template_tokens.as_ref(), // Pass reference
        };

        // 4. Run Rules
        let mut diagnostics = Vec::new();
        for rule in &self.rules {
            let rule_name = rule.name();
            // Check config
            let is_enabled = if let Some(conf) = self.config.rules.get(rule_name) {
                conf.is_enabled()
            } else {
                true // Enabled by default if not in config
            };

            if is_enabled {
                diagnostics.extend(rule.run(&ctx));
            }
        }

        // 5. Apply Fixes (if enabled)
        let mut fixed_count = 0;
        if fix {
            fixed_count = self.apply_fixes(&source_file, &diagnostics);
        }

        // 6. Enrich Diagnostics
        let enriched_diagnostics = diagnostics.into_iter().map(|d| {
            let abs_start = d.span.start as usize; // Rules now return absolute spans
            let abs_end = d.span.end as usize;
            let (start_line, start_column) = get_line_col(&original_content, abs_start);
            let (end_line, end_column) = get_line_col(&original_content, abs_end);
            
            DiagnosticWithLocation {
                message: d.message,
                start_line,
                start_column,
                end_line,
                end_column,
                fix_available: d.fix.is_some(),
            }
        }).collect();

        Some(LintResult {
            path: path.to_string_lossy().to_string(),
            diagnostics: enriched_diagnostics,
            fixed_count,
        })
    }

    fn apply_fixes(&self, source_file: &SourceFile, diagnostics: &[Diagnostic]) -> usize {
        let mut fixes: Vec<&Fix> = diagnostics.iter().filter_map(|d| d.fix.as_ref()).collect();
        
        if fixes.is_empty() {
            return 0;
        }

        // Sort fixes by start position descending
        fixes.sort_by(|a, b| b.span.start.cmp(&a.span.start));

        let fix_count = fixes.len();
        let mut new_content = source_file.original_content.clone();

        for fix in fixes {
            let abs_start = fix.span.start as usize;
            let abs_end = fix.span.end as usize;
            
            // Check boundaries
            if abs_start > new_content.len() || abs_end > new_content.len() {
                continue;
            }

            new_content.replace_range(abs_start..abs_end, &fix.replacement);
        }

        if let Err(e) = std::fs::write(&source_file.path, new_content) {
            eprintln!("Failed to write fix: {}", e);
            0
        } else {
            fix_count
        }
    }
}

fn get_line_col(content: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, c) in content.char_indices() {
        if i == offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

