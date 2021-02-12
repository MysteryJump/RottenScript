use super::func_info::FuncInfo;
use std::{collections::HashMap, fmt::Debug, ops::Index, rc::Rc};
pub struct MemberMap {
    // key: func_id, value: FuncInfo
    members: HashMap<u32, Rc<FuncInfo>>,
    // key: func_name(full), value: func_id
    func_path_to_func_id_map: HashMap<String, u32>,
    // key: file_path, value: func_id
    func_ids_of_files: HashMap<String, Vec<u32>>,
    count: usize,
    entries: Vec<u32>,
}

impl Index<&u32> for MemberMap {
    type Output = FuncInfo;

    fn index(&self, index: &u32) -> &Self::Output {
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
            func_ids_of_files: HashMap::new(),
            count: 0,
            entries: Vec::new(),
        }
    }
}

impl<'a> MemberMap {
    pub fn new() -> MemberMap {
        MemberMap::default()
    }
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn insert(&mut self, func: Rc<FuncInfo>) -> Result<(), ()> {
        let func_name = func.full_path.clone();

        self.func_path_to_func_id_map
            .insert(func_name, func.func_id);

        if self.func_path_to_func_id_map.contains_key(&func.file_name) {
            self.func_ids_of_files
                .insert(func.file_name.clone(), vec![func.func_id]);
        } else {
            match self.func_ids_of_files.get_mut(&func.file_name) {
                Some(x) => x.push(func.func_id),
                None => {
                    self.func_ids_of_files
                        .insert(func.file_name.clone(), vec![func.func_id]);
                }
            }
        }

        if func.is_entry {
            self.entries.push(func.func_id);
        }

        let result = self.members.insert(func.func_id, func);
        if result.is_some() {
            Err(())
        } else {
            self.count += 1;
            Ok(())
        }
    }

    pub fn get_from_file_name(&self, file_name: &str) -> Vec<Rc<FuncInfo>> {
        let ids = &self.func_ids_of_files[file_name];
        let infos = ids
            .iter()
            .map(|x| self.members[x].clone())
            .collect::<Vec<_>>();
        infos
    }

    pub fn get_entrypoint_ids(&self) -> &[u32] {
        &self.entries
    }
}
