use roto::ast::{Expr, Meta, ParseError, Parser, Spans};

type Result<T> = core::result::Result<T, ParseError>;

fn parse_expr(s: &str) -> Result<Meta<Expr>> {
    let mut spans = Spans::default();
    Parser::run_parser(Parser::expr, 0, &mut spans, s)
}

#[test]
fn assign_expr() {
    parse_expr("a = 4").unwrap();
    parse_expr("a + 3 = 4").unwrap_err();
    parse_expr("a = 4 + 3").unwrap();
}

#[test]
fn test_logical_expr() {
    let s = "( blaffer.waf().contains(my_set) ) || ( blaffer.blaf() < bop() )";
    parse_expr(s).unwrap();
    let s = r#"blaffer.blaf.contains(something,"somewhat") > blaf()"#;
    parse_expr(s).unwrap();
    let s = r"( my_set.contains(bla.bla()) ) || ( my_other_set.contains(bla.bla()) )";
    parse_expr(s).unwrap();
    let s = "(found_prefix.prefix.exists() && found_prefix.prefix.exists()) || route_in_table";
    parse_expr(s).unwrap();
}

//------------ Compute Expressions parsing ----------------------------------

#[test]
fn test_compute_expr() {
    let s = r#"source_asns.contains("asn", route.as_path.origin)"#;
    parse_expr(s).unwrap();
    let s = "a.b.c.d(x,y,z).e.f(o.p()).g";
    parse_expr(s).unwrap();
    let s = "send-to(a, b)";
    parse_expr(s).unwrap();
    let s = "global_record.field";
    parse_expr(s).unwrap();
    let s = "pph_asn.asn.set(AS200)";
    parse_expr(s).unwrap();
}

//------------ Other Expressions --------------------------------------------

#[test]
fn test_value_expr() {
    let s = "globlaf(bla)";
    parse_expr(s).unwrap();
}

#[test]
fn test_match() {
    let s = "
        match x {
            A(x) -> b(),
            C(y) -> d(),
        }
    ";
    parse_expr(s).unwrap();
}

#[test]
fn test_match_block() {
    let s = "
        match x {
            A(x) -> {
                a == b;
                a && b;
            }
            C(y) -> d(),
        }
    ";
    parse_expr(s).unwrap();
}

#[test]
fn test_and_and_and() {
    let s = "a && b && c && d";
    parse_expr(s).unwrap();
}

#[test]
fn test_or_or_or() {
    let s = "a || b || c || d";
    parse_expr(s).unwrap();
}

#[test]
fn test_and_or_and() {
    let s = "a && b || c && d";
    parse_expr(s).unwrap_err();
}

#[test]
fn test_if() {
    let s = "if true { 0 }";
    parse_expr(s).unwrap();
}

#[test]
fn test_if_else() {
    let s = "if true { 0 } else { 1 }";
    parse_expr(s).unwrap();
}

#[test]
fn test_if_else_if_else() {
    let s = "if true { 0 } else if false { 1 } else { 2 }";
    parse_expr(s).unwrap();
}

#[test]
fn test_not_true() {
    let s = "not true";
    parse_expr(s).unwrap();
}

#[test]
fn test_not_true_is_true() {
    let s = "not true == true";
    parse_expr(s).unwrap();
}

#[test]
fn hex_number() {
    let s = "0xffff029";
    parse_expr(s).unwrap();
}

#[test]
fn parse_match() {
    let s = "
      match s {
          Foo -> 10,
          Bar -> 20
      }
    ";
    parse_expr(s).unwrap();

    let s = "
      match s {
          Foo -> 10,
          Bar -> 20,
      }
    ";
    parse_expr(s).unwrap();

    let s = "
      match s {
          Foo -> { 10 },
          Bar -> { 20 },
      }
    ";
    parse_expr(s).unwrap();

    let s = "
      match s {
          Foo -> { 10 }
          Bar -> { 20 }
      }
    ";
    parse_expr(s).unwrap();
}
