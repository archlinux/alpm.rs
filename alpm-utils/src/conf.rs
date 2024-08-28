use alpm::{Alpm, SigLevel, Usage};

/// Re-export of pacmanconf.
pub mod config {
    pub use pacmanconf::*;
}

use pacmanconf::Config;

/// Initiates and configures Alpm using a pacman config.
///
/// ```no_run
/// use pacmanconf::Config;
/// use alpm_utils::alpm_with_conf;
///
/// # fn main() {
/// let conf = Config::new().unwrap();
/// let alpm = alpm_with_conf(&conf).unwrap();
/// # }
/// ```
pub fn alpm_with_conf(conf: &Config) -> alpm::Result<Alpm> {
    let mut alpm = Alpm::new(&*conf.root_dir, &*conf.db_path)?;
    configure_alpm(&mut alpm, conf)?;
    Ok(alpm)
}

/// Configures an exsting Alpm handle  using a pacman config.
///
/// You probably just want to use alpm_with_conf unless you need to do something before the
/// repos are registered such as setting the db ext.
///
/// ```no_run
/// use pacmanconf::Config;
/// use alpm_utils::configure_alpm;
/// use alpm::Alpm;
///
/// # fn main() {
/// let conf = Config::new().unwrap();
/// let mut alpm = Alpm::new(&*conf.root_dir, &*conf.db_path).unwrap();
/// let alpm = configure_alpm(&mut alpm, &conf).unwrap();
/// # }
/// ```
pub fn configure_alpm(alpm: &mut Alpm, conf: &Config) -> alpm::Result<()> {
    alpm.set_cachedirs(conf.cache_dir.iter())?;
    alpm.set_hookdirs(conf.hook_dir.iter())?;
    alpm.set_gpgdir(&*conf.gpg_dir)?;
    alpm.set_logfile(&*conf.log_file)?;
    alpm.set_ignorepkgs(conf.ignore_pkg.iter())?;
    alpm.set_ignorepkgs(conf.ignore_pkg.iter())?;
    alpm.set_architectures(conf.architecture.iter())?;
    alpm.set_noupgrades(conf.no_upgrade.iter())?;
    alpm.set_noextracts(conf.no_extract.iter())?;
    alpm.set_default_siglevel(parse_sig_level(&conf.sig_level))?;
    alpm.set_local_file_siglevel(parse_sig_level(&conf.local_file_sig_level))?;
    alpm.set_remote_file_siglevel(parse_sig_level(&conf.remote_file_sig_level))?;
    alpm.set_use_syslog(conf.use_syslog);
    alpm.set_check_space(conf.check_space);
    alpm.set_disable_dl_timeout(conf.disable_download_timeout);
    alpm.set_parallel_downloads(conf.parallel_downloads as u32);
    alpm.set_disable_sandbox(conf.disable_sandbox);
    alpm.set_sandbox_user(conf.download_user.clone())?;

    for repo in &conf.repos {
        register_db(alpm, repo)?;
    }

    Ok(())
}

fn parse_sig_level(levels: &[String]) -> SigLevel {
    let mut sig = SigLevel::NONE;

    for level in levels {
        match level.as_str() {
            "PackageNever" => sig.remove(SigLevel::PACKAGE),
            "PackageOptional" => sig.insert(SigLevel::PACKAGE | SigLevel::PACKAGE_OPTIONAL),
            "PackageRequired" => {
                sig.insert(SigLevel::PACKAGE);
                sig.remove(SigLevel::PACKAGE_OPTIONAL);
            }
            "PackageTrustOnly" => {
                sig.remove(SigLevel::PACKAGE_MARGINAL_OK | SigLevel::PACKAGE_UNKNOWN_OK)
            }
            "PackageTrustAll" => {
                sig.insert(SigLevel::PACKAGE_MARGINAL_OK | SigLevel::PACKAGE_UNKNOWN_OK)
            }
            "DatabaseNever" => sig.remove(SigLevel::DATABASE),
            "DatabaseOptional" => sig.insert(SigLevel::DATABASE | SigLevel::DATABASE_OPTIONAL),
            "DatabaseRequired" => {
                sig.insert(SigLevel::DATABASE);
                sig.remove(SigLevel::DATABASE_OPTIONAL);
            }
            "DatabaseTrustOnly" => {
                sig.remove(SigLevel::DATABASE_MARGINAL_OK | SigLevel::DATABASE_UNKNOWN_OK)
            }
            "DatabaseTrustAll" => {
                sig.insert(SigLevel::DATABASE_MARGINAL_OK | SigLevel::DATABASE_UNKNOWN_OK)
            }
            _ => {}
        }
    }

    sig
}

fn register_db(alpm: &mut alpm::Alpm, repo: &pacmanconf::Repository) -> alpm::Result<()> {
    let sig = if repo.sig_level.is_empty() {
        SigLevel::USE_DEFAULT
    } else {
        parse_sig_level(&repo.sig_level)
    };

    let db = alpm.register_syncdb_mut(&*repo.name, sig)?;
    db.set_servers(repo.servers.iter())?;

    let mut usage = Usage::NONE;

    for v in &repo.usage {
        match v.as_str() {
            "Sync" => usage |= Usage::SYNC,
            "Search" => usage |= Usage::SEARCH,
            "Install" => usage |= Usage::INSTALL,
            "Upgrade" => usage |= Usage::UPGRADE,
            _ => {}
        }

        if usage == Usage::NONE {
            usage = Usage::ALL
        }
    }

    db.set_usage(usage)?;
    Ok(())
}
