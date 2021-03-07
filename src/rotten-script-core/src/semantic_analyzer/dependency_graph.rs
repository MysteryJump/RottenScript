use daggy::Dag;

pub struct DependencyGraph {
    graph: Vec<Node>,
    dag: Dag<u32, u32, u32>,
}

struct FullPath(String);

impl DependencyGraph {
    fn push(&mut self, value: FullPath) -> i32 {
        let id = self.graph.len();
        let node = Node(id as i32, value, Vec::new());
        self.graph.push(node);
        id as i32
    }
    fn get(&self, id: i32) -> &Node {
        &self.graph[id as usize]
    }
}

struct Node(i32, FullPath, Vec<i32>);
