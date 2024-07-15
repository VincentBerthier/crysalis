{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    advisory-db,
  }:
    flake-utils.lib.eachSystem ["x86_64-linux"] (system: let
      pkgs = import nixpkgs {
        inherit system;
        # config.allowUnfree = true;
        overlays = [(import rust-overlay)];
      };
      rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      rustStable = pkgs.rust-bin.stable.latest.default;

      inherit (pkgs) lib;
      craneLib = (crane.mkLib pkgs).overrideToolchain rust;
      craneLibStable = (crane.mkLib pkgs).overrideToolchain rustStable;

      src = lib.cleanSourceWith {
        src = craneLib.path ./.;
        filter = path: type:
          (lib.hasSuffix "\.dic" path)
          || (lib.hasSuffix "\.json" path)
          || (craneLib.filterCargoSources path type);
      };

      commonArgs = {
        inherit src;
        pname = "crysalis";
        version = "0.1.0";
        strictDeps = true;
        doCheck = false;

        buildInputs = with pkgs;
          [
            # Add additional build inputs here
          ]
          ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];

        nativeBuildInputs = with pkgs; [mold];
      };

      # Build *just* the cargo dependencies, so we can reuse
      # all of that work (e.g. via cachix) when running in CI
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      ######################################################
      ###               Coverage & Doc                   ###
      ######################################################
      crysalis-docs = craneLib.cargoDoc (commonArgs
        // {
          pname = "crysalis-docs";
          inherit cargoArtifacts;
        });

      llvm-cov-pretty = craneLibStable.buildPackage (commonArgs
        // {
          pname = "llvm-cov-pretty";
          version = "0.1.9";
          cargoArtifacts = null;

          src = pkgs.fetchFromGitHub {
            owner = "dnaka91";
            repo = "llvm-cov-pretty";
            rev = "v0.1.9";
            sha256 = "sha256-ASu+BJTpoIXHI98j02FqbfqM6EP+sJCti+s0mrZ6VJs=";
            fetchSubmodules = true;
          };

          cargoBuildCommand = "pnpm run build && cargo build --profile release";
          doCheck = true;
          cargoTestExtraArgs = "-- --skip version";
          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [pkgs.tailwindcss];
        });

      crysalis-coverage = craneLib.mkCargoDerivation (commonArgs
        // {
          inherit src;

          pname = "bangk-coverage";
          BANGK_MODE = "TESTING";
          # CARGO_BUILD_JOBS = 8;
          cargoArtifacts = null;

          buildPhaseCargoCommand = ''
            cargo llvm-cov nextest --ignore-filename-regex="^\\\/nix\\\/store\\\/*" --locked --all-features --json --output-path coverage.json
          '';
          doInstallCargoArtifacts = false;
          installPhase = ''
            mkdir -p $out
            ${llvm-cov-pretty}/bin/llvm-cov-pretty --theme dracula --output-dir $out coverage.json
            cp coverage.json $out/
          '';
          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [pkgs.cargo-llvm-cov pkgs.cargo-nextest];
        });

      ######################################################
      ###                  Binaries                      ###
      ######################################################
      # Build the actual crate itself, reusing the dependency
      # artifacts from above.
      crysalis-bin = craneLib.buildPackage (commonArgs
        // {
          inherit src cargoArtifacts;
          pname = "crysalis";
          doCheck = false;
        });
    in {
      checks = {
        # Build the crate as part of `nix flake check` for convenience
        inherit crysalis-bin;

        ######################################################
        ###               Nix flake checks                 ###
        ######################################################
        # Run clippy (and deny all warnings) on the crate source,
        # again, resuing the dependency artifacts from above.
        #
        # Note that this is done as a separate derivation so that
        # we can block the CI if there are issues here, but not
        # prevent downstream consumers from building our crate by itself.
        crysalis-clippy = craneLib.cargoClippy (commonArgs
          // {
            inherit src cargoArtifacts;
            pname = "crysalis-clippy";

            cargoClippyExtraArgs = "--all-features --all-targets -- --deny warnings";
          });

        # Check formatting
        crysalis-fmt = craneLib.cargoFmt {
          inherit src;
          pname = "crysalis-fmt";
        };

        # Audit dependencies
        crysalis-audit = craneLib.cargoAudit {
          inherit src advisory-db;
          pname = "crysalis-audit";
        };

        # Audit licenses
        crysalis-deny = craneLib.cargoDeny {
          inherit src;
          pname = "crysalis-deny";
        };

        # Run tests with cargo-nextest
        crysalis-nextest = craneLib.cargoNextest (commonArgs
          // {
            inherit src cargoArtifacts;
            pname = "crysalis-tests";

            checkPhaseCargoCommand = "cargo nextest run";
            partitions = 1;
            partitionType = "count";
            BANGK_MODE = "TESTING";
          });

        bangk-spellcheck = craneLib.mkCargoDerivation (commonArgs
          // {
            inherit src cargoArtifacts;

            pnameSuffix = "-spellcheck";
            buildPhaseCargoCommand = "HOME=./ cargo spellcheck check -m 1";
            nativeBuildInputs = (commonArgs.buildInputs or []) ++ [pkgs.cargo-spellcheck];
          });
      };
      ######################################################
      ###                 Build packages                 ###
      ######################################################
      packages = {
        default = crysalis-bin;

        coverage = crysalis-coverage;
        docs = crysalis-docs;
      };

      ######################################################
      ###                   Dev’ shell                   ###
      ######################################################
      devShells.default = craneLib.devShell {
        name = "devshell";

        # Inherit inputs from checks.
        checks = self.checks.${system};

        # Additional dev-shell environment variables can be set directly
        PATH = "${pkgs.mold}/bin/mold";

        shellHook = ''
          export PATH="$HOME/.cargo/bin:$PATH"
          echo "Environnement $(basename $(pwd)) chargé" | cowsay | lolcat

          exec $SHELL
        '';

        # Extra inputs can be added here; cargo and rustc are provided by default.
        packages = with pkgs; [
          # virtual machine
          qemu

          # Compilation
          mold # rust linker

          # Utils
          cowsay
          gitmoji-cli # Use gitmojis to commit
          lolcat
          tokei # file lines count
          tokio-console

          # Cargo utilities
          cargo-bloat # check binaries size (which is fun but not terriby useful?)
          cargo-bootimage # to create the bootable image
          cargo-cache # cargo cache -a
          cargo-deny
          cargo-audit
          cargo-expand # for macro expension
          cargo-spellcheck # Spellcheck documentation
          # cargo-wizard
        ];
      };
    });
}
