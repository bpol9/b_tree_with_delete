use std::convert::TryFrom;
use std::fmt::Debug;
use std::cmp::PartialEq;
use std::mem;
use std::rc::Rc;
use std::cell::RefCell;

struct Node<T> {
    keys: Vec<T>,
    children: Vec<Node<T>>,
    parent: Option<Rc<RefCell<Node<T>>>>,
    parent_index: usize,
}

pub struct BTree<T> {
    root: Node<T>,
    props: BTreeProps,
}

// Why to need a different Struct for props...
// Check - http://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/#fnref:improvement
struct BTreeProps {
    degree: usize,
    max_keys: usize,
    min_keys: usize,
    mid_key_index: usize,
}

impl<T> Node<T>
where
    T: Ord,
{
   fn new(degree: usize, _keys: Option<Vec<T>>, _children: Option<Vec<Node<T>>>, _parent: Option<Rc<RefCell<Node<T>>>>, _parent_index: usize) -> Self {
        Node {
            keys: match _keys {
                Some(_keys) => _keys,
                None => Vec::with_capacity(degree - 1),
            },
            children: match _children {
                Some(_children) => _children,
                None => Vec::with_capacity(degree),
            },
            parent: _parent,
            parent_index: _parent_index,
        }
   }

   fn is_leaf(&self) -> bool {
		return self.children.len() == 0
   }
	 
	fn has_right_sibling(&self) -> bool {
        match self.parent {
            Some(ref node) => node.borrow().children.len() > self.parent_index + 1,
            None => false,
        }
	}
	 
	fn has_left_sibling(&self) -> bool {
        match self.parent {
            None => false,
            Some(_) => self.parent_index > 0,
        }
	}
	
	fn is_root(&self) -> bool {
		match self.parent {
            None => true,
            Some(_) => false,
        }
	}

    // caller must already check existence of right sibling
    /*
    fn right_sibling(&self) -> &mut Node<T> {
        match self.parent {
            None => panic!("Called right_sibling method of root node"),
            Some(ref p) => &mut (p.borrow_mut().children[self.parent_index + 1])
        }
    }
    */

    // caller must already check existence of left sibling
    /*
    fn left_sibling(&self) -> &mut Node<T> {
        match self.parent {
            None => panic!("left_sibling method called on root"),
            Some(ref p) => &mut (p.borrow_mut().children[self.parent_index - 1])
        }
	}
    */
}

impl BTreeProps {
    fn new(degree: usize) -> Self {
        BTreeProps {
            degree,
            max_keys: degree - 1,
            min_keys: (degree - 1) / 2,
            mid_key_index: (degree - 1) / 2,
        }
    }

    fn is_maxed_out<T: Ord + Copy>(&self, node: &Node<T>) -> bool {
        node.keys.len() == self.max_keys
    }
	 
	 fn can_donate_from_left_sibling<T: Ord + Copy>(&self, node: &Node<T>) -> bool {
         if !node.has_left_sibling() {
             return false;
         }
         match node.parent {
             None => false,
             Some(ref p) => p.borrow().children[node.parent_index - 1].keys.len() > self.min_keys,
         }
	 }
	 
	 fn can_donate_from_right_sibling<T: Ord + Copy>(&self, node: &Node<T>) -> bool {
         if !node.has_right_sibling() {
             return false;
         }
         match node.parent {
             None => false,
             Some(ref p) => p.borrow().children[node.parent_index + 1].keys.len() > self.min_keys,
         }
	 }

