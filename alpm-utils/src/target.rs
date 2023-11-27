use std::fmt;

/// Trait for being generic over Target and Targ
pub trait AsTarg {
    /// Converts to a targ.
    fn as_targ(&self) -> Targ;
}

impl<T> AsTarg for T
where
    T: AsRef<str>,
{
    fn as_targ(&self) -> Targ {
        Targ::from(self.as_ref())
    }
}

/// A packge to find, optionally from a specific repository.
#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub struct Target {
    /// The repository the package should come from. None for any repository.
    pub repo: Option<String>,
    /// The name of the package, may also contain a version constraint.
    pub pkg: String,
}

impl AsTarg for Target {
    fn as_targ(&self) -> Targ {
        Targ::new(self.repo.as_deref(), &self.pkg)
    }
}

impl Target {
    /// Create a new Target.
    pub fn new<S: Into<String>>(repo: Option<S>, pkg: S) -> Target {
        Target {
            repo: repo.map(Into::into),
            pkg: pkg.into(),
        }
    }
}

/// A packge to find, optionally from a specific repository.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub struct Targ<'a> {
    /// The repository the package should come from. None for any repository.
    pub repo: Option<&'a str>,
    /// The name of the package, may also contain a version constraint.
    pub pkg: &'a str,
}

impl<'a> Targ<'a> {
    /// Create a new Targ.
    pub fn new(repo: Option<&'a str>, pkg: &'a str) -> Targ<'a> {
        Targ { repo, pkg }
    }
}

impl<'a> AsTarg for Targ<'a> {
    fn as_targ(&self) -> Targ {
        *self
    }
}

impl<'a, S: AsRef<str> + ?Sized> From<&'a S> for Targ<'a> {
    fn from(s: &'a S) -> Self {
        let mut split = s.as_ref().split('/');
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

        Targ { repo, pkg }
    }
}

impl<'a> fmt::Display for Targ<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(repo) = self.repo {
            write!(fmt, "{}/{}", repo, self.pkg)
        } else {
            write!(fmt, "{}", self.pkg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target() {
        let pkg = "repo/pkg";
        let pkg2 = String::from("pkg2");

        let target = Targ::from(pkg);
        let target2 = Targ::from(pkg2.as_str());

        assert_eq!(target.repo, Some("repo"));
        assert_eq!(target.pkg, "pkg");
        assert_eq!(target2.repo, None);
        assert_eq!(target2.pkg, "pkg2");
    }
}
