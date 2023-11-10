/// Finds an installed python version on the system
///
/// This is mostly the same as the version used by [servo](https://github.com/servo/servo/blob/c78b98252a7a6de2df8628adef9515d454c9c3ac/components/style/build.rs#L21C1-L43C2)
use std::{env, process::Command, sync::LazyLock};

pub static PYTHON: LazyLock<String> = LazyLock::new(|| {
    env::var("PYTHON3").ok().unwrap_or_else(|| {
        let candidates = if cfg!(windows) {
            ["python.exe"]
        } else {
            ["python3"]
        };

        for name in candidates {
            if Command::new(name)
                .arg("--version")
                .output()
                .ok()
                .map_or(false, |out| out.status.success())
            {
                return name.to_owned();
            }
        }
        panic!(
            "Can't find python (tried {})! Try fixing PATH or setting the PYTHON3 env var",
            candidates.join(", ")
        )
    })
});
