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
        let mut values = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| {
                let mut d: Vec<_> = line.split(".").collect();
                d.reverse();
                d
            })
            .collect::<Vec<_>>();
        values.sort();

        let mut trie: Self = Trie::new().into();
        for value in values {
            trie.insert(value);
        }

        Ok(trie)
    }
}

impl DomainTrie {
    pub fn domain_prefix_match<I, T>(&self, domain: I) -> bool
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
        <I as IntoIterator>::IntoIter: DoubleEndedIterator,
    {
        self.prefix_match(domain.into_iter().rev())
    }
}
