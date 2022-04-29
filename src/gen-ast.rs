use std::env;
use std::fs;
/** Metaprogramming for generating expressions and statements
 *
 */
use std::io;
use std::io::Write;
use std::process;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: gen-ast <outdir>");
        process::exit(64);
    }

    let outdir = args.get(1).unwrap();

    // Productions for expressions
    define_ast(
        outdir,
        "Expr",
        &[
            "std::rc::Rc",
            "std::hash::Hash",
            "std::hash::Hasher",
            "crate::error::*",
            "crate::token::*",
            "crate::object::*",
        ],
        &[
            "Assign       : Token name, Rc<Expr> value",
            "Binary       : Rc<Expr> left, Token operator, Rc<Expr> right",
            "Call         : Rc<Expr> callee, Token paren, Vec<Rc<Expr>> arguments",
            "Grouping     : Rc<Expr> expression",
            "Literal      : Option<Object> value",
            "Logical      : Rc<Expr> left, Token operator, Rc<Expr> right",
            "Unary        : Token operator, Rc<Expr> right",
            "Variable     : Token name",
        ],
    )?;

    // Productions for statements
    define_ast(
        outdir,
        "Stmt",
        &[
            "std::rc::Rc",
            "std::hash::Hash",
            "std::hash::Hasher",
            "crate::error::*",
            "crate::expr::Expr",
            "crate::token::Token",
        ],
        &[
            "Block        : Rc<Vec<Rc<Stmt>>> statements",
            "Expression   : Rc<Expr> expression",
            "Function     : Token name, Rc<Vec<Token>> params, Rc<Vec<Rc<Stmt>>> body",
            "If           : Rc<Expr> condition, Rc<Stmt> then_branch, Option<Rc<Stmt>> else_branch",
            "Print        : Rc<Expr> expression",
            "Return       : Token keyword, Option<Rc<Expr>> value",
            "Var          : Token name, Option<Rc<Expr>> initializer",
            "While        : Rc<Expr> condition, Rc<Stmt> body",
            "Break        : Token token",
        ],
    )?;

    Ok(())
}

#[derive(Debug)]
struct AstType {
    base_class_name: String,
    class_name: String,
    fields: Vec<String>,
}

fn define_ast(
    outdir: &str,
    base_name: &str,
    imports: &[&str],
    productions: &[&str],
) -> io::Result<()> {
    let mut types = Vec::new();

    for production in productions {
        let (base_class_name, tokens_str) = production.split_once(':').unwrap();
        let class_name = format!("{}{}", base_class_name.trim(), base_name.trim());
        let tokens_iter = tokens_str.split(',');
        let mut fields: Vec<String> = Vec::new();
        for token in tokens_iter {
            let (token_type, token_name) = token.trim().split_once(' ').unwrap();
            fields.push(format!("{}: {}", token_name, token_type));
        }
        types.push(AstType {
            base_class_name: base_class_name.trim().to_string(),
            class_name,
            fields,
        });
    }

    let path = format!("{outdir}/{}.rs", base_name.to_lowercase());
    let mut file = fs::File::create(path)?;

    writeln!(
        file,
        "// This is an autogenerated file. Do not edit manually. Use gen-ast package."
    )?;
    writeln!(file, "// Use gen-ast package to generate this file.\n")?;

    for i in imports {
        writeln!(file, "use {};", i)?;
    }

    // Define enum of expression types
    writeln!(file, "\npub enum {base_name} {{")?;
    for ty in &types {
        writeln!(file, "    {}(Rc<{}>),", ty.base_class_name, ty.class_name)?;
    }
    writeln!(file, "}}\n")?;

    // Implement PartialEq for ASTs
    writeln!(file, "impl PartialEq for {} {{", base_name)?;
    writeln!(file, "    fn eq(&self, other: &Self) -> bool {{")?;
    writeln!(file, "        match (self, other) {{")?;
    for t in &types {
        writeln!(
            file,
            "            ({0}::{1}(a), {0}::{1}(b)) => Rc::ptr_eq(a, b),",
            base_name, t.base_class_name
        )?;
    }
    writeln!(file, "            _ => false,")?;
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}\n")?;

    writeln!(file, "impl Eq for {} {{}}\n", base_name)?;

    // Implement hash for ASTs
    writeln!(file, "impl Hash for {} {{", base_name)?;
    writeln!(file, "    fn hash<H: Hasher>(&self, hasher: &mut H) {{")?;
    writeln!(file, "        match self {{ ")?;
    for t in &types {
        writeln!(
            file,
            "            {}::{}(a) => {{ hasher.write_usize(Rc::as_ptr(a) as usize); }}",
            base_name, t.base_class_name
        )?;
    }
    writeln!(file, "        }}\n    }}\n}}\n")?;

    // Implement enum expression
    writeln!(file, "impl {base_name} {{")?;
    writeln!(
        file,
        "    pub fn accept<T>(&self, base: Rc<{}>, visitor: &dyn {}Visitor<T>) -> Result<T, LoxResult> {{",
        base_name, base_name
    )?;
    writeln!(file, "        match self {{")?;
    for ty in &types {
        writeln!(
            file,
            "            {}::{}(v) => visitor.visit_{}_{}(base, &v),",
            base_name,
            ty.base_class_name,
            ty.base_class_name.to_lowercase(),
            base_name.to_lowercase(),
        )?;
    }
    writeln!(file, "        }}")?;
    writeln!(file, "    }}\n")?;
    writeln!(file, "}}\n")?;

    // Define concrete expressions
    for ty in &types {
        writeln!(file, "pub struct {} {{", ty.class_name)?;
        for field in &ty.fields {
            writeln!(file, "    pub {},", field)?;
        }
        writeln!(file, "}}\n")?;
    }

    // Define visitors traits for expressions
    writeln!(file, "pub trait {}Visitor<T> {{", base_name)?;
    for ty in &types {
        writeln!(
            file,
            "    fn visit_{0}_{1}(&self, base: Rc<{2}>, {1}: &{3}) -> Result<T, LoxResult>;",
            ty.base_class_name.to_lowercase(),
            base_name.to_lowercase(),
            base_name,
            ty.class_name,
        )?;
    }
    writeln!(file, "}}\n")?;
    Ok(())
}
