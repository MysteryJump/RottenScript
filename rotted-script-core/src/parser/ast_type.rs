use super::non_terminal::NonTerminal;
#[derive(Debug, PartialEq)]
pub enum AstType {
    Terminal,
    NonTerminal(NonTerminal),
}
