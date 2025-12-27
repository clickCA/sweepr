use crate::error::{PurgeError, Result};
use crate::graph::{ImportEdge, Symbol, SymbolReference};
use oxc_ast::ast::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::GetSpan;
use oxc_span::SourceType;
use rayon::prelude::*;
use std::path::PathBuf;

pub struct AstAnalyzer;

#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub imports: Vec<ImportEdge>,
    pub exports: Vec<Symbol>,
    pub references: Vec<SymbolReference>,
}

impl AstAnalyzer {
    /// Parse all files in parallel
    pub fn parse_files_parallel(files: Vec<PathBuf>) -> Result<Vec<ParsedFile>> {
        let results: Vec<Result<ParsedFile>> = files
            .into_par_iter()
            .map(|file| Self::parse_file(file))
            .collect();

        results.into_iter().collect()
    }

    /// Parse a single file
    pub fn parse_file(path: PathBuf) -> Result<ParsedFile> {
        let source = std::fs::read_to_string(&path)
            .map_err(|e| PurgeError::Io(e))?;

        let parser_result = Self::parse_source(&source, &path);

        match parser_result {
            Ok(parsed) => Ok(parsed),
            Err(e) => Err(PurgeError::ParseError {
                path: path.to_string_lossy().to_string(),
                message: e,
            }),
        }
    }

    fn parse_source(source: &str, path: &PathBuf) -> std::result::Result<ParsedFile, String> {
        // Parse the source code
        let source_type = SourceType::from_path(path).unwrap();
        let allocator = Allocator::default();
        let parser = Parser::new(&allocator, source, source_type);
        let result = parser.parse();

        if !result.errors.is_empty() {
            return Err(format!("Parse error: {:?}", result.errors[0]));
        }

        let program = result.program;

        let mut parsed = ParsedFile {
            path: path.clone(),
            imports: Vec::new(),
            exports: Vec::new(),
            references: Vec::new(),
        };

        // Walk the AST
        Self::visit_module(&program, path, &mut parsed);

        Ok(parsed)
    }

    fn visit_module(program: &Program, path: &PathBuf, parsed: &mut ParsedFile) {
        // Program body is directly accessible
        Self::visit_module_body(&program.body, path, parsed);
    }

    fn visit_module_body(body: &[Statement], path: &PathBuf, parsed: &mut ParsedFile) {
        for stmt in body {
            match stmt {
                Statement::ImportDeclaration(import_decl) => {
                    Self::handle_import_declaration(import_decl, path, parsed);
                }
                Statement::ExportNamedDeclaration(export_decl) => {
                    Self::handle_export_named_declaration(export_decl, path, parsed);
                }
                Statement::ExportDefaultDeclaration(export_decl) => {
                    Self::handle_export_default_declaration(export_decl, path, parsed);
                }
                Statement::ExportAllDeclaration(export_decl) => {
                    // Barrel export - skip for now
                    let _ = export_decl;
                }
                Statement::ExpressionStatement(expr_stmt) => {
                    Self::extract_references(&expr_stmt.expression, path, parsed);
                }
                Statement::BlockStatement(block) => {
                    Self::visit_block(block, path, parsed);
                }
                Statement::IfStatement(if_stmt) => {
                    Self::extract_references(&if_stmt.test, path, parsed);
                    Self::visit_statement(&if_stmt.consequent, path, parsed);
                    if let Some(alternate) = &if_stmt.alternate {
                        Self::visit_statement(alternate, path, parsed);
                    }
                }
                Statement::WhileStatement(while_stmt) => {
                    Self::extract_references(&while_stmt.test, path, parsed);
                    Self::visit_statement(&while_stmt.body, path, parsed);
                }
                Statement::ForStatement(for_stmt) => {
                    if let Some(init) = &for_stmt.init {
                        match init {
                            ForStatementInit::VariableDeclaration(var_decl) => {
                                Self::visit_for_init(var_decl, path, parsed);
                            }
                            _ if init.as_expression().is_some() => {
                                if let Some(expr) = init.as_expression() {
                                    Self::extract_references(expr, path, parsed);
                                }
                            }
                            _ => {}
                        }
                    }
                    if let Some(test) = &for_stmt.test {
                        Self::extract_references(test, path, parsed);
                    }
                    Self::visit_statement(&for_stmt.body, path, parsed);
                }
                Statement::FunctionDeclaration(func_decl) => {
                    // Function declarations are hoisted
                    if let Some(ident) = &func_decl.id {
                        parsed.exports.push(Symbol {
                            name: ident.name.to_string(),
                            file: path.clone(),
                            span: (ident.span.start as usize, ident.span.end as usize),
                        });
                    }
                }
                Statement::ClassDeclaration(class_decl) => {
                    if let Some(ident) = &class_decl.id {
                        parsed.exports.push(Symbol {
                            name: ident.name.to_string(),
                            file: path.clone(),
                            span: (ident.span.start as usize, ident.span.end as usize),
                        });
                    }
                }
                Statement::VariableDeclaration(var_decl) => {
                    Self::handle_variable_declaration(var_decl, path, parsed, true);
                }
                _ => {}
            }
        }
    }

