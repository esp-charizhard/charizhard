use heapless::String;
use serde::Deserialize;

/// Wrapper on [`heapless::String`] for additional capabilities.
#[derive(Deserialize, Default)]
pub struct HeaplessString<const N: usize>(pub String<N>);

impl<const N: usize> HeaplessString<N> {
    pub fn new() -> Self {
        Self(String::<N>::new())
    }

    pub fn push_str(&mut self, s: &str) -> anyhow::Result<()> {
        if s.len() > N {
            return Err(anyhow::anyhow!("String too long."));
        }

        _ = self.0.push_str(s);

        Ok(())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    fn chars(&self) -> std::str::Chars<'_> {
        self.0.chars()
    }

    pub fn clean_string(&self) -> HeaplessString<N> {
        self.chars()
            .filter(|&c| c.is_ascii() && (c.is_ascii_graphic() || c.is_ascii_whitespace()))
            .collect()
    }
}

impl<const N: usize> FromIterator<char> for HeaplessString<N> {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let mut heapless_string = HeaplessString::<N>::new();

        for c in iter {
            if heapless_string.0.len() < N {
                if heapless_string.0.push(c).is_err() {
                    break;
                }
            } else {
                break;
            }
        }

        heapless_string
    }
}

impl<const N: usize> TryInto<heapless::String<N>> for HeaplessString<N> {
    type Error = anyhow::Error;

    fn try_into(self) -> anyhow::Result<heapless::String<N>> {
        Ok(self.0)
    }
}
