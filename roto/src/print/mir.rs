use crate::{
    ast, mir,
    print::{IrPrinter, Printable, TypeDisplay},
    types::ResolvedName,
};
use core::fmt::Write;

impl Printable for mir::Mir {
    fn print(&self, printer: &IrPrinter) -> String {
        let strings: Vec<_> = self.functions.iter().map(|f| f.print(printer)).collect();
        strings.join("\n")
    }
}

impl Printable for mir::Function {
    fn print(&self, printer: &IrPrinter) -> String {
        let mut s = String::new();

        let printer = IrPrinter {
            scope: Some(self.variables.scope),
            types: printer.types,
            labels: printer.labels,
            rt: printer.rt,
        };

        let _ = write!(
            &mut s,
            "fn {}({}) {{",
            self.name,
            self.parameters
                .iter()
                .map(|a| a.print(&printer))
                .collect::<Vec<_>>()
                .join(", ")
        );

        s.push('\n');
        for var in &self.variables.variables {
            let var = var.print(&printer);
            let _ = writeln!(&mut s, "  {var}");
        }

        let blocks = self.blocks.iter();
        for b in blocks {
            s.push('\n');
            for line in b.print(&printer).lines() {
                let _ = writeln!(&mut s, "  {line}");
            }
        }

        s.push_str("}\n\n");
        s
    }
}

impl Printable for mir::Block {
    fn print(&self, printer: &IrPrinter) -> String {
        let mut s = String::new();
        writeln!(s, "{}:", &self.label.print(printer)).unwrap();
        for i in &self.body {
            writeln!(s, "  {}", i.print(printer)).unwrap();
        }
        s
    }
}

impl Printable for mir::Instruction {
    fn print(&self, printer: &IrPrinter) -> String {
        match self {
            Self::Jump(to) => format!("jump {}", to.print(printer)),
            Self::Assign(to, ty, value) => {
                let to = to.print(printer);
                let ty = ty.display(printer.types);
                let value = value.print(printer);
                format!("{to}: {ty} = {value}")
            }
            Self::Switch {
                examinee,
                branches,
                fallback,
            } => {
                format!(
                    "switch {} [{}]{}",
                    examinee.0.print(printer),
                    branches
                        .iter()
                        .map(|(i, b)| format!("{i} => {}", b.print(printer)))
                        .collect::<Vec<_>>()
                        .join(", "),
                    if let Some(fallback) = fallback {
                        format!(" else {}", fallback.print(printer))
                    } else {
                        String::new()
                    },
                )
            }
            Self::Branch {
                cond,
                accept,
                reject,
            } => {
                format!(
                    "brif {} then {} else {}",
                    cond.print(printer),
                    accept.print(printer),
                    reject.print(printer),
                )
            }

            Self::SetDiscriminant { to, ty, variant } => {
                format!(
                    "{}.$discriminant = {}.{}",
                    to.print(printer),
                    ty.display(printer.types),
                    variant.name
                )
            }
            Self::Return(var) => format!("return {}", var.print(printer)),
            Self::Drop(val, _ty) => format!("drop({})", val.print(printer)),
        }
    }
}

impl Printable for mir::Place {
    fn print(&self, printer: &IrPrinter) -> String {
        let mut var = self.var.0.print(printer);
        for projection in &self.proj {
            match projection {
                mir::Projection::VariantField(_, i) => write!(var, ".{i}"),
                mir::Projection::Field(ident) => write!(var, ".{ident}"),
            }
            .unwrap();
        }
        var
    }
}

impl Printable for mir::TypedVar {
    fn print(&self, printer: &IrPrinter) -> String {
        format!(
            "{}: {}",
            self.0.print(printer),
            self.1.display(printer.types)
        )
    }
}

impl Printable for mir::Value {
    fn print(&self, printer: &IrPrinter) -> String {
        match self {
            Self::Const(x, ty) => {
                let ty = ty.display(printer.types);
                let x = x.print(printer);
                format!("{ty}({x})")
            }
            Self::Constant(x, ty) => {
                let x = x.print(printer);
                let ty = ty.display(printer.types);
                format!("load_constant({x}: {ty})")
            }
            Self::Clone(place) => format!("clone({})", place.print(printer)),
            Self::Discriminant(var) => format!("discriminant({})", var.print(printer)),
            Self::Unary(op, var) => match op {
                mir::UnOp::Not => format!("not({})", var.print(printer)),
                mir::UnOp::Neg => format!("neg({})", var.print(printer)),
            },
            Self::Move(var) => format!("move({})", var.print(printer)),
            Self::BinOp { lhs, op, ty, rhs } => {
                let lhs = lhs.print(printer);
                let ty = ty.display(printer.types);
                let rhs = rhs.print(printer);
                format!("{lhs} {op}({ty}) {rhs}")
            }
            Self::Call { func, args } => {
                let func = func.print(printer);
                let args = args.iter().map(|a| a.print(printer));
                let args = args.collect::<Vec<_>>().join(", ");
                format!("{func}({args})")
            }
            Self::CallRuntime { func, args } => {
                let func = printer.rt.get_function(*func).name.print(printer);
                let args = args.iter().map(|a| a.print(printer));
                let args = args.collect::<Vec<_>>().join(", ");
                format!("<{func}>({args})")
            }
        }
    }
}

impl Printable for ast::BinOp {
    fn print(&self, _printer: &IrPrinter) -> String {
        self.to_string()
    }
}

impl Printable for ResolvedName {
    fn print(&self, printer: &IrPrinter) -> String {
        let f = self.scope.print(printer);
        if f.is_empty() {
            self.ident.to_string()
        } else {
            format!("{f}.{}", self.ident)
        }
    }
}

impl Printable for ast::Literal {
    fn print(&self, _printer: &IrPrinter) -> String {
        match self {
            Self::String(str) => str.into(),
            Self::Integer(i) => i.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Unit => "()".into(),
        }
    }
}
