use oxc_ast::ast::*;
use oxc_span::Span;
use crate::linter::{Rule, Context, Diagnostic, Fix};

pub struct NoProcessEnv;

impl Rule for NoProcessEnv {
    fn name(&self) -> &'static str {
        "no-process-env"
    }

    fn run(&self, ctx: &Context) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        // Recursive traversal helper
        // For MVP, we'll iterate top-level statements and support some common patterns.
        // In a real linter, we would use a Visitor pattern or oxc_semantic.
        for stmt in &ctx.program.body {
            check_statement(stmt, &mut diagnostics, ctx);
        }
        
        diagnostics
    }
}

fn check_statement(stmt: &Statement, diagnostics: &mut Vec<Diagnostic>, ctx: &Context) {
    match stmt {
        Statement::ExpressionStatement(expr) => check_expression(&expr.expression, diagnostics, ctx),
        Statement::VariableDeclaration(decl) => {
            for declarator in &decl.declarations {
                if let Some(init) = &declarator.init {
                    check_expression(init, diagnostics, ctx);
                }
            }
        },
        _ => {}
    }
}

fn check_expression(expr: &Expression, diagnostics: &mut Vec<Diagnostic>, ctx: &Context) {
    match expr {
        Expression::StaticMemberExpression(member) => {
            // Check for process.env
            if let Expression::Identifier(obj) = &member.object {
                if obj.name == "process" && member.property.name == "env" {
                    let span = member.span;
                     diagnostics.push(Diagnostic {
                        message: "Use `import.meta.env` instead of `process.env`.".to_string(),
                        span: Span::new(span.start + ctx.source_file.script_start_offset as u32, span.end + ctx.source_file.script_start_offset as u32),
                        fix: Some(Fix {
                            span: Span::new(span.start + ctx.source_file.script_start_offset as u32, span.end + ctx.source_file.script_start_offset as u32),
                            replacement: "import.meta.env".to_string(),
                        }),
                    });
                }
            }
            // Check nested (e.g. process.env.FOO)
            // If the object is another member expression, recurse.
            check_expression(&member.object, diagnostics, ctx);
        },
        Expression::CallExpression(call) => {
            check_expression(&call.callee, diagnostics, ctx);
            for arg in &call.arguments {
                if let Some(e) = arg.as_expression() {
                    check_expression(e, diagnostics, ctx);
                }
            }
        },
        _ => {}
    }
}
