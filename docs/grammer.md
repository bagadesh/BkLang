
# Grammer
- `Term` > `Ident`| `IntLiteral` |  `Expr`
- `Expr` > `Term` | `BinaryExpr` | `Call`
- `Call` > `FnCall`
- `BinaryExpr`
	- `lhs` > `Expr`
	- `rhs` > `Expr`
	- `operator` > `BinaryOp`
- `BinaryOp` > `Add` | `Multiple` | `Subtract` | `Division`
