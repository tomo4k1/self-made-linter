use crate::linter::{Context, Diagnostic, Rule, Fix};
use oxc_ast::ast::{Expression, Statement};
use oxc_span::Span;

pub struct PreferImportMeta;

impl Rule for PreferImportMeta {
    fn name(&self) -> &'static str {
        "nuxt/prefer-import-meta"
    }


    fn run(&self, ctx: &Context) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Traverse AST manually for MVP
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
        Statement::BlockStatement(block) => {
            for s in &block.body {
                check_statement(s, diagnostics, ctx);
            }
        },
        Statement::IfStatement(if_stmt) => {
            check_expression(&if_stmt.test, diagnostics, ctx);
            check_statement(&if_stmt.consequent, diagnostics, ctx);
            if let Some(alt) = &if_stmt.alternate {
                check_statement(alt, diagnostics, ctx);
            }
        },
        _ => {}
    }
}

fn check_expression(expr: &Expression, diagnostics: &mut Vec<Diagnostic>, ctx: &Context) {
    match expr {
        Expression::StaticMemberExpression(member) => {
            // Check for process.client / process.server
            if let Expression::Identifier(ident) = &member.object {
                if ident.name == "process" {
                    let prop_name = &member.property.name;
                    if prop_name == "client" || prop_name == "server" {
                        let span = member.span;
                        let replacement = format!("import.meta.{}", prop_name);
                        
                        diagnostics.push(Diagnostic {
                            message: format!("Use `import.meta.{}` instead of `process.{}`.", prop_name, prop_name),
                            span: Span::new(span.start + ctx.source_file.script_start_offset as u32, span.end + ctx.source_file.script_start_offset as u32),
                            fix: Some(Fix {
                                span: Span::new(span.start + ctx.source_file.script_start_offset as u32, span.end + ctx.source_file.script_start_offset as u32),
                                replacement,
                            }),
                        });
                    }
                }
            }
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
