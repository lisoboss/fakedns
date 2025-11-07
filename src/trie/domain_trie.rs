use super::Trie;
use std::{
    fs::read_to_string,
    io,
    ops::{Deref, DerefMut},
    path::Path,
};

pub struct DomainTrie(Trie);

impl Deref for DomainTrie {
    type Target = Trie;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DomainTrie {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Trie> for DomainTrie {
    fn from(trie: Trie) -> Self {
        Self(trie)
    }
}

impl TryFrom<&Path> for DomainTrie {
    type Error = io::Error;

    fn try_from(filename: &Path) -> Result<Self, Self::Error> {
        let content = read_to_string(filename)?;

        let estimated_domains = content.len() / 20;
        let mut trie: Self = Trie::with_capacity(estimated_domains).into();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let normalized = line.to_ascii_lowercase();
            let parts: Vec<&[u8]> = normalized.as_bytes().split(|&b| b == b'.').rev().collect();

            trie.insert(parts);
        }

        trie.shrink_to_fit();

        Ok(trie)
    }
}

impl DomainTrie {
    #[inline]
    pub fn domain_prefix_match<I, T>(&self, reversed_domain: I) -> bool
    where
        I: IntoIterator<Item = T>,
        T: AsRef<[u8]>,
        <I as IntoIterator>::IntoIter: DoubleEndedIterator,
    {
        self.prefix_match(reversed_domain.into_iter())
    }
}
