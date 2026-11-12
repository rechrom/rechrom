//! Chromium net/extras/preload_data/decoder.cc:18-319 and
//! idn_spoof_checker.cc:34-93,648-686. Immutable Huffman preload data.
use crate::{Error, SkeletonType, TopDomainEntry};
mod data {
    include!("top_domains_data.rs");
}
const HUFFMAN: &[u8] = include_bytes!("../data/top_domains_huffman.bin");
const TRIE: &[u8] = include_bytes!("../data/top_domains_trie.bin");
struct BitReader {
    offset: usize,
}
impl BitReader {
    fn bit(&mut self) -> Result<u32, Error> {
        // Next reads the final backing byte's padding as in the source.
        let byte = *TRIE.get(self.offset / 8).ok_or(Error::CorruptTrie)?;
        let bit = (byte >> (7 - self.offset % 8)) & 1;
        self.offset += 1;
        Ok(bit.into())
    }
    fn read(&mut self, n: u32) -> Result<u32, Error> {
        if n > 32 {
            return Err(Error::CorruptTrie);
        }
        let mut value = 0;
        for _ in 0..n {
            value = (value << 1) | self.bit()?;
        }
        Ok(value)
    }
    fn seek(&mut self, offset: usize) -> Result<(), Error> {
        if offset >= data::TRIE_BITS {
            return Err(Error::CorruptTrie);
        }
        self.offset = offset;
        Ok(())
    }
    fn size(&mut self) -> Result<usize, Error> {
        let prefix = self.read(2)?;
        if prefix == 0 {
            return Ok(0);
        }
        let prefix = (prefix << 1) | self.bit()?;
        match prefix {
            2 => Ok(4),
            4 => Ok(1),
            5 => Ok(2),
            6 => Ok(3),
            3 | 7 => {
                let mut length = 3;
                loop {
                    length += 1;
                    if self.bit()? == 0 {
                        break;
                    }
                }
                Ok((length - 2) * 2 - usize::from(prefix == 7))
            }
            _ => Err(Error::CorruptTrie),
        }
    }
    fn character(&mut self) -> Result<u8, Error> {
        let mut offset = HUFFMAN.len().checked_sub(2).ok_or(Error::CorruptTrie)?;
        loop {
            let next = *HUFFMAN
                .get(offset + self.bit()? as usize)
                .ok_or(Error::CorruptTrie)?;
            if next & 128 != 0 {
                return Ok(next & 127);
            }
            offset = usize::from(next) * 2;
        }
    }
    fn entry(&mut self, search: &str) -> Result<TopDomainEntry, Error> {
        let same = self.bit()? != 0;
        let is_top_bucket = self.bit()? != 0;
        let skeleton_type = if self.bit()? == 0 {
            SkeletonType::Full
        } else {
            SkeletonType::SeparatorsRemoved
        };
        let domain = if same {
            search.into()
        } else {
            let com = self.bit()? != 0;
            let mut domain = String::new();
            loop {
                let c = self.character()?;
                if c == 127 {
                    break;
                }
                domain.push(char::from(c));
            }
            if com {
                domain.push_str(".com");
            }
            domain
        };
        Ok(TopDomainEntry {
            domain,
            is_top_bucket,
            skeleton_type,
        })
    }
}
pub(crate) fn decode(search: &str) -> Result<Option<TopDomainEntry>, Error> {
    let mut reader = BitReader { offset: 0 };
    let mut offset = data::TRIE_ROOT;
    let mut remaining = search.len();
    loop {
        reader.seek(offset)?;
        for _ in 0..reader.size()? {
            if remaining == 0 {
                return Ok(None);
            }
            if search.as_bytes()[remaining - 1] != reader.character()? {
                return Ok(None);
            }
            remaining -= 1;
        }
        let mut first = true;
        let mut current = 0;
        loop {
            let c = reader.character()?;
            if c == 127 {
                return Ok(None);
            }
            if c == 0 {
                let entry = reader.entry(search)?;
                if remaining == 0 {
                    return Ok(Some(entry));
                }
                continue;
            }
            if remaining == 0 || search.as_bytes()[remaining - 1] < c {
                return Ok(None);
            }
            if first {
                let width = reader.read(5)?;
                let delta = reader.read(width)? as usize;
                current = offset.checked_sub(delta).ok_or(Error::CorruptTrie)?;
                first = false;
            } else {
                let width = if reader.bit()? == 0 {
                    7
                } else {
                    reader.read(4)? + 8
                };
                current = current
                    .checked_add(reader.read(width)? as usize)
                    .ok_or(Error::CorruptTrie)?;
                if current >= offset {
                    return Err(Error::CorruptTrie);
                }
            }
            if search.as_bytes()[remaining - 1] == c {
                offset = current;
                remaining -= 1;
                break;
            }
        }
    }
}
pub(crate) fn lookup(skeleton: &str, kind: SkeletonType) -> Result<Option<TopDomainEntry>, Error> {
    let mut labels = skeleton.split('.').collect::<Vec<_>>();
    if labels.len() > 4 {
        labels.drain(..labels.len() - 4);
    }
    while !labels.is_empty() {
        if labels.len() == 1 && kind == SkeletonType::Full {
            break;
        }
        if let Some(found) = decode(&labels.join("."))? {
            return Ok(Some(found));
        }
        labels.remove(0);
    }
    Ok(None)
}
