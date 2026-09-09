{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    emacs-overlay.url = "github:nix-community/emacs-overlay";
    catppuccinEmacs = {
      url = "github:catppuccin/emacs";
      flake = false;
    };
    cargoMakedocs = {
      url = "github:Bunogi/cargo-makedocs";
      flake = false;
    };
  };

  outputs =
    { self, nixpkgs, ... }@inputs:
    let
      inherit (nixpkgs) lib;

      withSystem =
        f:
        lib.fold lib.recursiveUpdate { } (
          map (s: f s) [
            "x86_64-linux"
            "x86_64-darwin"
            "aarch64-linux"
            "aarch64-darwin"
          ]
        );
    in
    withSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ inputs.emacs-overlay.overlays.default ];
        };
        inherit (pkgs) lib stdenv;

        #
        # Utility functions
        #

        symbol = s: "'" + s;

        tangle =
          orgFile:
          pkgs.runCommand "tangle" { buildInputs = [ pkgs.python3Minimal ]; } ''
            python ${./util/tangle.py} ${orgFile} $out
          '';

        substitute =
          substitutions: elFile:
          let
            substitutionsFile = pkgs.writeText "substitutions" (builtins.toJSON substitutions);
          in
          pkgs.runCommand "substitute"
            {
              buildInputs = [ pkgs.python3Minimal ];
            }
            ''
              python ${./util/substitute.py} ${elFile} ${substitutionsFile} $out
            '';

        toCSS =
          scssFile:
          builtins.readFile (
            pkgs.runCommand "convert-scss"
              {
                buildInputs = [ pkgs.dart-sass ];
              }
              ''
                ${lib.getExe pkgs.dart-sass} --style=compressed ${scssFile} $out
              ''
          );

        #
        # Emacs Packages
        #

        customEmacsPackages = epkgs: [
          # Example
          # (epkgs.trivialBuild {
          #   pname = "corfu-candidate-overlay";
          #   version = "1.5";
          #   src = inputs.corfuCandidateOverlay;
          #   packageRequires = [ epkgs.corfu ];
          # })
        ];

        #
        # Configuration
        #

        emacsPackage = pkgs.emacs-pgtk;
        envPackages = with pkgs; [
          # Common
          pkg-config
          gcc
          gdb

          # SVGs
          librsvg

          # Shell
          shellcheck

          # Nix
          nixfmt

          # Python
          (python3.withPackages (
            p: with p; [
              python-lsp-server
              python-lsp-ruff # Linting
              pylsp-rope # Completions and refactoring
              pylsp-mypy # Type checking
              pyls-memestra # Deprecation tracking
              black # Formatting
            ]
          ))

          # Rust
          rustc # Compiler
          rust-analyzer # LSP
          rustfmt # Formatting
          clippy # Linting
          cargo # Project management
          cargo-edit # Dependency management
          cargo-expand # Macro expansion
          pandoc # Documentation
          inputs.cargoMakedocs
          ripgrep
          fd

          # Web
          nodejs
          nodePackages.prettier
          nodePackages.typescript
          nodePackages.typescript-language-server

          # Java
          jdk17
        ];

        config = rec {
          file = ./config.org;
          vars = rec {
            # Variables defined here can be accessed in the configuration file.
            # Example:
            #
            # (getnix "themeDir")
            themeDir = inputs.catppuccinEmacs;
            themeVariant = symbol "macchiato";
            fontFamily = "Aporetic Sans Mono";
            tagsFontFamily = "Aporetic Sans Mono";
            fontSize = 12;
            font = "${fontFamily}-${toString fontSize}";
            exportCSS = toCSS ./export.scss;
          };
          output = substitute vars (tangle file);
        };
      in
      {

        #
        # Outputs
        #

        nixConfig = {
          extra-substituters = [ "https://nix-community.cachix.org" ];
          extra-trusted-public-keys = [
            "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
          ];
        };

        packages.${system} = {
          default =
            (pkgs.emacsWithPackagesFromUsePackage {
              package = emacsPackage.overrideAttrs (package: {
                postInstall = (package.postInstall or "") + ''
                  wrapProgram $out/bin/emacs \
                    --prefix PATH : "${lib.makeBinPath envPackages}" \
                    --set RUST_SRC_PATH ${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}
                '';
              });
              config = config.output;
              defaultInitFile = true;
              extraEmacsPackages = epkgs: [ epkgs.use-package ] ++ (customEmacsPackages epkgs);
            }).overrideAttrs
              (_: {
                meta.mainProgram = "emacs";
              });

          installFiles =
            let
              emacs = self.packages.${system}.default;
              icon = stdenv.mkDerivation {
                name = "icon.png";
                src = lib.fileset.toSource {
                  root = ./.;
                  fileset = ./install/icon.png;
                };
                postInstall = "cp $src/**/icon.png $out";
              };

              replaceFile =
                prefix: file:
                pkgs.replaceVarsWith {
                  src = file;
                  name = "${prefix}${builtins.baseNameOf file}";
                  replacements = {
                    inherit icon;
                    bin = "${emacs}/bin";
                  };
                };

              files = [
                (replaceFile "" ./install/install-emacs)
                (replaceFile "fr33macs." ./install/emacs.desktop)
                (replaceFile "fr33macs." ./install/emacsclient.desktop)
                (replaceFile "fr33macs." ./install/emacs.service)
              ];
            in
            pkgs.runCommand "emacs-install-files" { } ''
              mkdir -p $out
              for src in ${lib.concatStringsSep " " files}; do
                src_bn="$(basename $src)"
                out_bn="''${src_bn#*-}"
                cp "$src" "$out/$out_bn"
              done
              chmod +x "$out/install-emacs"
            '';
        };

        devShells.${system}.default =
          let
            emacs = self.packages.${system}.default;
            installFiles = self.packages.${system}.installFiles;
          in
          pkgs.mkShell {
            packages = [
              emacs
              installFiles
            ];
            shellHook = ''
              export PATH="${installFiles}:$PATH"
              printf '\n%s\n%s\n%s\n\n%s\n\n' \
                'emacs      => ${emacs}' \
                'config.org => ${config.output}' \
                'install    => ${installFiles}' \
                'Use command `install-emacs` to install on non-NixOS systems.' 
            '';
            installPhase = "install-emacs";
          };

        nixosModules.${system}.default = {
          services.emacs = {
            enable = true;
            install = true;
            defaultEditor = true;
            package = self.packages.${system}.default;
          };
        };
      }
    );
}
