/// A packge to find, optionally from a specific repository.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub struct Target<'a> {
    /// The repository the package should come from. None for any repository.
    pub repo: Option<&'a str>,
    /// The name of the package, may also contain a version constraint.
    pub pkg: &'a str,
}

impl<'a> From<&'a str> for Target<'a> {
    fn from(s: &'a str) -> Self {
        let mut split = s.split('/');
        let first = split.next().unwrap();
        let repo;
        let pkg;

        if let Some(p) = split.next() {
            repo = Some(first);
            pkg = p;
        } else {
            repo = None;
            pkg = first;
        }

        Target { repo, pkg }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target() {
        let pkg = "repo/pkg";
        let pkg2 = String::from("pkg2");

        let target = Target::from(pkg);
        let target2 = Target::from(pkg2.as_str());

        assert_eq!(target.repo, Some("repo"));
        assert_eq!(target.pkg, "pkg");
        assert_eq!(target2.repo, None);
        assert_eq!(target2.pkg, "pkg2");
    }
}