    fn visit_block(block: &BlockStatement, path: &PathBuf, parsed: &mut ParsedFile) {
        Self::visit_module_body(&block.body, path, parsed);
    }

    fn visit_statement(stmt: &Statement, path: &PathBuf, parsed: &mut ParsedFile) {
        match stmt {
            Statement::BlockStatement(block) => Self::visit_block(block, path, parsed),
            Statement::IfStatement(if_stmt) => {
                Self::extract_references(&if_stmt.test, path, parsed);
                Self::visit_statement(&if_stmt.consequent, path, parsed);
                if let Some(alternate) = &if_stmt.alternate {
                    Self::visit_statement(alternate, path, parsed);
                }
            }
            _ => {}
        }
    }

    fn visit_for_init(init: &VariableDeclaration<'_>, path: &PathBuf, parsed: &mut ParsedFile) {
        // For now, just handle variable declarations in for loops
        Self::handle_variable_declaration(init, path, parsed, false);
    }

    fn handle_import_declaration(
        import_decl: &ImportDeclaration,
        path: &PathBuf,
        parsed: &mut ParsedFile,
    ) {
        let source = import_decl.source.value.as_str();

        // Check if it's a package import (starts with non-dot/slash)
        let is_package_import = !source.starts_with('.') && !source.starts_with('/');

        let mut imported_symbols = Vec::new();

        // Iterate over specifiers - convert to slice first
        if let Some(specifiers) = &import_decl.specifiers {
            let specifiers_slice: &[ImportDeclarationSpecifier] = specifiers;
            for specifier in specifiers_slice {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        imported_symbols.push(spec.imported.name().to_string());
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(_spec) => {
                        imported_symbols.push("default".to_string());
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(_spec) => {
                        imported_symbols.push("*".to_string());
                    }
                }
            }
        }

        // Don't track package imports in the file graph for now
        if !is_package_import {
            parsed.imports.push(ImportEdge {
                from: path.clone(),
                to: path.parent().unwrap().join(source).to_path_buf(),
                imported_symbols,
                is_type_only: import_decl.import_kind.is_type(),
            });
        }
    }

    fn handle_export_named_declaration(
        export_decl: &ExportNamedDeclaration,
        path: &PathBuf,
        parsed: &mut ParsedFile,
    ) {
        if let Some(declaration) = &export_decl.declaration {
            match declaration {
                Declaration::FunctionDeclaration(func_decl) => {
                    if let Some(ident) = &func_decl.id {
                        parsed.exports.push(Symbol {
                            name: ident.name.to_string(),
                            file: path.clone(),
                            span: (ident.span.start as usize, ident.span.end as usize),
                        });
                    }
                }
                Declaration::ClassDeclaration(class_decl) => {
                    if let Some(ident) = &class_decl.id {
                        parsed.exports.push(Symbol {
                            name: ident.name.to_string(),
                            file: path.clone(),
                            span: (ident.span.start as usize, ident.span.end as usize),
                        });
                    }
                }
                Declaration::VariableDeclaration(var_decl) => {
                    Self::handle_variable_declaration(var_decl, path, parsed, true);
                }
                _ => {}
            }
        }

        // Handle explicit export specifiers (e.g., export { foo, bar })
        for specifier in &export_decl.specifiers {
            parsed.exports.push(Symbol {
                name: specifier.exported.name().to_string(),
                file: path.clone(),
                span: (specifier.span.start as usize, specifier.span.end as usize),
            });
        }
    }

