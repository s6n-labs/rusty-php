mod subprocess;
mod target;

use std::env::var;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

use anyhow::bail;

use crate::target::{find_target_dir, TargetDir};

pub struct Version {
    major: isize,
    minor: isize,
    patch: isize,
}

impl Version {
    pub fn new(major: isize, minor: isize, patch: isize) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    fn tag(&self) -> String {
        format!("php-{}", self.to_string())
    }
}

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s
            .split('.')
            .map(isize::from_str)
            .collect::<Result<Vec<_>, _>>()?;

        if parts.len() != 3 {
            bail!("Invalid version spec");
        }

        Ok(Self::new(parts[0], parts[1], parts[2]))
    }
}

impl TryFrom<String> for Version {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl ToString for Version {
    fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub struct Builder {
    version: Version,
    arguments: Vec<String>,
}

impl Builder {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_version<V, E>(mut self, version: V) -> Result<Self, E>
    where
        V: TryInto<Version, Error = E>,
    {
        self.version = version.try_into()?;
        Ok(self)
    }

    #[inline]
    pub fn without_default_arguments(mut self) -> Self {
        self.arguments = vec![];
        self
    }

    #[inline]
    pub fn with_argument<A>(mut self, argument: A) -> Self
    where
        A: Into<String>,
    {
        self.arguments.push(argument.into());
        self
    }

    #[inline]
    pub fn with_arguments<I, A>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = A>,
        A: Into<String>,
    {
        arguments.into_iter().for_each(|a| {
            self.arguments.push(a.into());
        });
        self
    }

    pub fn build(self) -> anyhow::Result<()> {
        println!("cargo:rerun-if-changed=build.rs");

        let target_dir = match find_target_dir(Path::new(&var("OUT_DIR")?)) {
            TargetDir::Path(d) => d,
            _ => bail!("Could not find the target directory"),
        };

        let php_src_dir = target_dir.join(format!("php-src-{}", self.version.to_string()));
        let cache_key_path = target_dir.join(".rusty-php-build-cache-key");

        if !php_src_dir.exists() {
            subprocess::run(
                Command::new("git")
                    .args([
                        "clone",
                        "--single-branch",
                        "-b",
                        self.version.tag().as_str(),
                        "https://github.com/php/php-src.git",
                        php_src_dir.to_str().unwrap(),
                    ])
                    .current_dir(target_dir),
            )?;
        }

        if std::fs::read_to_string(&cache_key_path).ok() != Some(self.cache_key()) {
            subprocess::run(
                Command::new("./buildconf")
                    .arg("--force")
                    .current_dir(&php_src_dir),
            )?;

            subprocess::run(
                Command::new("./configure")
                    .args(["--disable-all", "--disable-cgi", "--disable-cli"])
                    .current_dir(&php_src_dir),
            )?;

            std::fs::write(&cache_key_path, self.cache_key())?;
        }

        let makefile = std::fs::read_to_string(php_src_dir.join("Makefile"))?;
        let extra_libs = makefile
            .lines()
            .find_map(|l| l.strip_prefix("EXTRA_LIBS = "))
            .map(|l| l.split(' ').filter_map(|l| l.strip_prefix("-l")).collect())
            .unwrap_or(vec![]);

        subprocess::run(
            Command::new("make")
                .args(["-j6", "libphp.la"])
                .current_dir(&php_src_dir),
        )?;

        println!(
            "cargo:rustc-link-search=native={}",
            php_src_dir.join(".libs").to_string_lossy(),
        );
        println!("cargo:rustc-link-lib=static=php");

        extra_libs.iter().for_each(|l| {
            println!("cargo:rustc-link-lib=dylib={}", l);
        });

        Ok(())
    }

    fn cache_key(&self) -> String {
        format!(
            "version={};arguments={}",
            self.version.to_string(),
            self.arguments.join(","),
        )
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            version: Version::new(8, 2, 4),
            arguments: vec!["--disable-all".to_string(), "--disable-cgi".to_string()],
        }
    }
}

pub fn builder() -> Builder {
    Builder::new()
}
