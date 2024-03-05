use crate::{AlpmList, AlpmListMut, BorrowAlpmListItem, IntoAlpmListItem, IntoAlpmListPtr};

use std::iter::FromIterator;

pub trait AsAlpmList<T> {
    #[doc(hidden)]
    fn with<R, F: FnOnce(AlpmList<T>) -> R>(self, f: F) -> R;
}

impl<'l, T> AsAlpmList<T> for AlpmList<'l, T> {
    fn with<R, F: FnOnce(AlpmList<'l, T>) -> R>(self, f: F) -> R {
        f(self)
    }
}

impl<'b, T: IntoAlpmListItem + BorrowAlpmListItem<'b>> AsAlpmList<T::Borrow> for AlpmListMut<T> {
    fn with<R, F: FnOnce(AlpmList<T::Borrow>) -> R>(self, f: F) -> R {
        f(self.list())
    }
}

impl<'b, T: IntoAlpmListItem + BorrowAlpmListItem<'b>> AsAlpmList<T::Borrow> for &AlpmListMut<T> {
    fn with<R, F: FnOnce(AlpmList<T::Borrow>) -> R>(self, f: F) -> R {
        f(self.list())
    }
}

impl<'a, T: IntoAlpmListPtr, I> AsAlpmList<<T::Output as BorrowAlpmListItem<'a>>::Borrow> for I
where
    I: Iterator<Item = T>,
    T::Output: BorrowAlpmListItem<'a>,
{
    fn with<R, F: FnOnce(AlpmList<<T::Output as BorrowAlpmListItem<'a>>::Borrow>) -> R>(
        self,
        f: F,
    ) -> R {
        let list = AlpmListMut::<T::Output>::from_iter(self);
        let list = list.list();
        f(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dep, Depend};

    fn foo<'a>(list: impl AsAlpmList<&'a Dep>) {
        list.with(|list| assert_eq!(list.iter().nth(1).unwrap().name(), "bb"));
    }

    fn bar<'a>(list: impl AsAlpmList<&'a str>) {
        list.with(|list| assert_eq!(list.iter().nth(1).unwrap(), "bb"));
    }

    fn deps() -> Vec<Depend> {
        vec![Depend::new("aa"), Depend::new("bb"), Depend::new("cc")]
    }

    fn deps2() -> Vec<&'static Dep> {
        Vec::new()
    }

    fn deps3() -> Vec<&'static Depend> {
        Vec::new()
    }

    #[test]
    fn test_with_alpm_list() {
        foo(deps().into_iter());
        foo(deps().iter().map(|d| d.as_dep()));
        foo(deps().iter());
        let _ = || foo(deps2().iter());
        let _ = || foo(deps3().iter());
    }

    #[test]
    fn test_with_alpm_list_string() {
        bar(vec!["aa", "bb", "xx"].iter());
        bar(vec!["aa", "bb", "xx"].into_iter());
        bar(vec!["aa".to_string(), "bb".to_string(), "xx".to_string()].iter());
        bar(vec!["aa".to_string(), "bb".to_string(), "xx".to_string()].into_iter());
    }
}
