use std::collections::HashMap;

use super::func_info::ExportedType;

// TODO: temporary
#[derive(Debug)]
pub struct InterfaceInfo {
    pub name: String,
    pub full_path: String,
    pub file_name: String,
    exported_type: ExportedType,
    pub interface_id: u32,
    attributes: Vec<String>,
    // only allow function member with no visibility and member name needs &'static str
    pub members: HashMap<&'static str, super::func_info::Type>,
}

impl InterfaceInfo {
    pub fn new(
        name: String,
        path: String,
        id: u32,
        attributes: Vec<String>,
        exported_type: ExportedType,
    ) -> InterfaceInfo {
        InterfaceInfo {
            name: name.clone(),
            full_path: format!("{}#{}", path, name),
            file_name: path,
            exported_type,
            interface_id: id,
            attributes,
            members: HashMap::new(),
        }
    }

    pub fn add_member(&mut self, name: &'static str, return_type: super::func_info::Type) {
        self.members.insert(name, return_type);
    }
}
