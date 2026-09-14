//! Experimental markup language

mod editor;
mod highlighter;
pub mod parser;
mod viewer;

pub use self::editor::EasyMarkEditor;
pub use self::highlighter::MemoizedHighlighter;
pub use self::viewer::easy_mark;
