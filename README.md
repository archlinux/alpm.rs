#  alpm.rs - rust bindings for libalpm

[![Latest version](https://img.shields.io/crates/v/alpm.svg)](https://crates.io/crates/alpm)
[![Documentation](https://docs.rs/alpm/badge.svg)](https://docs.rs/alpm)

alpm.rs provides complete, safe, erganomic bindings to the libalpm API,
the package management library used by pacman and other tools.

# Features

- mtree - enables the alpm_pkg_mtree_* funtions
- generate - generate the raw alpm-sys bindings at build time
- checkver - check that the version of libalpm installed is compatible with alpm.rs
- git - target the git master API


**Note:** checkver does not work with the git feature. You can instead use
the generate feature to ensure alpm.rs builds against a compatible libalpm version.

# libalpm compatibility

alpm.rs always targets the latest version of libalpm. It may also support
previous versions if the API was not changed.

alpm.rs also supports the pacman git master via the git feature.

Currently alpm.rs supports libalpm v12.x.0 to v12.x.2.

**Note:** When using the git feature, alpm.rs is updated against the libalpm git master
as commits happen. As the git version is not considered stable software, this is done
without bumping the major version.

# alpm-sys

This repo also contains the alpm-sys crate, providing raw bindings for libalpm.
Although you probably just want to use the alpm crate instead.
