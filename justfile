set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

ROOT_DIR := justfile_directory()
EPAINT_REPO := "https://github.com/emilk/egui.git"
EPAINT_TAG := "0.31.1"
EGUI_DIR := "ext/egui"
EPAINT_DIR := "ext/egui/crates/epaint"
EPAINT_PATCH := "patches/epaint_0.31.1_layout.patch"

# Show available commands
default:
    @just --list

# Clone full egui repo into ext/egui and apply text_layout patch to epaint.
setup-epaint:
    mkdir -p "{{ROOT_DIR}}/ext"
    git config --global core.autocrlf false
    if [ ! -d "{{ROOT_DIR}}/{{EGUI_DIR}}/.git" ]; then \
      git clone --depth 1 --branch "{{EPAINT_TAG}}" "{{EPAINT_REPO}}" "{{ROOT_DIR}}/{{EGUI_DIR}}"; \
    else \
      echo "{{EGUI_DIR}} already exists; skipping clone"; \
    fi
    test -d "{{ROOT_DIR}}/{{EPAINT_DIR}}"
    cd "{{ROOT_DIR}}/{{EPAINT_DIR}}" && git apply --check "../../../../{{EPAINT_PATCH}}"
    cd "{{ROOT_DIR}}/{{EPAINT_DIR}}" && git apply "../../../../{{EPAINT_PATCH}}"

# Reset ext/egui to the target tag and re-apply patch.
reset-epaint:
    test -d "{{ROOT_DIR}}/{{EGUI_DIR}}/.git"
    cd "{{ROOT_DIR}}/{{EGUI_DIR}}" && git fetch --depth 1 origin tag "{{EPAINT_TAG}}"
    cd "{{ROOT_DIR}}/{{EGUI_DIR}}" && git reset --hard "{{EPAINT_TAG}}"
    cd "{{ROOT_DIR}}/{{EGUI_DIR}}" && git clean -fd
    cd "{{ROOT_DIR}}/{{EPAINT_DIR}}" && git apply --check "../../../../{{EPAINT_PATCH}}"
    cd "{{ROOT_DIR}}/{{EPAINT_DIR}}" && git apply "../../../../{{EPAINT_PATCH}}"

# Remove vendored egui completely.
clean-epaint:
    rm -rf "{{ROOT_DIR}}/{{EGUI_DIR}}"

# Recreate vendored egui from scratch.
recreate-epaint: clean-epaint setup-epaint

# Check whether the epaint patch can be applied cleanly.
check-epaint-patch:
    test -d "{{ROOT_DIR}}/{{EGUI_DIR}}/.git"
    test -d "{{ROOT_DIR}}/{{EPAINT_DIR}}"
    cd "{{ROOT_DIR}}/{{EPAINT_DIR}}" && git apply --check "../../../../{{EPAINT_PATCH}}"

# Update the epaint patch file from current epaint local changes.
update-epaint-patch:
    test -d "{{ROOT_DIR}}/{{EGUI_DIR}}/.git"
    test -d "{{ROOT_DIR}}/{{EPAINT_DIR}}"
    cd "{{ROOT_DIR}}/{{EPAINT_DIR}}" && git diff > "../../../../{{EPAINT_PATCH}}"

# Setup all vendored dependencies.
setup-ext: setup-epaint

# Reset all vendored dependencies and re-apply patches.
reset-ext: reset-epaint

# Recreate all vendored dependencies from scratch.
recreate-ext: recreate-epaint

# Check all patches.
check-patches: check-epaint-patch