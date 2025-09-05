use std::collections::HashMap;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            root: TrieNode::new(),
        }
    }

    pub fn insert<I, T>(&mut self, values: I)
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut current = &mut self.root;

        for v in values {
            let v = v.as_ref();
            if current.children.get(v).is_none() {
                current
                    .children
                    .insert(v.to_owned(), Box::new(TrieNode::new()));
            }

            current = current.children.get_mut(v).unwrap().as_mut();

            if current.is_end {
                return;
            }
        }

        current.is_end = true;
    }

    pub fn prefix_match<I, T>(&self, values: I) -> bool
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut current = &self.root;

        for v in values {
            let v = v.as_ref();
            if current.children.get(v).is_none() {
                return false;
            }
            current = current.children[v].as_ref();
            if current.is_end {
                return true;
            }
        }

        false
    }
}

#[test]
fn test() {
    let mut trie = Trie::new();

    trie.insert(["a", "p", "p"]);
    trie.insert(["a", "p", "p", "l", "e"]);
    trie.insert(["b", "a", "n", "a", "n", "a"]);
    trie.insert(["g", "r", "a", "p", "e"]);
    trie.insert(["c"]);

    // dbg!(&trie);

    assert!(!trie.prefix_match(["a", "p"]));
    assert!(trie.prefix_match(["a", "p", "p"]));
    assert!(trie.prefix_match(["a", "p", "p", "l", "e"]));
    assert!(trie.prefix_match(["a", "p", "p", "l", "l"]));

    assert!(!trie.prefix_match(&["g", "r"]));
    assert!(!trie.prefix_match(&["p", "e", "a"]));

    assert!(trie.prefix_match(&["c"]));
    assert!(trie.prefix_match(&["c", "s"]));

    assert!(trie.prefix_match(&["a", "p", "p", "l", "e"]));
    assert!(trie.prefix_match(&["b", "a", "n", "a", "n", "a"]));
    assert!(!trie.prefix_match(&["o", "r", "a", "n", "g", "e"]));
}
