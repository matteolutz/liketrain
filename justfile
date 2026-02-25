set shell := ["powershell.exe", "-c"]

project_name := file_name(justfile_directory())
default_bin := "ui"

[group('project-agnostic')]
default:
    @just --list --justfile {{justfile()}}

[group("development")]
[arg("type", pattern="bin|lib")]
new package type="bin":
    cargo new crates/{{project_name}}-{{package}} --{{type}} --vcs none

[group("development")]
[arg("profile", pattern="release|debug")]
build package="" profile="debug":
    cargo build {{ if profile == "release" { "--release" } else { "" } }} {{ if package == "" { "" } else { f'-p {{project_name}}-{{package}}' } }}

[group("development")]
[arg("profile", pattern="release|debug")]
run bin=default_bin profile="debug":
    cargo run --bin {{project_name}}-{{bin}} {{ if profile == "release" { "--release" } else { "" } }}

[group("development")]
[confirm("Are you sure you want to clean the cargo target directory? (y/N)")]
clean:
    cargo clean

[group("testing")]
test package="":
    cargo test {{ if package == "" { "" } else { f'-p {{project_name}}-{{package}}' } }} -- --nocapture
