{
  description = "ProtoBot, a Discord bot made for friends";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-hooks-nix = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.git-hooks-nix.flakeModule ];

      systems = [
        "x86_64-linux"
      ];

      perSystem =
        {
          config,
          system,
          pkgs,
          lib,
          ...
        }:
        let
          toolchainFor = p: p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain toolchainFor;

          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              (craneLib.fileset.commonCargoSources ./.)
              ./CODENAME
            ];
          };

          commonArgs = {
            inherit src;
            strictDeps = true;
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          protobot = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              doCheck = false;
            }
          );

          pkgsMusl = pkgs.pkgsCross.musl64;
          craneLibMusl = (inputs.crane.mkLib pkgsMusl).overrideToolchain toolchainFor;
          protobot-static = pkgsMusl.callPackage (
            _:
            craneLibMusl.buildPackage {
              inherit src;
              strictDeps = true;
              doCheck = false;
            }
          ) { };

          image = pkgs.dockerTools.buildLayeredImage {
            name = "ghcr.io/klercke/protobot";
            tag = protobot-static.version;
            extraCommands = "mkdir -p app/data app/logs";
            fakeRootCommands = "chown -R 65534:65534 app";
            config = {
              Entrypoint = [ "${protobot-static}/bin/protobot" ];
              WorkingDir = "/app";
              User = "65534:65534";
              Volumes."/app/data" = { };
              Labels = {
                "org.opencontainers.image.source" = "https://github.com/klercke/ProtoBot";
                "org.opencontainers.image.licenses" = "GPL-3.0-only";
              };
            };
          };

          staticOnly = lib.optionalAttrs (system == "x86_64-linux") { inherit protobot-static image; };
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          packages = {
            default = protobot;
            inherit protobot;
          }
          // staticOnly;

          checks = {
            inherit protobot;

            clippy = craneLib.cargoClippy (
              commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              }
            );

            nextest = craneLib.cargoNextest (
              commonArgs
              // {
                inherit cargoArtifacts;
              }
            );

            deny = craneLib.cargoDeny (
              commonArgs
              // {
                src = lib.fileset.toSource {
                  root = ./.;
                  fileset = lib.fileset.unions [
                    (craneLib.fileset.commonCargoSources ./.)
                    ./deny.toml
                  ];
                };
              }
            );
          }
          // staticOnly;

          pre-commit.settings = {
            package = pkgs.prek;
            hooks = {
              rustfmt = {
                enable = true;
                packageOverrides = {
                  cargo = toolchainFor pkgs;
                  rustfmt = toolchainFor pkgs;
                };
              };
              nixfmt.enable = true;
              statix.enable = true;
              deadnix.enable = true;
              taplo.enable = true;
              typos.enable = true;
              actionlint.enable = true;
              zizmor.enable = true;
              convco.enable = true;
            };
          };

          devShells.default = craneLib.devShell {
            packages = [
              pkgs.cargo-nextest
              pkgs.cargo-deny
              pkgs.sqlite
              pkgs.prek
            ]
            ++ config.pre-commit.settings.enabledPackages;
            shellHook = config.pre-commit.installationScript;
          };

          formatter = pkgs.nixfmt;
        };
    };
}
