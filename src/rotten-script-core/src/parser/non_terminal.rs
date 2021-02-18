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
    PrimaryExpression,
    ParenthesizedExpression,
    UnaryExpression,
    ExponentiationExpression,
    MultiplicativeExpression,
    AdditiveExpression,
    ShiftExpression,
    RelationalExpression,
    EqualityExpression,
    BitwiseAndExpression,
    BitwiseXorExpression,
    BitwiseOrexpression,
    LogicalAndExpression,
    LogicalOrExpression,
}
