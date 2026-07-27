use std::{
    env,
    path::{Path, PathBuf},
};

pub const SHADERS: &[(&str, &str)] = &[
    ("CRT Aperture", "crt/crt-aperture.slangp"),
    ("CRT Easymode", "crt/crt-easymode.slangp"),
    ("CRT Glow (Gaussian)", "crt/crtglow_gauss.slangp"),
    ("CRT Glow (Lanczos)", "crt/crtglow_lanczos.slangp"),
    ("CRT Hyllian", "crt/crt-hyllian.slangp"),
    ("CRT Royale", "crt/crt-royale.slangp"),
    ("CRT Sim", "crt/crtsim.slangp"),
];

pub fn find_shader_root() -> Option<PathBuf> {
    if let Some(path) = env::var_os("NTSCRT_SHADER_DIR").map(PathBuf::from) {
        if is_shader_root(&path) {
            return Some(path);
        }
    }

    let mut candidates = Vec::new();
    if let Ok(executable) = env::current_exe() {
        if let Some(directory) = executable.parent() {
            candidates.push(directory.join("slang-shaders"));
            candidates.push(directory.join("../share/ntscrt/slang-shaders"));
            candidates.push(directory.join("../Resources/slang-shaders"));
        }
    }
    if let Ok(current) = env::current_dir() {
        candidates.push(current.join("slang-shaders"));
        candidates.push(current.join("Vendor/slang-shaders"));
        candidates.push(current.join("../Vendor/slang-shaders"));
    }

    candidates.into_iter().find(|path| is_shader_root(path))
}

fn is_shader_root(path: &Path) -> bool {
    path.join("crt/crt-easymode.slangp").is_file() && path.join("include").is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_presets_and_royale_dependencies_exist() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../Vendor/slang-shaders");

        for (name, relative_path) in SHADERS {
            assert!(
                root.join(relative_path).is_file(),
                "bundled preset {name} is missing at {relative_path}"
            );
        }

        for relative_path in [
            "blurs/shaders/royale/blur9fast-vertical.slang",
            "blurs/shaders/royale/blur9fast-horizontal.slang",
        ] {
            assert!(
                root.join(relative_path).is_file(),
                "CRT Royale dependency is missing at {relative_path}"
            );
        }
    }
}
