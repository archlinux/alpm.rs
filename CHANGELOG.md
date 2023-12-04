# Changelog

## alpm v2.2.3 alpm-utils v1.1.3 alpm-sys v2.1.3 (2021-12-04)

### Fixed

- Fix check_signature being on wrong type
- Fix Target having broken constructor
- Update alpm-sys bindings

## alpm v2.2.2 (2023-05-07)

### Fixed

- Fix segfault when calling groups

## alpm-utils v1.1.2 (2022-02-16)

### Breaking

- Update pacmanconf to v2.0.0

## alpm v2.2.1 (2021-12-12)

### Fixed

- Revert breaking change that broke existing user code

## alpm v2.2.0 (2021-12-09)

### Fixed

- Fix LoadedPackage::pkg() being unsound
- Use NonNull where possible
- Fix typos
- Fix AlpmList<Package> Not working for AlpmList<Pkg>

### Added

- Add Alpm::release() to catch release errors
- Add docs for top level crate

## alpm v2.1.3 alpm-utils v1.1.2 (2021-10-11)

### Fixed

- Fix wrong alpm-sys dep

## alpm v2.1.2 alpm-utils v1.1.1 alpm-sys v2.1.2 (2021-10-11)

### Added

- Add repository doc alias for db
- Add examples

## alpm v2.1.1 alpm-sys v2.1.1 (2021-09-05)

### Added

- Add doc alias for Alpm::new()

### Pacman-git

- Bump git support to pacman-git@39c3cbdf

