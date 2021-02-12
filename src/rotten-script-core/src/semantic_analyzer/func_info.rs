#[derive(Debug)]
pub struct FuncInfo {
    pub name: String,
    pub full_path: String,
    pub file_name: String,
    exported_type: ExportedType,
    args: Arguments,
    return_type: Type,
    // uuid is a good choice for func_id?
    pub func_id: u32,
    pub is_entry: bool,
    attributes: Vec<String>,
}

impl FuncInfo {
    pub fn new(
        name: String,
        path: String,
        id: u32,
        attributes: Vec<String>,
        exported_type: ExportedType,
    ) -> FuncInfo {
        let is_entry = attributes.iter().any(|x| x == &String::from("EntryPoint"));
        FuncInfo {
            name: name.clone(),
            full_path: format!("{}#{}", path, name),
            file_name: path,
            exported_type,
            args: Arguments {
                arguments: Vec::new(),
            },
            return_type: Type::Primitive(PrimitiveType::Void),
            func_id: id,
            is_entry,
            attributes,
        }
    }
}

#[derive(Debug)]
pub enum ExportedType {
    Export,
    DefaultExport,
    None,
}

#[derive(Debug)]
struct Arguments {
    arguments: Vec<(String, Type)>,
}
#[derive(Debug)]
enum Type {
    Primitive(PrimitiveType),
    Object,
}
#[derive(Debug)]
enum PrimitiveType {
    Number,
    String,
    Boolean,
    Void,
}
