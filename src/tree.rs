use taffy::{NodeId, PrintTree, TaffyTree, TraversePartialTree};
use gosub_html5::node::NodeId as GosubId;
use gosub_styling::render_tree::{RenderNodeData, RenderTree};

pub fn print_tree(tree: &TaffyTree<GosubId>, root: NodeId, gosub_tree: &RenderTree) {
    println!("TREE");
    print_node(tree, root, false, String::new(), gosub_tree);

    /// Recursive function that prints each node in the tree
    fn print_node(tree: &TaffyTree<GosubId>, node_id: NodeId, has_sibling: bool, lines_string: String, gosub_tree: &RenderTree) {
        let layout = &tree.get_final_layout(node_id);
        let display = tree.get_debug_label(node_id);
        let num_children = tree.child_count(node_id);
        let gosub_id = tree.get_node_context(node_id).unwrap();
        
        let fork_string = if has_sibling { "├── " } else { "└── " };
        let node = gosub_tree.get_node(*gosub_id).unwrap();
        let mut node_render = String::new();
        
        match &node.data {
            RenderNodeData::Element(element) => {
                node_render.push('<');
                node_render.push_str(&element.name);
                for (key, value) in element.attributes.iter() {
                    node_render.push_str(&format!(" {}=\"{}\"", key, value));
                }
                node_render.push('>');
            },
            RenderNodeData::Text(text) => {
                let text = text.text.replace('\n', " ");
                node_render.push_str(text.trim());
            },
            
            _ => {}
        }

        println!(
            "{lines}{fork} {display} [x: {x:<4} y: {y:<4} width: {width:<4} height: {height:<4}] ({key:?}) |{node_render}|",
            lines = lines_string,
            fork = fork_string,
            display = display,
            x = layout.location.x,
            y = layout.location.y,
            width = layout.size.width,
            height = layout.size.height,
            key = node_id,
        );
        let bar = if has_sibling { "│   " } else { "    " };
        let new_string = lines_string + bar;

        // Recurse into children
        for (index, child) in tree.child_ids(node_id).enumerate() {
            let has_sibling = index < num_children - 1;
            print_node(tree, child, has_sibling, new_string.clone(), gosub_tree);
        }
    }
}