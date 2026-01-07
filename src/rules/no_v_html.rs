use crate::linter::{Context, Diagnostic, Rule};
use crate::template_parser::TemplateToken;
use oxc_span::Span;

pub struct NoVHtml;

impl Rule for NoVHtml {
    fn name(&self) -> &'static str {
        "vue/no-v-html"
    }

    fn run(&self, ctx: &Context) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if let Some(tokens) = ctx.template_tokens {
            for token in tokens {
                if let TemplateToken::StartTag { attributes, span, .. } = token {
                     if attributes.contains_key("v-html") {
                         // Span is already captured by SpannedEmitter relative to template content.
                         // Need to add template_start_offset.
                         let abs_start = span.start + ctx.source_file.template_start_offset as u32;
                         let abs_end = span.end + ctx.source_file.template_start_offset as u32;
                         
                         diagnostics.push(Diagnostic {
                            message: "Do not use `v-html` to prevent XSS.".to_string(),
                            span: Span::new(abs_start, abs_end), // Point to the tag
                            fix: None,
                        });
                     }
                }
            }
        }

        diagnostics
    }
}