    // Split Child expects the Child Node to be full
    /// Move the middle_key to parent node and split the child_node's
    /// keys/chilren_nodes into half
    fn split_child<T: Ord + Copy + Default>(&self, parent: &mut Node<T>, child_index: usize) {
        let child = &mut parent.children[child_index];
        let middle_key = child.keys[self.mid_key_index];
        let right_keys = match child.keys.split_off(self.mid_key_index).split_first() {
            Some((_first, _others)) => {
                // We don't need _first, as it will move to parent node.
                _others.to_vec()
            }
            None => Vec::with_capacity(self.max_keys),
        };
        let mut right_children = None;
        if !child.is_leaf() {
            right_children = Some(child.children.split_off(self.mid_key_index + 1));
        }
        let new_child_node: Node<T> = Node::new(self.degree, Some(right_keys), right_children, child.parent.clone(), child_index + 1);

        parent.keys.insert(child_index, middle_key);
        parent.children.insert(child_index + 1, new_child_node);
    }

    fn insert_non_full<T: Ord + Copy + Default>(&mut self, node: &mut Node<T>, key: T) {
        let mut index: isize = isize::try_from(node.keys.len()).ok().unwrap() - 1;
        while index >= 0 && node.keys[index as usize] >= key {
            index -= 1;
        }

        let mut u_index: usize = usize::try_from(index + 1).ok().unwrap();
        if node.is_leaf() {
            // Just insert it, as we know this method will be called only when node is not full
            node.keys.insert(u_index, key);
        } else {
            if self.is_maxed_out(&node.children[u_index]) {
                self.split_child(node, u_index);
                if node.keys[u_index] < key {
                    u_index += 1;
                }
            }

            self.insert_non_full(&mut node.children[u_index], key);
        }
    }

    fn traverse_node<T: Ord + Debug>(&self, node: &Node<T>, depth: usize) {
        if node.is_leaf() {
            print!(" {0:{<1$}{2:?}{0:}<1$} ", "", depth, node.keys);
        } else {
            let _depth = depth + 1;
            for (index, key) in node.keys.iter().enumerate() {
                self.traverse_node(&node.children[index], _depth);
                // Check https://doc.rust-lang.org/std/fmt/index.html
                // And https://stackoverflow.com/a/35280799/2849127
                print!("{0:{<1$}{2:?}{0:}<1$}", "", depth, key);
            }
            self.traverse_node(&node.children.last().unwrap(), _depth);
        }
    }
	 
	 
	fn delete_key<T: Ord + Copy + Debug + PartialEq>(&self, node: &mut Node<T>, key: T) {
		if node.is_leaf() {
			self.remove_key_from_node(node, key);
			self.rebalance_after_deletion(node);
		}
		else {		
			
		    let key_index = node.keys.iter().position(|&e| e == key).unwrap();
		
            {
		        let mut leaf_left = &mut node.children[key_index];
		        while !leaf_left.is_leaf() {
			        leaf_left = leaf_left.children.last_mut().unwrap();
		        }
                if leaf_left.keys.len() > self.min_keys {
                    let new_sep = leaf_left.keys.pop().unwrap();
                    self.rebalance_after_deletion(leaf_left);
                    self.replace_keys(node, key, new_sep);
                    return;
                }
            }

            {
		
                let mut leaf_right = &mut node.children[key_index + 1];
                while !leaf_right.is_leaf() {
                    leaf_right = leaf_right.children.first_mut().unwrap();
                }
				let new_sep = leaf_right.keys.remove(0);
				self.rebalance_after_deletion(leaf_right);
				self.replace_keys(node, key, new_sep);
            }

		}
	}
	
	fn remove_key_from_node<T: PartialEq>(&self, node: &mut Node<T>, key: T) {
		if let Some(pos) = node.keys.iter().position(|x| *x == key) {
			node.keys.remove(pos);
		}
	}
	
    /*
	fn get_donor_leafs<T: Ord, PartialEq>(&self, node: &Node<T>, key: T) -> (&mut Node<T>, &mut Node<T>) {
		let key_index = node.keys.iter().position(|&e| e == key).unwrap();
		
		let left_leaf = &mut node.children[key_index];
		while !left_leaf.is_leaf() {
			left_leaf = left_leaf.children.last_mut().unwrap();
		}
		
		let right_leaf = &mut node.children[key_index + 1];
		while !right_leaf.is_leaf() {
			right_leaf = right_leaf.children.first_mut().unwrap();
		}
		
		return (left_leaf, right_leaf);
		
	}
    */
	
