set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

root_dir := justfile_directory()
# Show available commands
default:
    @just --list

setup-epaint:
    @just --justfile "{{justfile()}}" setup-target \
        "ext/egui" \
        "ext/egui/crates/epaint" \
        "1669e52a7ccfc3489c1b0999b9ed48894a0b3887" \
        "https://github.com/emilk/egui.git" \
        "patches/epaint_0.31.1_layout.patch" \
        ""

setup-difftastic:
    @just --justfile "{{justfile()}}" setup-target \
        "ext/difftastic" \
        "ext/difftastic" \
        "a6611b97a35a240a3751594234540dfccbd104a6" \
        "https://github.com/Wilfred/difftastic.git" \
        "patches/difftastic_0.70.0.patch" \
        "src vendored_parsers"

setup-target clone_dir target_dir repo_rev repo_url target_patch sparse_dirs:
    mkdir -p "{{root_dir}}/ext"
    git config --global core.autocrlf false
    if [ ! -d "{{root_dir}}/{{clone_dir}}/.git" ]; then \
      git clone --depth 1 --filter=blob:none --no-checkout "{{repo_url}}" "{{root_dir}}/{{clone_dir}}"; \
    else \
      echo "{{clone_dir}} already exists; skipping clone"; \
    fi
    cd "{{root_dir}}/{{clone_dir}}" && git fetch --depth 1 origin "{{repo_rev}}"
    if [ -n "{{sparse_dirs}}" ]; then \
      cd "{{root_dir}}/{{clone_dir}}" && git sparse-checkout init --cone; \
      cd "{{root_dir}}/{{clone_dir}}" && git sparse-checkout set {{sparse_dirs}}; \
    fi
    cd "{{root_dir}}/{{clone_dir}}" && git checkout --detach FETCH_HEAD
    test -d "{{root_dir}}/{{target_dir}}"
    cd "{{root_dir}}/{{target_dir}}" && git apply --check "{{root_dir}}/{{target_patch}}"
    cd "{{root_dir}}/{{target_dir}}" && git apply "{{root_dir}}/{{target_patch}}"

reset-target clone_dir target_dir repo_rev target_patch:
    test -d "{{root_dir}}/{{clone_dir}}/.git"
    cd "{{root_dir}}/{{clone_dir}}" && git fetch --depth 1 origin "{{repo_rev}}"
    cd "{{root_dir}}/{{clone_dir}}" && git reset --hard FETCH_HEAD
    cd "{{root_dir}}/{{clone_dir}}" && git clean -fd
    cd "{{root_dir}}/{{target_dir}}" && git apply --check "{{root_dir}}/{{target_patch}}"
    cd "{{root_dir}}/{{target_dir}}" && git apply "{{root_dir}}/{{target_patch}}"

setup-ext: setup-epaint setup-difftastic