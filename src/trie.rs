use std::collections::HashMap;

struct TrieNode {
    children: HashMap<String, Box<TrieNode>>, // Assuming lowercase English letters
    is_end: bool,
}

impl TrieNode {
    fn new() -> Self {
        TrieNode {
            children: HashMap::new(),
            is_end: false,
        }
    }
}

pub struct Trie {
    root: TrieNode,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            root: TrieNode::new(),
        }
    }

    pub fn insert(&mut self, value: &[&str]) {
        let mut current = &mut self.root;

        for v in value.iter().copied() {
            if current.children.get(v).is_none() {
                current
                    .children
                    .insert(v.to_string(), Box::new(TrieNode::new()));
            }

            if current.is_end {
                current.is_end = false;
            }

            current = current.children.get_mut(v).unwrap().as_mut();
        }

        current.is_end = true;
    }

    #[allow(dead_code)]
    pub fn search(&self, value: &[&str]) -> bool {
        let mut current = &self.root;

        for v in value.iter().copied() {
            if current.children.get(v).is_none() {
                return false;
            }
            current = current.children[v].as_ref();
        }

        current.is_end
    }

    pub fn starts_with(&self, value: &[&str]) -> bool {
        let mut current = &self.root;

        for v in value.iter().copied() {
            if current.children.get(v).is_none() {
                return current.is_end;
            }
            current = current.children[v].as_ref();
        }

        true
    }
}

#[test]
fn test() {
    let mut trie = Trie::new();

    trie.insert(&["a", "p", "p", "l", "e"]);
    trie.insert(&["b", "a", "n", "a", "n", "a"]);
    trie.insert(&["g", "r", "a", "p", "e"]);
    trie.insert(&["c"]);

    assert!(trie.starts_with(&["a", "p", "p", "l", "e", "_"]));
    assert!(!trie.starts_with(&["a", "p", "p", "l", "l", "_"]));

    assert!(trie.search(&["a", "p", "p", "l", "e"]));
    assert!(trie.search(&["b", "a", "n", "a", "n", "a"]));
    assert!(!trie.search(&["o", "r", "a", "n", "g", "e"]));

    assert!(trie.starts_with(&["a", "p", "p"]));
    assert!(trie.starts_with(&["g", "r"]));
    assert!(!trie.starts_with(&["p", "e", "a"]));

    assert!(trie.starts_with(&["c"]));
    assert!(trie.starts_with(&["c", "s"]));
}
