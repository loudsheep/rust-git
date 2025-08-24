use anyhow::{Context, Result, bail};

#[derive(Debug, Clone)]
pub struct Kvlm {
    /// Ordered headers: (key, value) pairs, in the exact order parsed/added.
    pub headers: Vec<(Vec<u8>, Vec<u8>)>,
    /// Raw message bytes (everything after the blank line).
    pub message: Vec<u8>,
}

impl Kvlm {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            message: Vec::new(),
        }
    }

    pub fn values<'a>(&'a self, key: &[u8]) -> impl Iterator<Item = &'a [u8]> {
        self.headers.iter().filter_map(move |(k, v)| {
            if k.as_slice() == key {
                Some(v.as_slice())
            } else {
                None
            }
        })
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.values(key).next()
    }
}

pub fn kvlm_parse(raw: &[u8]) -> Result<Kvlm> {
    let mut kvlm = Kvlm::new();
    let mut pos = 0usize;

    if raw.is_empty() {
        return Ok(kvlm);
    }

    let next_new_line = |from: usize| -> Result<usize> {
        raw[from..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|off| from + off)
            .context("Expected new line in KVLM")
    };

    loop {
        let spc = raw[pos..]
            .iter()
            .position(|&b| b == b' ')
            .map(|off| pos + off);
        let nl = raw[pos..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|off| pos + off);

        match (spc, nl) {
            (_, Some(nlpos)) if spc.map(|s| s > nlpos).unwrap_or(true) => {
                if nlpos != pos {
                    bail!("malformed KVLM: expected blank line at headers/message boundary");
                }

                kvlm.message = raw.get(nlpos + 1..).unwrap_or(&[]).to_vec();
                break;
            }
            (Some(spcpos), Some(_nlpos)) => {
                let key = raw[pos..spcpos].to_vec();
                let mut end = spcpos;

                loop {
                    let nlpos = next_new_line(end + 1)?;
                    if nlpos + 1 < raw.len() && raw[nlpos + 1] == b' ' {
                        end = nlpos;
                    } else {
                        end = nlpos;
                        break;
                    }
                }

                let slice = &raw[spcpos + 1..end];
                let mut val = Vec::with_capacity(slice.len());
                let mut i = 0usize;
                while i < slice.len() {
                    if slice[i] == b'\n' && i + 1 < slice.len() && slice[i + 1] == b' ' {
                        val.push(b'\n');
                        i += 2;
                    } else {
                        val.push(slice[i]);
                        i += 1;
                    }
                }

                kvlm.headers.push((key, val));

                pos = end + 1;
                if pos >= raw.len() {
                    kvlm.message.clear();
                    break;
                }
            }
            _ => bail!("malformed KVLM: missing space/newline in header"),
        }
    }

    Ok(kvlm)
}

pub fn kvlm_serialize(kvlm: &Kvlm) -> Vec<u8> {
    let mut out = Vec::new();

    for (k, v) in &kvlm.headers {
        out.extend_from_slice(k);
        out.push(b' ');
        let mut i = 0usize;
        while i < v.len() {
            if v[i] == b'\n' {
                out.push(b'\n');
                out.push(b' ');
                i += 1;
            } else {
                out.push(v[i]);
                i += 1;
            }
        }
        out.push(b'\n');
    }

    out.push(b'\n');
    out.extend_from_slice(&kvlm.message);
    out
}
