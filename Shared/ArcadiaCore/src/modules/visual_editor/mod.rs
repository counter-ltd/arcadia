use super::ModuleCommand;

pub mod blocks;
pub mod codegen;
pub mod palette;
pub mod parse;

pub const NAME: &str = "visual-editor";

pub fn commands() -> &'static [ModuleCommand] {
    &[]
}
