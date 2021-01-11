#[derive(Debug, PartialEq)]
pub enum NonTerminal {
    TranslationUnit,
    Attribute,
    ExportableConstDeclaration,
    ConstDeclaration,
    LetDeclaration,
    DeclarationBody,
    Expression,
    CallExpression,
    FunctionExpression,
    CompoundExpression,
    Args,
    ExpressionStatement,
    NamedImportDeclaration,
    DefaultImportDeclaration,
    ImportDeclaration,
}
