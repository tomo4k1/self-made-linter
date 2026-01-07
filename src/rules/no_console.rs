use oxc_ast::ast::*;
use oxc_span::Span;
use crate::linter::{Rule, Context, Diagnostic, Fix};

pub struct NoConsole;

impl Rule for NoConsole {
    fn name(&self) -> &'static str {
        "no-console"
    }

    fn run(&self, ctx: &Context) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for stmt in &ctx.program.body {
            if let Statement::ExpressionStatement(expr_stmt) = stmt {
                if let Expression::CallExpression(call_expr) = &expr_stmt.expression {
                     if let Expression::StaticMemberExpression(member) = &call_expr.callee {
                        if let Expression::Identifier(obj) = &member.object {
                            if obj.name == "console" {
                                // "log", "warn", "error" etc.
                                // For MVP, we catch everything under console.*
                                
                                diagnostics.push(Diagnostic {
                                    message: format!("Unexpected console statement: console.{}", member.property.name),
                                    span: Span::new(expr_stmt.span.start + ctx.source_file.script_start_offset as u32, expr_stmt.span.end + ctx.source_file.script_start_offset as u32),
                                    fix: Some(Fix {
                                        span: Span::new(expr_stmt.span.start + ctx.source_file.script_start_offset as u32, expr_stmt.span.end + ctx.source_file.script_start_offset as u32),
                                        replacement: format!("/* console.{} */", member.property.name), // Comemnt out as fix
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
