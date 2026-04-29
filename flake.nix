{
  description = "Rust graphics dev shell (minifb / pixels / winit)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        # --- Rust toolchain ---
        rustc
        cargo

        # --- X11 stack (required for minifb, fallback for winit) ---
        xorg.libX11
        xorg.libXcursor
        xorg.libXrandr
        xorg.libXi
        xorg.libXext

        # --- OpenGL (pixels, wgpu backends sometimes) ---
        libGL
        mesa

        # --- Wayland (only needed if winit uses it) ---
        wayland
        wayland-protocols
        libxkbcommon

        # --- misc runtime helpers ---
        pkg-config
      ];

      shellHook = ''
        # Force X11 backend for libraries that support both
        export WINIT_UNIX_BACKEND=x11

        # Ensure runtime loader can find libs (NixOS-safe workaround)
        export LD_LIBRARY_PATH=${
          pkgs.lib.makeLibraryPath [
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXrandr
            pkgs.xorg.libXi
            pkgs.xorg.libXext
            pkgs.libGL
            pkgs.mesa
            pkgs.wayland
            pkgs.libxkbcommon
          ]
        }
      '';
    };
  };
}
