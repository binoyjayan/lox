use crate::expr::*;
use crate::error::*;
use crate::object::*;

// Test AST Printer Vistor implementation - not part of interpreter
#[derive(Default)]
pub struct AstPrinter { }

impl AstPrinter {
    #[allow(dead_code)]
    pub fn print(&self, expr: &Expr) -> Result<String, LoxErr> {
        expr.accept(self)
    }
    fn paranthesize(&self, name: String, exprs: &[&Box<Expr>]) -> Result<String, LoxErr> {
        let mut builder = format!("({}", name);

        for expr in exprs {
            builder = format!("{} {}", builder, expr.accept(self)?);
        }
        builder = format!("{})", builder);

        Ok(builder)
    }
}

impl ExprVisitor<String> for AstPrinter {
    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<String, LoxErr> {
        self.paranthesize(expr.operator.lexeme.to_string(), &[&expr.left, &expr.right])
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<String, LoxErr> {
        self.paranthesize("group".to_string(), &[&expr.expression])
    }

    fn visit_literal_expr(&self, expr: &LiteralExpr) -> Result<String, LoxErr> {
        Ok(match &expr.value {
            Some(value) => value.to_string(),
            None => "nil".to_string(),
        })
    }

    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<String, LoxErr> {
        self.paranthesize(expr.operator.lexeme.to_string(), &[&expr.right])
    }
}

// TODO:
// assign:  Ok(format!("{} = {}", name.lexeme, self.evaluate(*value)?).into())
// logical: Ok(format!("({} {} {})", self.evaluate(*left)?, operator.lexeme, self.evaluate(*right)?))

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::*;
    #[test]
    fn ast_test1() -> Result<(), LoxErr> {
        //-123 * (45.67)   -->>    (* (- 123) (group 45.67))
        let expression = Expr::Binary(
            BinaryExpr {
                left: Box::new(Expr::Unary({
                    UnaryExpr {
                        operator: Token {
                            ttype: TokenType::Minus,
                            lexeme: "-".to_string(),
                            literal: None,
                            line: 1,
                            col: 1,
                        },
                        right: Box::new(Expr::Literal(
                            LiteralExpr {
                                value: Some(Object::Number(123.0)),
                            }
                        ))
                    }
                })),
                operator: Token {
                    ttype: TokenType::Star,
                    lexeme: "*".to_string(),
                    literal: None, line: 1, col: 1
                },
                right: Box::new(Expr::Grouping({
                    GroupingExpr {
                        expression: Box::new(Expr::Literal(
                            LiteralExpr {
                                value: Some(Object::Number(45.67)),
                            }
                        ))
                    }
                }))
            }
        );
        let result = AstPrinter::default().print(&expression)?;
        assert_eq!(result, "(* (- 123) (group 45.67))");
        Ok(())
    }
}
