use crate::linter::{Context, Diagnostic, Rule};
use crate::template_parser::TemplateToken;
use oxc_span::Span;

pub struct RequireVForKey;

impl Rule for RequireVForKey {
    fn name(&self) -> &'static str {
        "vue/require-v-for-key"
    }

    fn run(&self, ctx: &Context) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if let Some(tokens) = ctx.template_tokens {
            for token in tokens {
                if let TemplateToken::StartTag { attributes, span, .. } = token {
                     // Check if v-for exists
                     if attributes.contains_key("v-for") {
                         // Check if :key or v-bind:key exists
                         let has_key = attributes.contains_key(":key") || attributes.contains_key("v-bind:key");
                         
                         if !has_key {
                             // Report error
                             let abs_start = span.start + ctx.source_file.template_start_offset as u32;
                             let abs_end = span.end + ctx.source_file.template_start_offset as u32;
                             
                             diagnostics.push(Diagnostic {
                                message: "Elements in iteration expect to have 'v-bind:key' directives.".to_string(),
                                span: Span::new(abs_start, abs_end),
                                fix: None, // Too complex to autofix (need to choose key)
                            });
                         }
                     }
                }
            }
        }

        diagnostics
    }
}
