use ahash::AHashMap;

#[derive(Debug)]
struct TrieNode {
    children: AHashMap<Box<[u8]>, TrieNode>,
    is_end: bool,
}

impl TrieNode {
    fn new() -> Self {
        TrieNode {
            children: AHashMap::new(),
            is_end: false,
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        TrieNode {
            children: AHashMap::with_capacity(capacity),
            is_end: false,
        }
    }
}

#[derive(Debug)]
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    pub fn with_capacity(capacity: usize) -> Self {
        Trie {
            root: TrieNode::with_capacity(capacity),
        }
    }

    /// 插入字节序列
    pub fn insert<I, T>(&mut self, values: I)
    where
        I: IntoIterator<Item = T>,
        T: AsRef<[u8]>,
    {
        let mut current = &mut self.root;

        for v in values {
            let v = v.as_ref();

            current = current
                .children
                .entry(v.into())
                .or_insert_with(TrieNode::new);

            if current.is_end {
                return;
            }
        }

        current.is_end = true;
    }

    /// 构建后压缩内存
    pub fn shrink_to_fit(&mut self) {
        fn shrink_node(node: &mut TrieNode) {
            node.children.shrink_to_fit();
            for child in node.children.values_mut() {
                shrink_node(child);
            }
        }
        shrink_node(&mut self.root);
    }

    /// 前缀匹配（支持字节序列）
    #[inline]
    pub fn prefix_match<I, T>(&self, values: I) -> bool
    where
        I: IntoIterator<Item = T>,
        T: AsRef<[u8]>,
    {
        let mut current = &self.root;

        for v in values {
            let v = v.as_ref();
            match current.children.get(v) {
                Some(node) => {
                    current = node;
                    if current.is_end {
                        return true;
                    }
                }
                None => return false,
            }
        }

        false
    }
}

#[test]
fn test() {
    let mut trie = Trie::with_capacity(100);

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
