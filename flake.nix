{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    emacs-overlay.url = "github:nix-community/emacs-overlay";

    alejandra = {
      url = "github:kamadorueda/alejandra/3.0.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, emacs-overlay, alejandra, }:
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
          overlays = [ emacs-overlay.overlays.default ];
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
          builtins.readFile
          (pkgs.runCommand "convert-scss" { buildInputs = [ pkgs.sass ]; } ''
            ${lib.getExe pkgs.sass} --style compressed --scss ${scssFile} $out
          '');

        #
        # Resources
        #

        catppuccinTheme = pkgs.fetchgit {
          url = "https://github.com/catppuccin/emacs.git";
          rev = "438622d74f1b0034c354fdd67ed39c36f059e1b4";
          sha256 = "sha256-CKxOsQHiYmURp5Fe9fOSvwcq6veDXUqKBRr8ynDOB/0=";
        };

        #
        # Configuration
        #

        emacsPackage = pkgs.emacs29-pgtk;
        envPackages = with pkgs; [
          gcc
          ripgrep

          # Required to display SVGs
          librsvg

          # Required by langtool package
          jre_minimal
          languagetool

          # Required for formatting Nix
          nixfmt
        ];
        config = rec {
          file = ./config.org;
          vars = rec {
            # Variables defined here can be accessed in the configuration file.
            # Example:
            # ```
            # (getnix "themeDir")
            # ```
            themeDir = catppuccinTheme;
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

        packages.${system}.default = (pkgs.emacsWithPackagesFromUsePackage {
          package = emacsPackage.overrideAttrs (package: {
            postInstall = (package.postInstall or "") + ''
              wrapProgram $out/bin/emacs \
                --prefix PATH : "${lib.makeBinPath envPackages}"
            '';
          });
          config = config.output;
          defaultInitFile = true;
          extraEmacsPackages = epkgs: [ epkgs.use-package ];
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