	fn replace_keys<T: PartialEq>(&self, node: &mut Node<T>, old_key: T, new_key: T) {
		let index = node.keys.iter().position(|e| *e == old_key).unwrap();
		node.keys[index] = new_key;
	}
	
	fn rebalance_after_deletion<T: Ord + Copy>(&self, node: &mut Node<T>) {
		
		if node.is_root() || node.keys.len() >= self.min_keys {
			return;
		}
		
						
		if self.can_donate_from_right_sibling(&node) {
			self.donate_from_right(node);
		}
		else if self.can_donate_from_left_sibling(&node) {
			self.donate_from_left(node);
		}
		else if node.has_right_sibling() {
			self.merge_with_right(node);
            match node.parent {
                None => return, // panic here, parent can't be none
                Some(ref parent) => self.rebalance_after_deletion(&mut parent.borrow_mut()), // parent lost one key during merge, check if she needs rebalance.
            }
				
		}
		else if node.has_left_sibling() {
			self.merge_with_left(node);
            match node.parent {
                None => return, // panic, parent can't be None
                Some(ref parent) => self.rebalance_after_deletion(&mut parent.borrow_mut()), // parent lost one key during merge, check if she needs rebalance.
            }
		}
	}
	
	fn donate_from_right<T: Ord + Copy>(&self, node: &mut Node<T>) {
        match node.parent {
            None => return, // panic, parent can't be None
            Some(ref parent_cell) => {
                let parent = &mut parent_cell.borrow_mut();
    		    //let sibling = node.right_sibling();
                let sibling = &mut parent.children[node.parent_index + 1];
	         	let sibling_key = sibling.keys.remove(0);
                if !node.is_leaf() {
                    let sibling_child = sibling.children.remove(0);
    	    	    node.children.push(sibling_child);
                }
	    	    let parent_key = std::mem::replace(&mut parent.keys[node.parent_index], sibling_key);
	    	    node.keys.push(parent_key);
            }
        }
	}
	
	fn donate_from_left<T: Ord + Copy>(&self, node: &mut Node<T>) {
        match node.parent {
            None => return, // panic, parent can't be None
            Some(ref n) => {
                let parent = &mut n.borrow_mut();
		        let sibling = &mut parent.children[node.parent_index - 1];
	        	let sibling_key = sibling.keys.pop().unwrap();
                if !node.is_leaf() {
                    let sibling_child = sibling.children.pop().unwrap();
    		        node.children.insert(0, sibling_child);
                }
		        let parent_key = std::mem::replace(&mut parent.keys[node.parent_index - 1], sibling_key);
		        node.keys.insert(0, parent_key);
            }
        }
	}

    fn merge_with_right<T: Ord>(&self, node: &mut Node<T>) {
        match node.parent {
            None => panic!("trying to merge root with right sibling"),
            Some(ref n) => {
                let parent = &mut n.borrow_mut();
                let right_sibling = &mut parent.children[node.parent_index + 1];
                let keys = &mut right_sibling.keys;
                let children = &mut right_sibling.children;
                node.children.append(children);
                node.keys.append(keys);
                parent.keys.remove(node.parent_index);
                parent.children.remove(node.parent_index + 1);
            }
        }
    }

    fn merge_with_left<T: Ord>(&self, node: &mut Node<T>) {
        match node.parent {
            None => panic!("trying to merge root with left sibling"),
            Some(ref n) => {
                let parent = &mut n.borrow_mut();
                let left_sibling = &mut parent.children[node.parent_index - 1];
                let keys = &mut node.keys;
                let children = &mut node.children;
                left_sibling.keys.append(keys);
                left_sibling.children.append(children);
                parent.keys.remove(node.parent_index - 1);
                parent.children.remove(node.parent_index);
            }
        }
    }

}

