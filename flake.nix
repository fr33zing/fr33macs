{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    emacs-overlay.url = "github:nix-community/emacs-overlay";
    catppuccinEmacs = {
      url = "github:catppuccin/emacs";
      flake = false;
    };
    corfuCandidateOverlay = {
      url = "git+https://code.bsdgeek.org/adam/corfu-candidate-overlay";
      flake = false;
    };
    cargoMakedocs = {
      url = "github:Bunogi/cargo-makedocs";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, ... }@inputs:
    let
      inherit (nixpkgs) lib;

      withSystem = f:
        lib.fold lib.recursiveUpdate { } (map (s: f s) [
          "x86_64-linux"
          "x86_64-darwin"
          "aarch64-linux"
          "aarch64-darwin"
        ]);
    in withSystem (system:
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

        tangle = orgFile:
          pkgs.runCommand "tangle" { buildInputs = [ pkgs.python3Minimal ]; } ''
            python ${./util/tangle.py} ${orgFile} $out
          '';

        substitute = substitutions: elFile:
          let
            substitutionsFile =
              pkgs.writeText "substitutions" (builtins.toJSON substitutions);
          in pkgs.runCommand "substitute" {
            buildInputs = [ pkgs.python3Minimal ];
          } ''
            python ${./util/substitute.py} ${elFile} ${substitutionsFile} $out
          '';

        toCSS = scssFile:
          builtins.readFile (pkgs.runCommand "convert-scss" {
            buildInputs = [ pkgs.dart-sass ];
          } ''
            ${lib.getExe pkgs.dart-sass} --style=compressed ${scssFile} $out
          '');

        #
        # Emacs Packages
        #

        customEmacsPackages = epkgs:
          [
            (epkgs.trivialBuild {
              pname = "corfu-candidate-overlay";
              version = "1.5";
              src = inputs.corfuCandidateOverlay;
              packageRequires = [ epkgs.corfu ];
            })
          ];

        #
        # Configuration
        #

        emacsPackage = pkgs.emacs29-pgtk;
        envPackages = with pkgs; [
          gcc

          # SVGs
          librsvg

          # Shell
          shellcheck

          # Nix
          nixfmt

          # Python
          (python3.withPackages (p:
            with p; [
              python-lsp-server
              python-lsp-ruff # Linting
              pylsp-rope # Completions and refactoring
              pylsp-mypy # Type checking
              pyls-memestra # Deprecation tracking
              black # Formatting
            ]))

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
            fontFamily = "Iosevka Comfy";
            tagsFontFamily = "Iosevka";
            fontSize = 12;
            font = "${fontFamily}-${toString fontSize}";
            exportCSS = toCSS ./export.scss;
          };
          output = substitute vars (tangle file);
        };
      in {

        #
        # Outputs
        #

        nixConfig = {
          extra-substituters = [ "https://nix-community.cachix.org" ];
          extra-trusted-public-keys = [
            "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
          ];
        };

        packages.${system}.default = (pkgs.emacsWithPackagesFromUsePackage {
          package = emacsPackage.overrideAttrs (package: {
            postInstall = (package.postInstall or "") + ''
              wrapProgram $out/bin/emacs \
                --prefix PATH : "${lib.makeBinPath envPackages}"
            '';
          });
          config = config.output;
          defaultInitFile = true;
          extraEmacsPackages = epkgs:
            [ epkgs.use-package ] ++ (customEmacsPackages epkgs);
        }).overrideAttrs (_: { meta.mainProgram = "emacs"; });

        devShells.${system}.default = pkgs.mkShell {
          packages = [ self.packages.${system}.default ];
          shellHook = ''
            printf '\n%s\n%s\n' \
              'emacs =>      ${self.packages.${system}.default}' \
              'config.org => ${config.output}'
          '';
        };
      });
}
