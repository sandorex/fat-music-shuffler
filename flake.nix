{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, naersk, rust-overlay }:
    let
      inherit self;
      system = "x86_64-linux";

      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };

      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;

      naersk' = pkgs.callPackage naersk {
        cargo = toolchain;
        rustc = toolchain;
      };

      # build for musl by default
      CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";

      # make vergen_git2 happy
      VERGEN_IDEMPOTENT = "1";
      VERGEN_GIT_SHA = if (self ? "rev") then (builtins.substring 0 7 self.rev) else "nix-dirty";

      # creates properly configured qmake environment
      qt6Env = pkgs.qt6.env "qt6-${pkgs.qt6.qtbase.version}" (with pkgs; [
        # equivalent to qt6.full that was removed
        # https://github.com/NixOS/nixpkgs/commit/f3d874fb9d2240ecacdfd6e2ce75cd442243a04e
        qt6.qtwayland
        qt6.qt3d
        qt6.qt5compat
        qt6.qtcharts
        qt6.qtconnectivity
        qt6.qtdatavis3d
        qt6.qtdeclarative
        qt6.qtdoc
        qt6.qtgraphs
        qt6.qtgrpc
        qt6.qthttpserver
        qt6.qtimageformats
        qt6.qtlanguageserver
        qt6.qtlocation
        qt6.qtlottie
        qt6.qtmultimedia
        qt6.qtmqtt
        qt6.qtnetworkauth
        qt6.qtpositioning
        qt6.qtsensors
        qt6.qtserialbus
        qt6.qtserialport
        qt6.qtshadertools
        qt6.qtspeech
        qt6.qtquick3d
        qt6.qtquick3dphysics
        qt6.qtquickeffectmaker
        qt6.qtquicktimeline
        qt6.qtremoteobjects
        qt6.qtsvg
        qt6.qtscxml
        qt6.qttools
        qt6.qttranslations
        qt6.qtvirtualkeyboard
        qt6.qtwebchannel
        qt6.qtwebengine
        qt6.qtwebsockets
        qt6.qtwebview

        # NOTE has to be in here as it requires qtbase
        kdePackages.wrapQtAppsHook
      ]);
    in
    {
      # TODO package for gui and cli
      # packages.${system}.default = naersk'.buildPackage {
      #   src = ./.;

      #   inherit CARGO_BUILD_TARGET VERGEN_IDEMPOTENT VERGEN_GIT_SHA;
      # };

      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          git
          toolchain
          cmake
          stdenv.cc
          stdenv.cc.cc.lib
          pkg-config
          makeWrapper
          qt6Env
          libGL
          libGLU

          openssl # for rand in shellHook
        ];

        shellHook = ''
          # Add Qt-related environment variables.
          # https://discourse.nixos.org/t/qt-development-environment-on-a-flake-system/23707/5
          setQtEnvironment=$(mktemp)
          random=$(openssl rand -base64 20 | sed "s/[^a-zA-Z0-9]//g")
          makeShellWrapper "$(type -p sh)" "$setQtEnvironment" "''${qtWrapperArgs[@]}" --argv0 "$random"
          sed "/$random/d" -i "$setQtEnvironment"
          source "$setQtEnvironment"
        '';
      };
    };
}