impl<T> BTree<T>
where
    T: Ord + Copy + Debug + Default,
{
    pub fn new(branch_factor: usize) -> Self {
        let degree = 2 * branch_factor;
        BTree {
            root: Node::new(degree, None, None, None, 0),
            props: BTreeProps::new(degree),
        }
    }

    pub fn insert(&mut self, key: T) {
        if self.props.is_maxed_out(&self.root) {
            // Create an empty root and split the old root...
            let mut new_root = Node::new(self.props.degree, None, None, None, 0);
            mem::swap(&mut new_root, &mut self.root);
            self.root.children.insert(0, new_root);
            self.props.split_child(&mut self.root, 0);
        }
        self.props.insert_non_full(&mut self.root, key);
    }

    pub fn traverse(&self) {
        self.props.traverse_node(&self.root, 0);
        println!("");
    }

    pub fn search(&self, key: T) -> bool {
        let mut current_node = &self.root;
        let mut index: isize;
        loop {
            index = isize::try_from(current_node.keys.len()).ok().unwrap() - 1;
            while index >= 0 && current_node.keys[index as usize] > key {
                index -= 1;
            }

            let u_index: usize = usize::try_from(index + 1).ok().unwrap();
            if index >= 0 && current_node.keys[u_index - 1] == key {
                break true;
            } else if current_node.is_leaf() {
                break false;
            } else {
                current_node = &current_node.children[u_index];
            }
        }
    }
	
	pub fn delete(&mut self, key: T) -> bool {
        let mut node: Option<&mut Node<T>> = None;
		let mut current_node = &mut self.root;
        let mut index: isize;
		loop {
			index = isize::try_from(current_node.keys.len()).ok().unwrap() - 1;
			while index >= 0 && current_node.keys[index as usize] > key {
				 index -= 1;
			}

			let u_index: usize = usize::try_from(index + 1).ok().unwrap();
			if index >= 0 && current_node.keys[u_index - 1] == key {
                 node = Some(current_node);
                 break;
			} else if current_node.is_leaf() {
				 break;
			} else {
				 current_node = &mut current_node.children[u_index];
			}
		}

		match node {
			None => false,
			Some(node) => {
				self.props.delete_key(node, key);
				if self.root.keys.len() == 0 {
                    /* if root is left with 0 keys, then its one and only child becomes the new root */
					self.root = self.root.children.pop().unwrap();
				}
				true
			}
		}
	}
	
	//fn find_node_with_key(&mut self, key: T) -> Option<&mut Node<T>> {
	//	let mut current_node = &mut self.root;
    //    let mut index: isize;
	//	loop {
	//		index = isize::try_from(current_node.keys.len()).ok().unwrap() - 1;
	//		while index >= 0 && current_node.keys[index as usize] > key {
	//			 index -= 1;
	//		}

	//		let u_index: usize = usize::try_from(index + 1).ok().unwrap();
	//		if index >= 0 && current_node.keys[u_index - 1] == key {
	//			 break Some(current_node);
	//		} else if current_node.is_leaf() {
	//			 break None;
	//		} else {
	//			 current_node = &mut current_node.children[u_index];
	//		}
	//	}
	//}
}

#[cfg(test)]
mod test {
    use super::BTree;

    #[test]
    fn test_search() {
        let mut tree = BTree::new(2);
        tree.insert(10);
        tree.insert(20);
        tree.insert(30);
        tree.insert(5);
        tree.insert(6);
        tree.insert(7);
        tree.insert(11);
        tree.insert(12);
        tree.insert(15);
        assert!(tree.search(15));
        assert_eq!(tree.search(16), false);
        //tree.delete(15);
        //assert_eq!(tree.search(15), false);
        //assert!(tree.search(12));
        //tree.delete(12);
        //assert_eq!(tree.search(12), false);
        tree.delete(10);
        assert_eq!(tree.search(10), false);
        assert!(tree.search(5));
        assert!(tree.search(7));
        assert!(tree.search(11));
        assert!(tree.search(12));
        assert!(tree.search(15));
        assert!(tree.search(30));
    }
}
