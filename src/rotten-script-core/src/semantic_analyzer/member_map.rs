use super::semantic_tree::FuncInfo;
use std::{collections::HashMap, fmt::Debug, ops::Index};
pub struct MemberMap {
    // key: func_id, value: FuncInfo
    members: HashMap<i32, FuncInfo>,
    // key: func_name(full), value: func_id
    func_path_to_func_id_map: HashMap<String, i32>,
    count: usize,
}

impl Index<&i32> for MemberMap {
    type Output = FuncInfo;

    fn index(&self, index: &i32) -> &Self::Output {
        &self.members[index]
    }
}

/// Index for func_full_path
impl Index<&String> for MemberMap {
    type Output = FuncInfo;

    fn index(&self, index: &String) -> &Self::Output {
        let func_id = &self.func_path_to_func_id_map[index];
        &self.members[func_id]
    }
}

impl Debug for MemberMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.members.values()).finish()
    }
}

impl Default for MemberMap {
    fn default() -> Self {
        MemberMap {
            members: HashMap::new(),
            func_path_to_func_id_map: HashMap::new(),
            count: 0,
        }
    }
}

impl MemberMap {
    pub fn new() -> MemberMap {
        MemberMap::default()
    }
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn insert(&mut self, func: FuncInfo) -> Result<(), ()> {
        let func_name = func.full_path.clone();

        self.func_path_to_func_id_map
            .insert(func_name, func.func_id);
        let result = self.members.insert(func.func_id, func);

        if result.is_some() {
            Err(())
        } else {
            self.count += 1;
            Ok(())
        }
    }
}
