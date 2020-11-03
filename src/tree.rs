// https://stackoverflow.com/questions/29296038/implementing-a-mutable-tree-structure

#[derive(Debug)]
struct Node {
    children: Vec<Node>,
    data: usize,
}

impl Node {
    pub fn new() -> Node {
        Node {
            children: vec!(),
            data: 0
        }
    }

    pub fn expand(&mut self) {
        self.children = vec!(Node::new(), Node::new());
    }

    pub fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }

    fn expand_leaf_and_inc(&mut self) {
        if self.is_leaf() {
            self.expand();
        } else {
            let index = 0;
            self.children[index].expand_leaf_and_inc();
        }
        self.data += 1
    }
}

pub fn main() {
    let mut root = Node::new();

    for i in 0..2 {
        root.expand_leaf_and_inc();
        let index = 0;
        println!("i: {} ; root.data: {} ; root.children[{}]: {:#?}", i, root.data, index, root.children[index]);
    }

    //let mut root = Node::new();

    for i in 0..4 {
        let mut path = vec![];
        {
            let mut node = &mut root;
            // Descend and potential modify the node in the process
            while !node.is_leaf() {
                let index = 0;
                path.push(index);
                node = &mut {node}.children[index];
            }
            // Do something to the leaf node
            node.expand();
        }
        // Do something during "backup" (in my case it doesn't matter
        // in which order the modification is happening).
        let mut node = &mut root;
        for &i in path.iter() {
            node.data += 1;
            node = &mut {node}.children[i];
        }

        //let index = 0;
        //println!("i: {} ; root.data: {} ; root.children[{}]: {:#?}", i, root.data, index, root.children[index]);
    }
}
