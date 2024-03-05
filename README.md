#  alpm.rs - rust bindings for libalpm

[![Latest version](https://img.shields.io/crates/v/alpm.svg)](https://crates.io/crates/alpm)
[![Documentation](https://docs.rs/alpm/badge.svg)](https://docs.rs/alpm)

alpm.rs provides complete, safe, ergonomic bindings to the libalpm API,
the package management library used by pacman and other tools.

# Features

- mtree - enables the alpm_pkg_mtree_* functions
- generate - generate the raw alpm-sys bindings at build time
- checkver - check that the version of libalpm installed is compatible with alpm.rs
- git - target the git master API
- static - statically link to libalpm


**Note:** checkver does not work with the git feature. You can instead use
the generate feature to ensure alpm.rs builds against a compatible libalpm version.

# libalpm compatibility

alpm.rs always targets the latest version of libalpm. It may also support
previous versions if the API was not changed.

alpm.rs also supports the pacman git master via the git feature.

Currently alpm.rs supports libalpm v14.x.x.

**Note:** When using the git feature, alpm.rs is updated against the libalpm git master
as commits happen. As the git version is not considered stable software, this is done
without bumping the major version.

# Documentation

This crate just provides bindings for libalpm and hence does not document libalpm.
You can find documentation for libalpm in the [libalpm (3)](https://man.archlinux.org/man/core/pacman/libalpm.3.en) man page or in [alpm.h](https://gitlab.archlinux.org/pacman/pacman/-/blob/master/lib/libalpm/alpm.h).

There are also examples on how to use the alpm crate in [alpm/examples](alpm/examples).

# alpm-sys

This repo also contains the alpm-sys crate, providing raw bindings for libalpm.
Although you probably just want to use the alpm crate instead.
