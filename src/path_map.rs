use core::iter;

#[derive(Debug, Default)]
pub(crate) struct PathMap<T> {
    paths: Vec<(String, T)>,
}

impl<T> PathMap<T> {
    pub(crate) fn push(&mut self, path: String, val: T) {
        self.paths.push((path, val));
    }

    pub(crate) fn find_best_match(&self, path: &str) -> Option<&T> {
        sub_path_iter(path).find_map(|sub_path| {
            self.paths
                .iter()
                .find_map(|(p, v)| if p == sub_path { Some(v) } else { None })
        })
    }
}

fn sub_path_iter(path: &str) -> impl Iterator<Item = &str> {
    iter::once(path)
        .chain(suffixes(path))
        .chain(iter::once("."))
}

/// Return all suffixes
///
/// suffixes("authors.a.b") -> [".a.b",".b"]
///
fn suffixes(path: &str) -> impl Iterator<Item = &str> {
    iter::successors(Some(path), |s| {
        let mut dot_pos = s.find('.')?;
        if dot_pos == 0 {
            dot_pos = s[1..].find('.')? + 1;
        };

        s.split_at_checked(dot_pos)
            .map(|(_head, tail)| tail)
            .filter(|p| !p.is_empty())
    })
    .skip(1)
}

/// Return all prefixes
///
/// prefixes("authors.a.b") -> ["authors.a","authors"]
///
fn prefixes(path: &str) -> impl Iterator<Item = &str> {
    iter::successors(Some(path), |s| {
        s.rsplit_once('.')
            .map(|(head, _tail)| head)
            .filter(|p| !p.is_empty())
    })
    .skip(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_suffixes() {
        let mut it = suffixes("authors.a.b");

        assert_eq!(it.next(), Some(".a.b"));
        assert_eq!(it.next(), Some(".b"));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_prefixes() {
        let mut it = prefixes("authors.a.b");

        assert_eq!(it.next(), Some("authors.a"));
        assert_eq!(it.next(), Some("authors"));
        assert_eq!(it.next(), None);
    }
}
