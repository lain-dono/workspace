use crate::{
    lir,
    print::{IrPrinter, Printable, TypeDisplay},
};
use core::fmt::Write;

impl Printable for lir::Lir {
    fn print(&self, printer: &IrPrinter) -> String {
        let strings: Vec<_> = self.functions.iter().map(|f| f.print(printer)).collect();
        strings.join("\n")
    }
}

impl Printable for lir::Function {
    fn print(&self, printer: &IrPrinter) -> String {
        let mut s = String::new();

        let printer = IrPrinter {
            scope: Some(self.scope),
            types: printer.types,
            labels: printer.labels,
            rt: printer.rt,
        };

        let _ = write!(
            &mut s,
            "fn {}({}) {}{{",
            self.name,
            self.ir_signature
                .parameters
                .iter()
                .map(|(a, t)| {
                    let a = a.print(&printer);
                    let t = t.display(printer.types);
                    format!("{a}: {t}")
                })
                .collect::<Vec<_>>()
                .join(", "),
            self.ir_signature
                .return_type
                .map(|t| {
                    let t = t.display(printer.types);
                    format!("-> {t} ")
                })
                .unwrap_or_default(),
        );

        s.push('\n');
        for (var, ty) in &self.variables {
            let var = var.print(&printer);
            let ty = match ty {
                lir::ValueOrSlot::Value(ir_type) => ir_type.to_string(),
                lir::ValueOrSlot::Slot(layout) => {
                    format!(
                        "ptr = Slot(size={}, align={})",
                        layout.size(),
                        layout.align()
                    )
                }
            };
            let _ = writeln!(&mut s, "  {var}: {ty}");
        }

        for b in &self.blocks {
            writeln!(s).unwrap();
            for line in b.print(&printer).lines() {
                let _ = writeln!(&mut s, "  {line}");
            }
        }

        s.push_str("}\n\n");

        s
    }
}

impl Printable for lir::Block {
    fn print(&self, printer: &IrPrinter) -> String {
        let mut s = String::new();
        writeln!(s, "{}:", &self.label.print(printer)).unwrap();
        for i in &self.body {
            writeln!(s, "  {}", i.print(printer)).unwrap();
        }
        s
    }
}

impl Printable for lir::Instruction {
    fn print(&self, printer: &IrPrinter) -> String {
        match self {
            Self::Assign { to, from: val } => {
                format!("{}: {} = {}", to.0.print(printer), to.1, val.print(printer))
            }
            Self::ConstAddr { to, name } => {
                format!(
                    "{}: {} = ConstantAddress(\"{}\")",
                    to.0.print(printer),
                    to.1,
                    name.print(printer),
                )
            }
            Self::Call {
                to,
                func,
                args,
                out_ptr,
            } => {
                let to = match to {
                    Some(lir::TypedVar(to, ty)) => {
                        format_args!("{}: {} = ", to.print(printer), *ty)
                    }
                    None => format_args!(""),
                };
                let out = out_ptr.clone().map(lir::Operand::from);
                let out = out.as_ref().into_iter();
                let args = out.chain(args.iter()).map(|a| a.print(printer));
                let args = args.collect::<Vec<_>>().join(", ");
                format!("{to}{}({args})", func.print(printer))
            }
            Self::CallRuntime { func, args } => {
                let func = printer.rt.get_function(*func).name.print(printer);
                let args = args.iter().map(|a| a.print(printer));
                format!("<{}>({})", func, args.collect::<Vec<_>>().join(", "))
            }
            Self::InitString { to, string } => {
                format!("{}: str = InitString(\"{string}\")", to.0.print(printer))
            }
            Self::Return(None) => "return".to_string(),
            Self::Return(Some(v)) => format!("return {}", v.print(printer)),
            Self::Cmp { to, cmp, lhs, rhs } => {
                format!(
                    "{}: {} = {cmp}({}, {})",
                    to.0.print(printer),
                    to.1,
                    lhs.print(printer),
                    rhs.print(printer),
                )
            }
            Self::Unary { op, to, val } => {
                let (target, ty, val) = (to.0.print(printer), to.1, val.print(printer));
                match op {
                    lir::UnOp::Eqz => format!("{target}: {ty} = ieqz({val})"),
                    lir::UnOp::Clz => format!("{target}: {ty} = iclz({val})"),
                    lir::UnOp::Ctz => format!("{target}: {ty} = ictz({val})"),
                    lir::UnOp::Pop => format!("{target}: {ty} = ones({val})"),
                    lir::UnOp::BNot => format!("{target}: {ty} = bnot({val})"),
                    lir::UnOp::INeg => format!("{target}: {ty} = ineg({val})"),
                    lir::UnOp::FNeg => format!("{target}: {ty} = fneg({val})"),
                }
            }
            Self::Op { op, to, lhs, rhs } => {
                format!(
                    "{}: {} = {op:?}({}, {})",
                    to.0.print(printer),
                    to.1,
                    lhs.print(printer),
                    rhs.print(printer),
                )
            }
            Self::Jump(to) => format!("jump {}", to.print(printer)),
            Self::Switch {
                examinee,
                fallback,
                branches,
            } => {
                format!(
                    "switch {} [{}] else {}",
                    examinee.print(printer),
                    branches
                        .iter()
                        .map(|(i, b)| format!("{i} => {}", b.print(printer)))
                        .collect::<Vec<_>>()
                        .join(", "),
                    fallback.print(printer),
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
            Self::Initialize { to, bytes, layout } => {
                format!(
                    "{}: {} = mem::initialize([{}], size={}, align={})",
                    to.0.print(printer),
                    to.1,
                    bytes
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                    layout.size(),
                    layout.align(),
                )
            }
            Self::Offset { to, from, offset } => {
                format!(
                    "{}: {} = ptr::offset({}, {offset})",
                    to.0.print(printer),
                    to.1,
                    from.print(printer),
                )
            }
            Self::Read { to, from } => {
                format!(
                    "{}: {} = mem::read({})",
                    to.0.print(printer),
                    to.1,
                    from.print(printer),
                )
            }
            Self::Write { to, val } => {
                format!("mem::write({}, {})", to.print(printer), val.print(printer),)
            }
            Self::Copy { to, from, size } => {
                format!(
                    "mem::copy({}, {}, {size})",
                    to.print(printer),
                    from.print(printer),
                )
            }
            Self::Clone { to, from, clone } => {
                format!(
                    "mem::clone({}, {}, with={clone:?})",
                    to.print(printer),
                    from.print(printer),
                )
            }
            Self::Drop { var, drop } => {
                format!("mem::drop({}, {drop:?})", var.print(printer))
            }
        }
    }
}

impl Printable for lir::Operand {
    fn print(&self, printer: &IrPrinter) -> String {
        match self {
            Self::Place(x) => x.print(printer),
            Self::Value(x) => x.to_string(),
        }
    }
}
