#[derive(Debug, Clone)]
pub struct Payload(pub Vec<u8>);

impl From<&[u8]> for Payload {
    fn from(value: &[u8]) -> Self {
        Self(value.into())
    }
}

impl AsRef<[u8]> for Payload {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Payload {
    fn get(&self, index: usize) -> u8 {
        *unsafe { self.as_ref().get_unchecked(index) }
    }

    pub fn id(&self) -> u16 {
        let (a, b) = (self.get(0) as u16, self.get(1) as u16);
        a << 8 | b
    }

    pub fn domain(&self) -> (Vec<&[u8]>, usize) {
        let mut domain: Vec<&[u8]> = Vec::new();
        // default offset = 12
        let mut current_offset = 12;

        loop {
            let label_length = self.0[current_offset] as usize;

            if label_length == 0 {
                break;
            }

            // Regular label
            let label = &self.0[current_offset + 1..current_offset + 1 + label_length];
            domain.push(label);

            // Move to the next label
            current_offset += label_length + 1;
        }

        domain.reverse();

        (domain, current_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_work_id_from_slice() {
        assert_eq!(Payload::from(&[0u8, 0][..]).id(), 0);
        assert_eq!(Payload::from(&[0u8, 2][..]).id(), 2);
        assert_eq!(Payload::from(&[1u8, 0][..]).id(), 256);
        assert_eq!(Payload::from(&[1u8, 2][..]).id(), 258);
    }

    #[test]
    fn it_work_domain() {
        let b = [
            0x8fu8, 0xd6, 0x01, 0x20, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x03, 0x78,
            0x72, 0x31, 0x05, 0x76, 0x6c, 0x70, 0x65, 0x72, 0x03, 0x74, 0x6f, 0x70, 0x00, 0x00,
            0x01, 0x00, 0x01, 0x00, 0x00, 0x29, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let rel = Payload::from(&b[..]);
        let (domain, offset) = rel.domain();
        let domain2: Vec<&[u8]> = vec![b"top", b"vlper", b"xr1"];
        assert_eq!(domain, domain2);
        assert_eq!(offset, 26);
    }
}
