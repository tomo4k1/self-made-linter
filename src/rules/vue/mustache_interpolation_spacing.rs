use crate::linter::{Context, Diagnostic, Rule, Fix};
use crate::template_parser::TemplateToken;
use oxc_span::Span;
use regex::Regex;
use std::sync::OnceLock;

pub struct MustacheInterpolationSpacing;

impl Rule for MustacheInterpolationSpacing {
    fn name(&self) -> &'static str {
        "vue/mustache-interpolation-spacing"
    }

    fn run(&self, ctx: &Context) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        // Regex to find {{ content }} structure inside string
        // We match {{ followed by anything followed by }}
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r"\{\{(.*?)\}\}").unwrap());

        if let Some(tokens) = ctx.template_tokens {
            for token in tokens {
                if let TemplateToken::String { content, span } = token {
                     // Search for {{...}} in content
                     for cap in re.captures_iter(content) {
                         if let Some(full_match) = cap.get(0) {
                             let inner_text = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                             
                             // Check validation: should specifically be " value "
                             // Rule: " " at start, " " at end.
                             // But not "  value ". Just one space?
                             // Usually "always" configuration means at least one space, or exactly one.
                             // Let's enforce exactly one space if content is not empty.
                             
                             let trimmed = inner_text.trim();
                             if trimmed.is_empty() {
                                 continue; // {{ }} empty or {{   }}
                             }
                             
                             let expected = format!(" {} ", trimmed);
                             if inner_text != expected {
                                 // Diagnostic
                                 let match_range = full_match.range();
                                 let abs_start = span.start + match_range.start as u32 + ctx.source_file.template_start_offset as u32;
                                 let abs_end = span.start + match_range.end as u32 + ctx.source_file.template_start_offset as u32;

                                diagnostics.push(Diagnostic {
                                    message: "Mustache interpolation should have spacing.".to_string(),
                                    span: Span::new(abs_start, abs_end), // Precise span
                                    fix: Some(Fix {
                                        span: Span::new(abs_start, abs_end),
                                        replacement: format!("{{{{ {} }}}}", trimmed),
                                    }),
                                });
                             }
                         }
                     }
                }
            }
        }

        diagnostics
    }
}
