use alpm::{Alpm, AlpmListMut, Depend, SigLevel};

fn main() {
    // The List type is a wraper around alpm_list_t. This is a doubly linked list that alpm uses
    // for most of its list needs.
    //
    // These bindings define two list types AlpmList and AlpmListMut. These can be thought of
    // similarly to a &[] and Vec<T>. Where one is borrowed and immutable while the other is owned
    // by you and can be mutated.

    let handle = Alpm::new("/", "tests/db").unwrap();

    let core = handle
        .register_syncdb("core", SigLevel::USE_DEFAULT)
        .unwrap();

    // this returns an AlpmList<Package>.
    let pkgs = core.pkgs();

    // we can iterate
    pkgs.iter().map(|pkg| pkg.isize()).sum::<i64>();

    // we can clone the list
    // but as this is akin to &[] it just clones the reference and the list still points to the
    // same underlying data
    let _clone = pkgs.clone();

    // if we want a list we can actually mutate we need to call .to_list_mut()
    // this is akin to .to_vec()
    let mut pkgs = pkgs.to_list_mut();

    // we can now mutate the list
    pkgs.remove(10).unwrap();

    // or add to the list
    let linux = core.pkg("linux").unwrap();
    pkgs.push(linux);

    // or filter
    pkgs.retain(|pkg| pkg.name().starts_with("a"));

    println!("{:#?}", pkgs);

    // You can also create lists from scratch.
    //
    // However creating lists is usually not necessary as any function that accepts a list will
    // also accept an iterator intead. Though that function will just build the iterator into an
    // alpm_list anyway. So if you already have a list it's more effiecient to use that.
    let mut list = AlpmListMut::new();

    // and push things
    // but only certain supported types
    list.push(Depend::new("foo=1"));

    // extend
    list.extend(linux.depends().iter().map(|d| d.to_depend()));

    println!("{:#?}", list);
}
