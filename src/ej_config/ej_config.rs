use serde::{Deserialize, Serialize};

use super::ej_board::EjBoard;

#[derive(Debug, Serialize, Deserialize)]
pub struct EjGlobalConfig {
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EjConfig {
    pub global: EjGlobalConfig,
    pub boards: Vec<EjBoard>,
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::prelude::*;

    #[test]
    pub fn deserialize() -> Result<()> {
        let content = r#"
            # Build Configuration File
            # This file defines boards and their configurations for building and running jobs
            
            # Global settings
            [global]
            version = "1.0.0"
            
            
            # Boards
            [[boards]]
            name = "Raspberry Pi 3"
            description = "Raspberry Pi 3 Model B+"
            
            [[boards.configs]]
            board = "rpi3"
            description = "Rpi3 Wayland"
            tags = ["wayland", "arm64"]
            build_script = "/home/work/wayland-app/scripts/build_rpi4_wayland.sh"
            run_script = "/home/work/wayland-app/scripts/run_rpi4_wayland.sh"
            results_path = "/home/work/wayland-app/results/results.json"
            library_path = "/home/work/wayland-app/lib"
            
            [[boards.configs]]
            board = "rpi3"
            description = "Rpi3 SDL"
            tags = ["sdl2", "arm64"]
            build_script = "/home/work/wayland-app/scripts/build_rpi4_wayland.sh"
            run_script = "/home/work/wayland-app/scripts/run_rpi4_wayland.sh"
            results_path = "/home/work/wayland-app/results/results.json"
            library_path = "/home/work/wayland-app/lib"
            
            [[boards]]
            name = "x86 PC running Fedora 41"
            description = "AMD Ryzen 5 3600 desktop with NVIDIA GTX 1650"
            
            [[boards.configs]]
            board = "x86_desktop"
            description = "Wayland build for desktop"
            tags = ["wayland", "x86_64"]
            build_script = "scripts/build_desktop_wayland.sh"
            run_script = "scripts/run_desktop_wayland.sh"
            results_path = "/var/log/tests/desktop_wayland_results.json"
            library_path = "https://github.com/yourusername/lib-desktop-wayland.git"
            
            [[boards.configs]]
            board = "x86_desktop"
            description = "X11 build for desktop"
            tags = ["x11", "x86_64"]
            build_script = "scripts/build_desktop_x11.sh"
            run_script = "scripts/run_desktop_x11.sh"
            results_path = "/var/log/tests/desktop_x11_results.json"
            library_path = "https://github.com/yourusername/lib-desktop-x11.git"
        "#;
        toml::from_str::<EjConfig>(content)?;
        Ok(())
    }
}
