use std::{
    fs::{OpenOptions, create_dir_all},
    io::{self, Write},
    path::Path,
};

#[cfg(unix)]
use std::{
    fs::Permissions,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
};

/// Builder struct to specify how to write a file.
///
/// * [`path`](Self::path): Target path to which the file will be written.
/// * [`content`](Self::content): File contents to be written.
/// * [`force`](Self::force): Overwrite if the file already exists.
/// * [`make_parents`](Self::make_parents): Create the parent directory tree.
/// * [`atomic`](Self::atomic): Write the file atomically using the temporary file + rename pattern.
///   Requires the `tempfile` feature.
/// * [`permissions`](Self::permissions): File permissions. Only supported on Unix-like platforms.
#[derive(Debug)]
pub struct WriteParams<'a, 'b> {
    path: &'a Path,
    content: &'b [u8],
    force: bool,
    make_parents: bool,

    #[cfg(feature = "tempfile")]
    atomic: bool,

    #[cfg(unix)]
    permissions: Option<Permissions>,
}

impl<'a, 'b> WriteParams<'a, 'b> {
    /// Create a new [`WriteParams`] instance.
    ///
    /// Initially, all flags are `false`/`None`. Use the builder methods to set those.
    #[must_use]
    pub fn new<P, C>(path: &'a P, content: &'b C) -> Self
    where
        P: AsRef<Path> + ?Sized,
        C: AsRef<[u8]> + ?Sized,
    {
        Self {
            path: path.as_ref(),
            content: content.as_ref(),
            force: false,
            make_parents: false,

            #[cfg(feature = "tempfile")]
            atomic: false,

            #[cfg(unix)]
            permissions: None,
        }
    }

    /// Write the file with the supplied parameters.
    ///
    /// # Errors
    /// Returns [`io::Error`] if any steps fail.
    pub fn write(self) -> io::Result<()> {
        let parent = self.path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("path '{}' has no parent directory", self.path.display()),
            )
        })?;

        // Prevents some filesystem errors on relative paths, since `Path::new("foo.txt").parent()`
        // returns `Some(Path::new(""))`.
        let parent = if parent.is_empty() {
            Path::new(".")
        } else {
            parent
        };

        if self.make_parents {
            create_dir_all(parent)?;
        }

        #[cfg(feature = "tempfile")]
        if self.atomic {
            return self.atomic_write(parent);
        }

        self.nonatomic_write()
    }

    #[cfg(feature = "tempfile")]
    fn atomic_write(self, parent: &Path) -> io::Result<()> {
        #[allow(unused_mut)]
        let mut builder = tempfile::Builder::new();

        #[cfg(unix)]
        if let Some(permissions) = self.permissions {
            builder.permissions(permissions);
        }

        let mut file = builder.tempfile_in(parent)?;
        file.write_all(self.content)?;
        file.as_file().sync_all()?;

        if self.force {
            file.persist(self.path)?;
        } else {
            file.persist_noclobber(self.path)?;
        }

        Ok(())
    }

    fn nonatomic_write(self) -> io::Result<()> {
        let mut file = OpenOptions::new();
        file.truncate(true)
            .write(true)
            .create(true)
            .create_new(!self.force);

        #[cfg(unix)]
        {
            if let Some(permissions) = &self.permissions {
                file.mode(permissions.mode());
            }
        }

        let mut file = file.open(self.path)?; // Ignores `create` and `truncate` if set

        file.write_all(self.content)?;
        file.sync_all()
    }

    /// Set the output path.
    #[must_use]
    pub fn path<P: AsRef<Path> + ?Sized>(self, path: &'a P) -> Self {
        Self {
            path: path.as_ref(),
            ..self
        }
    }

    /// Set the file contents to write.
    #[must_use]
    pub fn content<C: AsRef<[u8]> + ?Sized>(self, content: &'b C) -> Self {
        Self {
            content: content.as_ref(),
            ..self
        }
    }

    /// If true, if the target path already exists, clobber it. If false, do not clobber existing
    /// files.
    #[must_use]
    pub fn force(self, force: bool) -> Self {
        Self { force, ..self }
    }

    /// If true, attempt to create the parent directory hierarchy if it does not exist.
    #[must_use]
    pub fn make_parents(self, make_parents: bool) -> Self {
        Self {
            make_parents,
            ..self
        }
    }

    /// If true, first write the file's [`contents`](Self::content) to a temporary file, then
    /// persist it.
    ///
    /// Requires the `tempfile` feature.
    #[cfg(feature = "tempfile")]
    #[must_use]
    pub fn atomic(self, atomic: bool) -> Self {
        Self { atomic, ..self }
    }

    /// Set the file's permissions. These will be set at file creation, so there is no race
    /// condition window.
    ///
    /// Because file permissions work so differently on non-Unix platforms, this is only supported
    /// on Unix-like targets.
    #[cfg(unix)]
    #[must_use]
    pub fn permissions(self, permissions: Option<Permissions>) -> Self {
        Self {
            permissions,
            ..self
        }
    }
}