    fn handle_export_default_declaration(
        export_decl: &ExportDefaultDeclaration,
        path: &PathBuf,
        parsed: &mut ParsedFile,
    ) {
        match &export_decl.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(func_decl) => {
                if let Some(ident) = &func_decl.id {
                    parsed.exports.push(Symbol {
                        name: ident.name.to_string(),
                        file: path.clone(),
                        span: (ident.span.start as usize, ident.span.end as usize),
                    });
                }
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class_decl) => {
                if let Some(ident) = &class_decl.id {
                    parsed.exports.push(Symbol {
                        name: ident.name.to_string(),
                        file: path.clone(),
                        span: (ident.span.start as usize, ident.span.end as usize),
                    });
                }
            }
            _ => {}
        }

        // Default export is always named "default"
        parsed.exports.push(Symbol {
            name: "default".to_string(),
            file: path.clone(),
            span: (export_decl.span.start as usize, export_decl.span.end as usize),
        });
    }

    fn handle_variable_declaration(
        var_decl: &VariableDeclaration,
        path: &PathBuf,
        parsed: &mut ParsedFile,
        is_exported: bool,
    ) {
        for declarator in &var_decl.declarations {
            let ident = match declarator.id.get_binding_identifier() {
                Some(ident) => ident,
                None => continue,
            };

            if is_exported {
                parsed.exports.push(Symbol {
                    name: ident.name.to_string(),
                    file: path.clone(),
                    span: (ident.span.start as usize, ident.span.end as usize),
                });
            } else {
                // It's a declaration, not a reference
            }

            // Extract references from the initializer
            if let Some(init) = &declarator.init {
                Self::extract_references(init, path, parsed);
            }
        }
    }

    fn extract_references(expr: &Expression, path: &PathBuf, parsed: &mut ParsedFile) {
        match expr {
            Expression::Identifier(ident) => {
                parsed.references.push(SymbolReference {
                    symbol: ident.name.to_string(),
                    file: path.clone(),
                    span: (ident.span.start as usize, ident.span.end as usize),
                });
            }
            Expression::CallExpression(call_expr) => {
                Self::extract_references(&call_expr.callee, path, parsed);
                for arg in &call_expr.arguments {
                    Self::extract_references_from_argument(arg, path, parsed);
                }
            }
            _ if expr.as_member_expression().is_some() => {
                if let Some(member_expr) = expr.as_member_expression() {
                    Self::extract_references(member_expr.object(), path, parsed);
                    // Extract the property name if it's a static property
                    if let Some(prop_name) = member_expr.static_property_name() {
                        parsed.references.push(SymbolReference {
                            symbol: prop_name.to_string(),
                            file: path.clone(),
                            span: (member_expr.span().start as usize, member_expr.span().end as usize),
                        });
                    }
                }
            }
            Expression::BinaryExpression(bin_expr) => {
                Self::extract_references(&bin_expr.left, path, parsed);
                Self::extract_references(&bin_expr.right, path, parsed);
            }
            Expression::AssignmentExpression(assign_expr) => {
                Self::extract_references(&assign_expr.right, path, parsed);
            }
            Expression::ArrayExpression(arr_expr) => {
                for elem in &arr_expr.elements {
                    match elem {
                        ArrayExpressionElement::SpreadElement(spread) => {
                            Self::extract_references(&spread.argument, path, parsed);
                        }
                        _ if elem.as_expression().is_some() => {
                            if let Some(expr) = elem.as_expression() {
                                Self::extract_references(expr, path, parsed);
                            }
                        }
                        ArrayExpressionElement::Elision(_) => {}
                        _ => {}
                    }
                }
            }
            Expression::ObjectExpression(obj_expr) => {
                for prop in &obj_expr.properties {
                    match prop {
                        ObjectPropertyKind::SpreadProperty(spread) => {
                            Self::extract_references(&spread.argument, path, parsed);
                        }
                        ObjectPropertyKind::ObjectProperty(data_prop) => {
                            Self::extract_references(&data_prop.value, path, parsed);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn extract_references_from_argument(
        arg: &Argument,
        path: &PathBuf,
        parsed: &mut ParsedFile,
    ) {
        match arg {
            _ if arg.as_expression().is_some() => {
                if let Some(expr) = arg.as_expression() {
                    Self::extract_references(expr, path, parsed);
                }
            }
            Argument::SpreadElement(spread) => {
                Self::extract_references(&spread.argument, path, parsed);
            }
            _ => {}
        }
    }
}
