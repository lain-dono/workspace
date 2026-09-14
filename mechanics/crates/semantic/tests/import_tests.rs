use crate::utils::SemanticTest;
use semantic::{ImportPath, names::ImportName};

mod utils;

#[test]
fn import_ast_transform() {
    let import_name_ast = ImportName::new("import1");
    let imports = [import_name_ast.clone()];
    assert_eq!(imports.len(), 1);
    assert_eq!(import_name_ast.to_string(), "import1");
    let x: ImportPath = vec![import_name_ast];
    assert_eq!(x.len(), 1);
    let t = SemanticTest::new();
    t.state.import(&x);
}
