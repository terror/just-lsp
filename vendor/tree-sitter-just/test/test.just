set shell := ["fish", "-c"]
set positional-arguments

export justrun := "just -d " + inst-script-dir + " -f " + justfile()

########################################################
## Most config variables below here
########################################################

to-root := "0"
export stow-package-dir := "packages"
export sysinstalldir := "/usr/local"
export blddir := "~/git-builds"
export inst-script-dir := "~/install-system.sh"
export installdir := if to-root != "0" { sysinstalldir } else {  "$HOME/.local" }

# The best way to use mold is by prefixing `lets install ...` with `mold --run `
export LD := "lld" # gold, mold, lld
export CC :=  "gcc" # gcc-11, clang, clang-12, zig cc, tinycc, zapcc
# CXX := "g++" # g++-11, clang++, clang++-12, zig c++
export CXX := replace(replace(replace(CC, "gcc", "g++"), "clang", "clang++"), "cc", "c++")

static := "0" # TODO: every project needs a different flag for static
native := "1"

use-sccache := "1" # TODO: apply this to c/c++
use-musl := "1" # Used only for cargo for now
cargo-bg := "0"

########################################################
## Most config variables above here
########################################################

export prefix := installdir + "/" + stow-package-dir

export local-apt-repo := inst-script-dir + "/apt-repo"
export local-tar-repo := inst-script-dir + "/tar-repo"
export local-flatpak-repo := inst-script-dir + "/flatpak-repo"

export bindir := installdir + "/bin"
export libdir := installdir + "/lib"
export libexecdir := installdir + "/libexec"
export incdir := installdir + "/include"
export sharedir := installdir + "/share"

export CMAKE_GENERATOR := "Ninja"
export CMAKE_BUILD_TYPE := "Release"
export CMAKE_EXPORT_COMPILE_COMMANDS := "OFF"
export CMAKE_C_COMPILER := CC
export CMAKE_CXX_COMPILER := CXX

export nprocs := `nproc`

export MAKEFLAGS := "-j " + nprocs
# export CARGO_PROFILE_RELEASE_LTO := "true"

static_flags := if static != "0" { " -static" } else { "" }
native_flags := if native != "0" { " -march=native -mtune=native" } else { "" }
fuse-ld := " -fuse-ld=" + LD  # Only support gold and lld
# ld-path := "--ld-path=" + `which LD`

# TODO: options for: LTO, static, mold
compflags := "-O3" + native_flags
ldflags := fuse-ld + native_flags + static_flags
export CFLAGS := compflags
export EXTRA_CFLAGS := compflags
export CXXFLAGS := compflags
export CPPFLAGS := CXXFLAGS
export EXTRA_CPPFLAGS := CXXFLAGS
export FFLAGS := compflags
export FCFLAGS := compflags
export LDFLAGS := ldflags
export EXTRA_LDFLAGS := ldflags

export CMAKE_C_FLAGS_RELEASE := CFLAGS
export CMAKE_CXX_FLAGS_RELEASE := CXXFLAGS
# export CMAKE_EXE_LINKER_FLAGS := ldflags # These don't get passed by environment variable
# export CMAKE_SHARED_LINKER_FLAGS := CMAKE_EXE_LINKER_FLAGS

rust_static_flags := if static != "0" { " -C target-feature=+crt-static -C link-self-contained=yes" } else { "" }
export RUSTFLAGS := "-C link-arg=-fuse-ld=" + LD + " -C target-cpu=native -Z share-generics=y" + rust_static_flags
export RUSTC_WRAPPER := if use-sccache != "0" { "sccache" } else { "" }

gnu-target := "x86_64-unknown-linux-gnu"
musl-target := "x86_64-unknown-linux-musl"
export CARGO_BUILD_TARGET := if use-musl != "0" { musl-target } else { "" }

cargo-install := if cargo-bg != "0" { "pueue add -- cargo install --force --jobs 1" } else { "cargo install" }# TODO: can use quickinstall
# cargo-install := "pueue add -- cargo install --force --jobs 1" # TODO: background compile

# TODO: figure out why/how/what is user vs system
flatpak-exe := if to-root != "0" { "flatpak" } else { "flatpak" }
flatpak-download := flatpak-exe + " create-usb " + local-flatpak-repo
flatpak-install-local := flatpak-exe + " install --or-update --sideload-repo " + local-flatpak-repo + "/.ostree/repo flathub"
flatpak-install-remote := flatpak-exe + " install --or-update flathub"
flatpak-install := flatpak-install-local

apt-install := "sudo apt-get install"

stow := if to-root != "0" { "sudo stow" } else { "stow" }

#################################################
# Recipes below here
#################################################

@_default:
    {{justrun}} --choose --chooser 'fzf --preview="just --show {}"' # --list

edit:
    cd {{inst-script-dir}} && nvim {{justfile()}}

# Open a shell with the environment variables defined above
sh *args:
    fish {{args}}

# mkrepo for offlline apt installs
# Install getpkg, mkrepo for creating offline repositories
getpkg-mkrepo: (mkpackdir "mkrepo")
    #!/usr/bin/env fish
    cd /tmp/
    wget https://gist.github.com/jeanlescure/084dd6113931ea5a0fd9/archive/5866e3e50e452b3a97bb3e534d51d2bba14679a3.zip 
    unzip 5866e3e50e452b3a97bb3e534d51d2bba14679a3.zip 
    chmod +x ./084dd6113931ea5a0fd9-5866e3e50e452b3a97bb3e534d51d2bba14679a3/*.sh
    cp ./084dd6113931ea5a0fd9-5866e3e50e452b3a97bb3e534d51d2bba14679a3/mkrepo.sh \
        ./084dd6113931ea5a0fd9-5866e3e50e452b3a97bb3e534d51d2bba14679a3/getpkg.sh \
        {{prefix}}/mkrepo/bin/
    {{justrun}} link-pack "mkrepo"
    sudo ./084dd6113931ea5a0fd9-5866e3e50e452b3a97bb3e534d51d2bba14679a3/dependencies.sh
# Add a local repo (created using mkrepo, give the directory containing the Packages.gz)
add-local-repo dir=local-apt-repo:
    touch /etc/apt/sources.list.d/local-apt-repo.list
    echo "deb [trusted=yes] file:{{dir}} ./" | sudo tee -a /etc/apt/sources.list.d/local-apt-repo.list
getpkg name: 
    getpkg.sh {{name}} {{local-apt-repo}}
mkrepo: 
    rm {{local-apt-repo}}/Packages.gz || true
    rm {{local-apt-repo}}/Packages || true
    mkrepo.sh {{local-apt-repo}}
    gzip -d {{local-apt-repo}}/Packages.gz
# FIXME: why have to extract Packages??

##############################################333
# Helpers
##############################################333

# Install as root
as-root +name:
    {{justrun}} installdir={{sysinstalldir}} stow='sudo stow' {{name}} 

common: 
    echo "TODO: save list of packages installed by this"

# Install-Upgrade using pipx
pipx-install name: pipx
    pipx install {{name}} || pipx upgrade {{name}}

# Link from a directory inside {{prefix}}
link-pack name:
    {{justrun}} dostow -R {{name}}
# chmod +x {{prefix}}/{{name}}/bin/*
# ln -s {{prefix}}/{{name}}/bin/* {{bindir}}
# ln -s {{prefix}}/{{name}}/lib/* {{libdir}}
# ln -s {{prefix}}/{{name}}/libexec/* {{libexecdir}}
# ln -s {{prefix}}/{{name}}/include/* {{incdir}}
# ln -s {{prefix}}/{{name}}/share/* {{sharedir}}

sustow arg name:
    sudo {{stow}} -d /usr/local/packages/ -t /usr/local {{arg}} {{name}}

dostow arg name:
    {{stow}} --ignore="share/info/dir" -d {{prefix}} -t {{installdir}} {{arg}} {{name}}
# FIXME: share/info/dir ignoring may not work correctly

mkpackdir name:
    mkdir -p {{prefix}}/{{name}}/bin

mkblddir name:
    mkdir -p {{blddir}}/{{name}}/build

# Clone or update a git repo (into blddir)
gitclone url name branch="":
    #!/usr/bin/env fish
    mkdir -p {{blddir}} && cd {{blddir}}
    if test -d {{blddir}}/{{name}}
        cd {{blddir}}/{{name}} 
        git pull && git submodule update --init --recursive \
        || git checkout {{branch}} # Assume that arguments
    else
        git clone --filter=blob:none {{ if branch == "" { "" } else { "-b " + branch } }} {{url}} {{blddir}}/{{name}} # --filter=tree:0
    end

_tmpdir name:
    mkdir -p /tmp/{{name}}

@install-gh:
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo gpg --dearmor -o /usr/share/keyrings/githubcli-archive-keyring.gpg
    echo "deb [arch="(dpkg --print-architecture)" signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
    sudo apt update
    {{apt-install}} gh
    gh auth login

# Install a program directory structure using stow
ins-copy name from: (mkpackdir name)
    cp -r {{from}}/{{name}}/*/* {{prefix}}/{{name}}/
    #rm -rf /tmp/{{name}} # TODO: should delete?
    chmod +x {{prefix}}/{{name}}/bin/*
    {{justrun}} link-pack {{name}}

# Install one or more binaries using stow
ins-bin name from:
    mkdir -p {{from}}/{{name}}/bin
    cp {{from}}/{{name}}/{{name}} {{from}}/{{name}}/bin
    {{justrun}} ins-copy {{name}} {{from}}

# Download an asset from Github releases (to /tmp/name)
gh name repo pattern: (_tmpdir name)
    #!/usr/bin/env fish
    if not test -e "/tmp/{{name}}/{{pattern}}"
        cd /tmp/{{name}} && gh release -R {{repo}} download -p '{{pattern}}'
    end
    test '/tmp/{{name}}/{{name}}' != '/tmp/{{name}}/{{pattern}}'
    and test -e /tmp/{{name}}/{{pattern}}
    and echo "Renaming" '/tmp/{{name}}/{{name}}' != '/tmp/{{name}}/{{pattern}}'
    and cp -v /tmp/{{name}}/{{pattern}} '/tmp/{{name}}/{{name}}'
    or echo "not renamed"
    {{justrun}} gh-check-update {{name}} {{repo}}

gh-check-update name repo:
    #!/usr/bin/env fish
    echo "#!/usr/bin/env fish" > {{blddir}}/{{name}}.sh
    set CURR (gh release -R {{repo}} list | head -1 | cut -f1)
    echo "echo $CURR -> (gh release -R {{repo}} list | head -1 | cut -f1)" >> {{blddir}}/{{name}}.sh
    chmod +x {{blddir}}/{{name}}.sh

# Download a tarball from Github releases and extract it
targh name repo pattern: (gh name repo pattern)
    cd /tmp/{{name}} && tar xavf {{name}}
    cp /tmp/{{name}}/{{name}} {{local-tar-repo}}

# Download a debian from Github releases and install it with dpkg
debgh name repo pattern: (gh name repo pattern)
    cd /tmp/{{name}} && sudo dpkg -i ./{{name}}
    cp /tmp/{{name}}/{{name}} {{local-apt-repo}}

# Download a binary from Github releases and move it to bindir
bingh name repo pattern: (gh name repo pattern) (ins-bin name "/tmp")
    echo "Installed Binary from GH"

# Download a program tarball from Github releases, extract the source to {{prefix}}/, and link the binary folder
insgh name repo pattern: (targh name repo pattern) (ins-copy name "/tmp")
    echo "GH Installed"
    # TODO: Link libraries and stuff also? this works for now

# Download a source tarball from Github releases and extract the source to blddir
srcgh name repo pattern: (targh name repo pattern) (mkblddir name)
    cp -r /tmp/{{name}}/*/* {{blddir}}/{{name}}
    rm -rf /tmp/{{name}}

# Build and Install an autotools project in a folder
autotool-install name options="":
    #!/usr/bin/env fish
    cd {{blddir}}/{{name}}
    ./configure --prefix="{{installdir}}" {{options}}
    and make
    and make install prefix="{{prefix}}/{{name}}"

# Install autotools project from github releases
autotoolsgh name repo pattern: (srcgh name repo pattern)
    {{justrun}} autotool-install "{{name}}"

# Install autotools project from a tarball link
autotoolscurl name url: (curlsrc url name)
    {{justrun}} autotool-install "{{name}}"

# Install autotools project from a git url
autotoolsgit name repo pattern: (gitclone name repo)
    {{justrun}} autotool-install "{{name}}"

# CMake build and install a project
cmake-build from in *options:
    cmake -DCMAKE_INSTALL_PREFIX="{{prefix}}/"(basename (realpath {{from}})) \
        -DCMAKE_BUILD_TYPE={{CMAKE_BUILD_TYPE}} \
        {{options}} -S {{from}} -B {{in}} ..
    cmake --build {{in}}

cmake-install name *options: 
    {{justrun}} cmake-build "{{blddir}}/{{name}}" "{{blddir}}/{{name}}/build" {{options}}
    cmake --install {{blddir}}/{{name}}/build
#   ninja
#   ninja install

cmakegh name repo pattern: (srcgh name repo pattern)
    {{justrun}} cmake-install "{{name}}"

curlbin url name: (mkpackdir name)
    curl -L {{url}} -o {{prefix}}/{{name}}/bin/{{name}}
    chmod +x {{prefix}}/{{name}}/bin/{{name}}

curlsrc url name: (mkblddir name)
    curl -L {{url}} -o {{blddir}}/{{name}}/{{name}}
    cd {{blddir}}/{{name}} && tar -zavf {{name}}

##############################################333
# Programs
##############################################333

# arm embedded gcc - newer than apt
gcc-arm-none-eabi: (insgh "arm-none-eabi-gcc" "xpack-dev-tools/arm-none-eabi-gcc-xpack" "*linux-x64.tar.gz")
    echo "Installed $argv"

# riscv embedded gcc - from xpack
gcc-riscv-none-embed: (insgh "riscv-none-embed-gcc" "xpack-dev-tools/riscv-none-embed-gcc-xpack" "*linux-x64.tar.gz")
    echo "Installed $argv"

# Ultimate Plumber, live shell pipelines 
ultimate-plumber: (bingh "up" "akavel/up" "up")
    echo "Installed UP"

# Wezterm -- cool terminal?
wezterm: (bingh "wezterm" "wez/wezterm" "*.AppImage")
    sudo update-alternatives --install /usr/bin/x-terminal-emulator x-terminal-emulator (which wezterm) 90

# kitty terminal -- pretty graphics in terminal
kitty: (mkpackdir "kitty.app")
    curl -L https://sw.kovidgoyal.net/kitty/installer.sh | sh /dev/stdin dest={{prefix}}
    {{justrun}} link-pack "kitty.app"

kitty-src: (gitclone "https://github.com/kovidgoyal/kitty" "kitty") (mkpackdir "kitty")
    {{apt-install}} libdbus-1-dev libxcursor-dev libxrandr-dev libxi-dev libxinerama-dev libgl1-mesa-dev libxkbcommon-x11-dev libfontconfig-dev libx11-xcb-dev liblcms2-dev libpython3-dev
    cd {{blddir}}/kitty && make

# Zenith -- better top
zenith: (debgh "zenith" "bvaisvil/zenith" "*amd64.deb")
    echo "Installed zenith"

# PathPicker -- terminal path picker
pathpicker: (gitclone "https://github.com/facebook/PathPicker" "pathpicker")
    #!/usr/bin/env -S bash -euxo pipefail
    cd {{blddir}}/pathpicker/debian 
    sh ./package.sh
    sudo dpkg -i ../*.deb

gvm:
    #!/usr/bin/env fish
    if not test -e ~/.gvm/scripts/gvm
        curl -sSL https://raw.githubusercontent.com/moovweb/gvm/master/binscripts/gvm-installer | bash
    end
    if functions -q gvm
        function gvm
          bass source ~/.gvm/scripts/gvm ';' gvm $argv
        end
        funcsave gvm
    end

go: gvm
    gvm install --binary go1.16
    gvm use go.1.16 --default
    gvm alias create latest go1.16

nvm:
    #!/usr/bin/env fish
    if type -q nvm
        fisher install jorgebucaran/nvm.fish
    end

npm: nvm
    nvm install latest
    nvm install lts
    nvm use latest

node: npm
    echo "Installed node+npm"

# cod -- completion learning daemon
cod:
    go get -u github.com/dim-an/cod

# gnome-quake
# go to https://extensions.gnome.org/extension/1411/quake-mode/

# localtunnel
localtunnel:
    npm install -g localtunnel

# nnn - not that, its a file manager
nnn:
    echo 'deb http://download.opensuse.org/repositories/home:/stig124:/nnn/xUbuntu_20.04/ /' | sudo tee /etc/apt/sources.list.d/home:stig124:nnn.list
    curl -fsSL https://download.opensuse.org/repositories/home:stig124:nnn/xUbuntu_20.04/Release.key | gpg --dearmor | sudo tee /etc/apt/trusted.gpg.d/home_stig124_nnn.gpg > /dev/null
    sudo apt update
    {{apt-install}} nnn
    {{justrun}} getpkg nnn

# tinycc compiler
tinycc: (gitclone "https://github.com/TinyCC/tinycc.git" "tinycc")
    #!/usr/bin/env -S bash -euxo pipefail
    cd {{blddir}}/tinycc
    ./configure --prefix={{prefix}}/tinycc \
    --extra-cflags={{compflags}} \
    --extra-ldflags={{ldflags}} \
    --cc={{CC}}
    make cross
    make install
    {{justrun}} link-pack "tinycc"

# keynav
keynav: (gitclone "https://github.com/jordansissel/keynav.git" "keynav") (mkpackdir "keynav")
    #!/usr/bin/env -S bash -euxo pipefail
    cd {{blddir}}/keynav
    make 
    ln -s $PWD/keynav {{prefix}}/keynav/bin
    {{justrun}} link-pack "keynav"

# zig compiler
zig: (gitclone "https://github.com/ziglang/zig.git" "zig")
    #!/usr/bin/env fish
    {{apt-install}} libllvm12 libclang-12-dev liblld-12-dev
    cmake -DCMAKE_INSTALL_PREFIX="{{prefix}}/zig/" \
        -DCMAKE_BUILD_TYPE={{CMAKE_BUILD_TYPE}} \
        -DCMAKE_C_COMPILER=clang-12 -DCMAKE_CXX_COMPILER=clang++-12 \
        -S {{blddir}}/zig -B {{blddir}}/zig/build
    and cmake --build {{blddir}}/zig/build
    and cmake --install {{blddir}}/zig/build
    {{justrun}} link-pack "zig"

# godbolt
godbolt: (gitclone "https://github.com/compiler-explorer/compiler-explorer.git" "compiler-explorer")
    cargo install rustfilt

# Caching C/C++ compiler for higher compile speed (not updated?)
zapcc: (gitclone "https://github.com/yrnkrn/zapcc.git" "zapcc")
    read --prompt "Are you sure you wanna build zapcc?"
    cmake -DLLVM_ENABLE_WARNINGS -DLLVM_USE_LINKER={{LD}} \
        -DCMAKE_INSTALL_PREFIX="{{prefix}}/zapcc/" \
        -DCMAKE_BUILD_TYPE={{CMAKE_BUILD_TYPE}} \
        -S {{blddir}}/zapcc -B {{blddir}}/zapcc/build
    cmake --install {{blddir}}/zapcc/build
    {{justrun}} link-pack "zapcc"

# google benchmark
googbench: (gitclone "https://github.com/google/benchmark.git" "googbench")
    cmake -DBENCHMARK_DOWNLOAD_DEPENDENCIES=ON -DBENCHMARK_ENABLE_GTEST_TESTS=OFF \
        -DCMAKE_INSTALL_PREFIX={{prefix}}/googbench \
        -DCMAKE_BUILD_TYPE={{CMAKE_BUILD_TYPE}} \
        -S {{blddir}}/googbench -B {{blddir}}/googbench/build
    cmake --install {{blddir}}/googbench/build
    {{justrun}} link-pack "googbench"

# drake
# cd ~/git-builds
# git clone https://github.com/RobotLocomotion/drake.git
# cd drake
# git remote add upstream git@github.com:RobotLocomotion/drake.git
# git remote set-url --push upstream no_push
# sudo setup/ubuntu/install_prereqs.sh
# bazel build
# docker pull robotlocomation/drake:latest
# cd tools/install/dockerhub/focal
# wget https://drake-packages.csail.mit.edu/drake/nightly/drake-latest-focal.tar.gz
# docker build -t robotlocomation/drake:focal

# todoist cli
todoist-cli: (gitclone "https://github.com/sachaos/todoist.git" "todoist")
    go install 

# tikzit
tikzit: (gitclone "https://github.com/tikzit/tikzit.git" "tikzit")
    {{apt-install}} flex bison qt5-default libpoppler-dev libpoppler-qt5-dev
    qmake -r
    make

# clifm
clifm: (debgh "clifm" "leo-arch/clifm" "*deb")
    {{apt-install}} libcap-dev libacl1-dev libreadline-dev

# mpv video player
mpv:
    sudo add-apt-repository -y ppa:mc3man/mpv-tests
    sudo apt-get update
    {{apt-install}} mpv

# mps-youtube -- cli youtube client
mps-youtube: mpv
    {{justrun}} pipx-install git+https://github.com/mps-youtube/mps-youtube.git
    pipx inject mps-youtube youtube-dl 
    pipx inject mps-youtube pyperclip

# Run a program on a remote computer
outrun:
    {{justrun}} pipx-install outrun

# SSH with your shell environment
xxh:
    {{justrun}} pipx-install xxh-xxh

# Futhark
futhark: (gitclone "https://github.com/diku-dk/futhark.git" "futhark")
    stack setup
    stack build
    stack install

# Ansi aware column command
ansicolumn:
    curl -sL http://cpanmin.us | sudo perl - App::ansicolumn

# Install systemwide VSCode (update via apt)
vscode: 
    #!/usr/bin/env -S bash -euxo pipefail
    cd /tmp/
    wget "https://code.visualstudio.com/sha/download?build=stable&os=linux-deb-x64" -O code.deb
    sudo dpkg -i code.deb
    sudo ln -s /usr/share/code/bin/code /usr/local/bin

# Install portable version of vscode
code-portable: (_tmpdir "code-portable") (mkpackdir "code-portable")
    #!/usr/bin/env -S bash -euxo pipefail
    cd /tmp/code-portable
    wget "https://code.visualstudio.com/sha/download?build=stable&os=linux-x64" -O portable.tar.gz
    tar -xzf portable.tar.gz
    cp -r VSCode-linux-x64 {{prefix}}/code-portable
    mkdir -p {{prefix}}/code-portable/data
    ln -s {{prefix}}/code-portable/code {{bindir}}/code-portable

# Install portable version of vscode
# code-appimage: (bingh "code-oss" "zilti/code-oss.AppImage" "*AppImage")
# mkdir -p {{prefix}}/code-oss/bin/code-oss.home
# mkdir -p {{prefix}}/code-oss/bin/code-oss.config

# TODO: add doom to yadm submodule
# EMACS, a great OS and a mediocre editor
emacs: (gitclone "https://github.com/emacs-mirror/emacs.git" "emacs")
    #!/usr/bin/env -S bash -euxo pipefail
    cd {{blddir}}/emacs
    {{apt-install}} libxaw7-dev libgnutls30 xaw3dg xutils-dev libgnutls28-dev libgccjit-10-dev
    ./autogen.sh
    ./configure --with-native-compilation --with-modules=on --prefix="{{prefix}}/emacs"
    make -C lisp autoloads-force
    make
    make install
    {{justrun}} link-pack "emacs"
    
# Git Credential Manager Core
gcm: (debgh "gcmcore" "microsoft/Git-Credential-Manager-Core" "*amd64*.deb")
    git-credential-manager-core configure
# git config --global credential.credentialStore secretservice # requires GUI (ok?)

# Quantum Mechanical Keyboard firmware
qmk: (gitclone "https://github.com/IndianBoy42/qmk_firmware.git" "qmk_firmware")
    cd ~/qmk_firmware
    git remote add upstream "https://github.com/qmk/qmk_firmware.git" && \
    sudo cp ~/qmk_firmware/util/udev/50-qmk.rules /etc/udev/rules.d/
    pipx install --user qmk
    qmk setup
    qmk doctor

# CMake (not a) build system
cmake: (insgh "cmake" "Kitware/CMake" "*linux-x86_64.tar.gz")
    echo "CMake Installed"

# Fuzzy finder
fzf: (targh "fzf" "junegunn/fzf" "*linux_amd64.tar.gz") (ins-bin "fzf" "/tmp")
    echo "Installed fzf"

# Fast realtime compression/decompression
# zstd: (srcgh "zstd" "facebook/zstd" "*.tar.gz") (mkpackdir "zstd")
zstd: (gitclone "https://github.com/facebook/zstd.git" "zstd") (mkpackdir "zstd")
    cd {{blddir}}/zstd && make
    ln -s {{blddir}}/zstd/zstd {{prefix}}/zstd/bin
    {{justrun}} link-pack "zstd"

# Computer-side QMK mapping for builtin keyboards
kmonad: (bingh "kmonad" "kmonad/kmonad" "*linux")
    getent group uinput || sudo groupadd uinput
    sudo usermod -aG uinput $USER
    sudo usermod -aG input $USER
    sudo modprobe uinput
    sudo cp {{inst-script-dir}}/uinput.rules /etc/udev/rules.d/uinput.rules
    systemctl --user enable kmonad.service
    systemctl --user start kmonad.service
    echo "The next output will help you find the device that your keyboard corresponds to"
    cat /proc/bus/input/devices | rg -C5 keyboard

# Alternative to submodules
git-subrepo: (gitclone "ingydotnet/git-subrepo" "git-subrepo") (mkpackdir "git-subrepo")
    ln -s {{blddir}}/git-subrepo/lib/* {{prefix}}/git-subrepo/bin
    {{justrun}} link-pack "git-subrepo"

# Modern Latex distribution
tectonic: (targh "tectonic" "tectonic-typesetting/tectonic" "*x86_64-unknown-linux-gnu.tar.gz") (ins-bin "tectonic" "/tmp")
    echo "Installed tectonic"

# A Version manager for Julia
jill: 
    curl -fsSL https://raw.githubusercontent.com/abelsiqueira/jill/master/jill.sh > {{bindir}}/jill.sh
    sudo chmod +x {{bindir}}/jill.sh

# Math+Science programming language
julia: jill
    jill.sh -y

# Install a Nerdfont by name
nerdfont name: install-gh
    #!/usr/bin/env fish
    echo "[-] Download font ({{name}}) [-]"
    mkdir -p ~/Downloads/fonts
    cd ~/Downloads/fonts
    set fontfilename "$argv.zip"
    gh release -R ryanoasis/nerd-fonts download -p $fontfilename
    and unzip $fontfilename -d ~/.fonts
    echo "done!"

# For patching nerdfonts yourself
fontforge: 
    sudo add-apt-repository ppa:fontforge/fontforge
    sudo apt-get update
    sudo apt-get install fontforge

# Nerdfont patching utility
font-patcher: fontforge (mkpackdir "nerd-font-patcher")
    wget https://raw.githubusercontent.com/ryanoasis/nerd-fonts/master/font-patcher -O {{prefix}}/nerd-font-patcher/bin
    {{justrun}} nerd-font-patcher

# Patch an existing font file with Nerdfonts
patch-font path: 
    font-patcher --complete --progressbars -out ~/.fonts {{path}}  

# Install Fira Code|Mono Nerd Font
firacode: (nerdfont "FiraCode") (nerdfont "FiraMono")
    fc-cache -fv

# Installer/Updater for GHC
ghcup:
    #!/usr/bin/env fish
    if not type -q ghcup
        curl --proto '=https' --tlsv1.2 -sSf https://get-ghcup.haskell.org | sh
    else
        ghcup
    end

# Build system for Haskell (uses isolated GHC instances)
stack:
    #!/usr/bin/env fish
    if not type -q stack
        curl -sSL https://get.haskellstack.org/ | sh -s -- -d {{prefix}}/stack/bin
        {{justrun}} link-pack stack
    else
        stack upgrade
    end

# Add common targets and component for rustup
rustup_targets:
    rustup target add x86_64-unknown-linux-gnu      
    rustup target add x86_64-unknown-linux-musl
    rustup target add thumbv7m-none-eabi
    rustup target add thumbv7em-none-eabi
    rustup target add thumbv7em-none-eabihf
    rustup target add aarch64-unknown-linux-musl
    rustup target add arm-unknown-linux-musl
    rustup component add rust-analyzer-preview

_rustup: 
    #!/usr/bin/env fish
    if not type -q rustup
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly --profile default
    end

# Rust toolchain manager
rustup: _rustup rustup_targets
    echo "Installed Rustup"

# Rustc compiler
rustc: _rustup rustup_targets
    rustup update

# Download prebuilt binaries for crates
cargo-quickinstall:
    cargo install cargo-quickinstall

# Helper for updating things install via cargo
cargo-update:
    CARGO_BUILD_TARGET={{gnu-target}} {{cargo-install}} cargo-update

# Shared cache for C/C++/Rust
sccache:
    echo "Installing sccache"
    @type -q sccache && cargo install-update sccache
    @type -q sccache || {{cargo-install}} sccache

# Download sccache from github releases
sccache-gh: (bingh "sccache" "mozilla/sccache" "sccache-*-x86_64-unknown-linux-musl.tar.gz")
    echo "Installed"

# Cargo install tools (sccache, quickinstall, cargo-update)
cargo-install: 
    type -q cargo-quickinstall || {{justrun}} cargo-quickinstall
    type -q cargo-install-update || {{justrun}} cargo-update
    type -q sccache || {{justrun}} sccache

# Blazing fast regex searcher
ripgrep:
    {{cargo-install}} $argv[1]

# Use ripgrep to search inside all kinds of files
ripgrep_all: ripgrep (gitclone "https://github.com/phiresky/ripgrep-all.git" "ripgrep_all")
    {{cargo-install}} --path {{blddir}}/ripgrep_all

# Easy to use TUI for git
gitui: cargo-install (gitclone "https://github.com/IndianBoy42/gitui.git" "gitui")
    {{cargo-install}} --path {{blddir}}/gitui
    cp {{blddir}}/gitui/vim_style_key_config.ron ~/.config/gitui/key_config.ron

# Minimal serial terminal/monitor
serial-monitor: cargo-install (gitclone "https://github.com/IndianBoy42/serial-monitor.git" "serial-monitor")
    {{cargo-install}} --path {{blddir}}/serial-monitor

# Index and search all local files
indexa: cargo-install (gitclone "https://github.com/mosmeh/indexa.git" "indexa")
    {{cargo-install}} --path {{blddir}}/indexa

# Fast implementation of the tldr standard
tealdeer: cargo-install
    CARGO_BUILD_TARGET={{gnu-target}} {{cargo-install}} $argv[1]
    tldr --update

# Manager for long running background shell commands/processes
pueue: cargo-install
    {{cargo-install}} $argv[1]
    systemctl --user enable pueued.service
    and systemctl --user start pueued.service
    pueue parallel (nproc)

# Network and port usage monitor
bandwhich: cargo-install
    {{cargo-install}} $argv[1]
    type -q bandwhich && sudo setcap cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep (which bandwhich)

# High performance GPU accelerated Terminal
alacritty: cargo-install
    {{cargo-install}} alacritty
    not type -q alacritty
    {{justrun}} pipx-install alacritty-colorscheme
    wget https://raw.githubusercontent.com/alacritty/alacritty/master/extra/logo/alacritty-term.svg -O /tmp/alacritty-term.svg
    sudo cp /tmp/alacritty-term.svg /usr/share/pixmaps/Alacritty.svg
    wget https://raw.githubusercontent.com/alacritty/alacritty/master/extra/alacritty.info -O /tmp/alacritty.info
    sudo tic -xe alacritty,alacritty-direct /tmp/alacritty.info
    wget https://raw.githubusercontent.com/alacritty/alacritty/master/extra/linux/Alacritty.desktop -O /tmp/Alacritty.desktop
    sudo desktop-file-install /tmp/Alacritty.desktop
    sudo update-desktop-database
    sudo update-alternatives --install /usr/bin/x-terminal-emulator x-terminal-emulator (which alacritty) 99

broot: cargo-install
    mkdir -p ~/.local/share/fonts
    test -f ~/.local/share/fonts/vscode.ttf; \
    or wget https://github.com/Canop/broot/raw/master/resources/icons/vscode/vscode.ttf -O ~/.local/share/fonts/vscode.ttf
    {{cargo-install}} broot

cargo-build-tools:
    #!/usr/bin/env fish
    set -x CARGO_BUILD_TARGET {{gnu-target}}
    {{cargo-install}} cargo-edit 
    {{cargo-install}} cargo-generate 
    {{cargo-install}} cargo-outdated 
    {{cargo-install}} probe-run 
    {{cargo-install}} cargo-update
    {{cargo-install}} cargo-binutils cargo-do 
    {{cargo-install}} cargo-watch
    {{cargo-install}} cargo-equip
    {{cargo-install}} crate2nix
    {{cargo-install}} flamegraph  

cargo-essentials: cargo-install gitui ripgrep broot tealdeer
    {{cargo-install}} bat
    {{cargo-install}} kn
    {{cargo-install}} zoxide
    {{cargo-install}} lsd
    {{cargo-install}} bottom
    {{cargo-install}} fd-find
    {{cargo-install}} git-delta 
    {{cargo-install}} huniq
    {{cargo-install}} hyperfine  
    {{cargo-install}} pipe-rename 
    {{cargo-install}} du-dust

cargo-all-gnu: cargo-install tealdeer
    #!/usr/bin/env fish
    set -x CARGO_BUILD_TARGET {{gnu-target}}
    {{cargo-install}} cargo-edit 
    {{cargo-install}} cargo-generate 
    {{cargo-install}} cargo-outdated 
    {{cargo-install}} probe-run 
    {{cargo-install}} cargo-update 
    {{cargo-install}} spotify-tui 
    {{cargo-install}} so 
    {{cargo-install}} starship  
    {{cargo-install}} duma  

cargo-all-musl: cargo-install serial-monitor bandwhich pueue indexa
    {{cargo-install}} macchina
    {{cargo-install}} onefetch
    {{cargo-install}} drone
    {{cargo-install}} dua
    {{cargo-install}} eva
    {{cargo-install}} exa 
    {{cargo-install}} fd-find
    {{cargo-install}} flip-link 
    {{cargo-install}} gfold
    {{cargo-install}} gping
    {{cargo-install}} mdbook
    {{cargo-install}} mdcat
    {{cargo-install}} procs
    {{cargo-install}} ptail
    {{cargo-install}} rustfilt  
    {{cargo-install}} scriptisto
    {{cargo-install}} sd
    {{cargo-install}} shy
    {{cargo-install}} skim 
    {{cargo-install}} verco 
    {{cargo-install}} watchexec-cli
    {{cargo-install}} xsv
    {{cargo-install}} tokei
    {{cargo-install}} gpg-tui 

cargo +args: cargo-install
    {{cargo-install}} {{args}}

# Install 'all' the cargo binaries (that i like)
cargo-all: cargo-essentials cargo-build-tools cargo-all-gnu cargo-all-musl
    {{justrun}} cargo-needs-update

# List cargo binaries that need updates
cargo-needs-update: 
    cargo install-update --all --list

# Install rust-analyzer from github
rust-analyzer: (bingh "rust-analyzer" "rust-analyzer/rust-analyzer" "*-linux")
    echo "Installed rust-analyzer"
# rust-analyzer: (curlbin "https://github.com/rust-analyzer/rust-analyzer/releases/latest/download/rust-analyzer-linux" "rust-analyzer")
#   {{justrun}} link-pack "rust-analyzer"

# Install just prebuilt
just: (mkpackdir "just")
    curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to {{prefix}}/just/bin
    {{justrun}} link-pack "just"
# Install just from source
just-cargo:
    {{cargo-install}} just

# Install/update Anaconda
conda:
    #!/usr/bin/env fish
    if not type -q conda
        wget https://repo.anaconda.com/archive/Anaconda3-2020.11-Linux-x86_64.sh -O /tmp/anaconda.sh
        sh /tmp/anaconda.sh -b -u
        conda config --add channels conda-forge
    end
    conda update -n base -c defaults --all

# Python dev tools
pydev: conda pipx pipenv poetry
    {{justrun}} pipx-install black mypy

# 'Essential' python packages
py-essentials:
    pip3 install --user click argcomplete rich ptpython ipython bpython
    conda install jupyter

# Isolated pip installs for applications
pipx:
    pip3 install --user pipx
    pipx ensurepath

# Pipenv venv manager
pipenv: pipx
    {{justrun}} pipx-install pipenv

# Poetry python dev environment manager
poetry: 
    {{justrun}} pipx-install poetry
#!/usr/bin/env fish
#if type -q poetry
#    poetry self update
#else
#    curl -sSL https://raw.githubusercontent.com/python-poetry/poetry/master/install-poetry.py | python - --preview
#end

# Temporary venvs for quick scripts
pip-run:
    {{justrun}} pipx-install pip-run 
    pipx inject pip-run click rich ptpython ipython numpy matplotlib ipython ptpython bpython

# Create conda environment from file
conda-env name:
    conda env create -f {{name}} || conda env update --prune -n (cat {{name}} | rg 'name: ([\S]+)' -C0 -r '$1')
    conda activate (cat {{name}} | rg 'name: ([\S]+)' -C0 -r '$1') && {{justrun}} py-essentials

# vulkan api sdk
vulcan-sdk:
    wget -qO - https://packages.lunarg.com/lunarg-signing-key-pub.asc | sudo apt-key add -
    sudo wget -qO /etc/apt/sources.list.d/lunarg-vulkan-focal.list https://packages.lunarg.com/vulkan/lunarg-vulkan-focal.list
    sudo apt update
    {{apt-install}} vulkan-sdk

# List installed flatpaks
@flatpak-list:
    flatpak list --app --columns=name:f | tail -n +1     

# Install flatpak and some packages (optionally)
flatpak +name:
    #!/usr/bin/env fish
    if not type -q flatpak
        {{apt-install}} flatpak
        flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
        flatpak remote-modify --collection-id=org.flathub.Stable flathub
    end
    test +n "{{name}}" && {{flatpak-install}} {{name}}

kicad:
    {{flatpak-install}} org.kicad.KiCad \
        org.kicad.KiCad.Library.Templates   \
        org.kicad.KiCad.Library.Symbols     \
        org.kicad.KiCad.Library.Packages3D  \
        org.kicad.KiCad.Library.Packages3D  \
        org.kicad.KiCad.Library.Footprints 
spotify: 
    {{flatpak-install}} com.spotify.Client 
anydesk: 
    {{flatpak-install}} com.anydesk.Anydesk 
gimp: 
    {{flatpak-install}} org.gimp.GIMP 
inkscape: 
    {{flatpak-install}} org.inkscape.Inkscape 
whatsapp: 
    {{flatpak-install}} io.bit3.WhatsAppQT 
kdenlive: 
    {{flatpak-install}} org.kde.kdenlive 
telegram: 
    {{flatpak-install}} org.telegram.desktop
discord: 
    {{flatpak-install}} com.discordapp.Discord 
signal: 
    {{flatpak-install}} org.signal.Signal 
vlc: 
    {{flatpak-install}} org.videolan.VLC 
slack: 
    {{flatpak-install}} com.slack.Slack 
todoist: 
    {{flatpak-install}} com.todoist.Todoist 
blender: 
    {{flatpak-install}} org.blender.Blender
megasync:
    {{flatpak-install}} nz.mega.MEGAsync

# Flatpak install and download to a local repo
flatpak-dl +command:
    {{justrun}} flatpak-install="{{flatpak-install-remote}}" {{command}} # Install first 
    {{justrun}} flatpak-install="{{flatpak-download}}" {{command}} # Then pack into the repo

# Install docker and enable permissions
docker:
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
    sudo rm /usr/share/keyrings/docker-archive-keyring.gpg
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
    sudo add-apt-repository -y "deb [arch=amd64] https://download.docker.com/linux/ubuntu $distro stable"
    apt update
    {{apt-install}} docker-ce docker-ce-cli containerd.io
    getent group docker || sudo groupadd docker
    sudo usermod -aG docker $USER
    newgrp docker 

# Update apt sources
@apt: 
    sudo apt-get update > /tmp/aptupdate.log || echo "Not connected to the internet, hope you have a cache"

# Some apt sources
apt-srcs:
    {{inst-script-dir}}/install-apt-srcs.fish

# Install all apt packages from a file in install-system.sh
@apt-all file: apt
    {{apt-install}} (cat {{inst-script-dir}}/{{file}}.apt)

# Install apt libraries
@apt-libs: apt
    {{justrun}} apt-all libs
    {{apt-install}} linux-tools-(uname -r)
# Install apt applications
@apt-apps: apt
    {{justrun}} apt-all apps

# Download all apt packages from a file in install-system.sh
@_apt-repo name:
    echo "The getpkg.sh script throws a lot of errors about missing dependencies, and returns error even when successful"
    xargs -P(math (nproc) "* 8") -i sh -c "just getpkg {} \
    || true" < {{inst-script-dir}}/{{name}}.apt

# Download all my apt packages into a repo
@apt-repo:
    {{justrun}} _apt-repo "system"
    {{justrun}} _apt-repo "libs"
    {{justrun}} _apt-repo "apps"
    {{justrun}} mkrepo

# Install newer git
git:
    sudo add-apt-repository -y ppa:git-core/ppa
    apt update
    {{apt-install}} git

# Sublime merge
smerge: 
    wget -qO - https://download.sublimetext.com/sublimehq-pub.gpg | sudo apt-key add - 
    echo "deb https://download.sublimetext.com/ apt/stable/" | sudo tee /etc/apt/sources.list.d/sublime-text.list
    apt update
    {{apt-install}} sublime-merge

# TODO: install in an isolated virtual environment? (use pipx)
nvim-cfg:
    test -d ~/.config/nvim || https://github.com/IndianBoy42/LunarVim.git ~/.config/nvim
    cd ~/.config/nvim && just install

# Install neovim from unstable ppa (also install my lunarvim)
nvim-apt:
    sudo add-apt-repository -y ppa:neovim-ppa/unstable
    apt update
    {{apt-install}} neovim
    {{justrun}} nvim-cfg

nvim-app: (bingh "neovim" "neovim/neovim" "*.appimage")
    {{justrun}} nvim-cfg

# Compile neovim from source
nvim: (gitclone "https://github.com/neovim/neovim.git" "neovim")
    cd {{blddir}}/neovim \
    && make \
        CMAKE_BUILD_TYPE=Release \
        CMAKE_INSTALL_PREFIX={{prefix}}/nvim \
        install
    {{justrun}} link-pack nvim
    {{justrun}} nvim-cfg

# Pretty GUI for neovim written in rust
neovide: (gitclone "https://github.com/Kethku/neovide" "neovide")
    #!/usr/bin/env fish
    if not type -q neovide
        {{apt-install}} curl \
            gcc-multilib g++-multilib \
            libfreetype6-dev libasound2-dev libexpat1-dev libxcb-composite0-dev \
            libbz2-dev libsndio-dev freeglut3-dev libxmu-dev libxi-dev libsdl2-dev
    end
    cd {{blddir}}/neovide
    CARGO_BUILD_TARGET={{gnu-target}} {{cargo-install}} --path .

# Selection first modal editor
kakoune: (gitclone "https://github.com/mawww/kakoune.git" "kakoune") (mkpackdir "kakoune")
    sudo apt install libncursesw5-dev pkg-config
    cd {{blddir}}/kakoune/src\
    && make CXX={{CXX}} \
    && PREFIX={{prefix}}/kakoune make install
    {{justrun}} link-pack "kakoune"

# Pack all necessary sources/repos
pack-repos:
    #tar --zstd -cf {{inst-script-dir}}/repos.tar.zst
    tar --zstd -cvf /dev/null \
        {{local-apt-repo}}  \
        {{local-flatpak-repo}}  \
        {{local-tar-repo}}  \
        {{blddir}}  \
        ~/.cargo/bin  \
        ~/.cargo/.crates.toml  \
        {{prefix}} 

# GNU Stow symlink farm manager -- build stow from source 
stow-src: (gitclone "https://github.com/aspiers/stow.git" "stow") (mkpackdir "stow")
    #!/usr/bin/env fish
    cd {{blddir}}/stow
    autoreconf -iv
    ./configure --prefix="{{installdir}}"
    make
    make install prefix="{{prefix}}/stow"
    {{justrun}} stow="./bin/stow" link-pack "stow"

# GNU Stow symlink farm manager -- Choose apt or src
stow: 
    #!/usr/bin/env fish
    if confirm "Install from source? [y/N]"
        {{justrun}} stow-src
    else
        {{apt-install}} stow
    end

# JSON query CLI
jq: (bingh "jq" "stedolan/jq" "jq-linux64")
    echo "Installed jq"

# Interactive JSON Query CLI
ijq: (gitclone "https://github.com/gpanders/ijq.git" "ijq") (mkpackdir "ijq")
    {{apt-install}} scdoc
    cd {{blddir}}/ijq && make prefix={{prefix}}/ijq install
    {{justrun}} link-pack "ijq"

# https://medium.com/@ankurloriya/install-gnome-extension-using-command-line-736199be1cda
# Install extension from the command line
gnome-extension path:
    #!/usr/bin/env fish
    set name (basename {{path}})
    mkdir -p /tmp/$name
    cd /tmp/$name
    cp {{path}} .
    extract $name
    test -d $name
    set uuid (cat $name/metadata.json | jq -r '.uuid')
    mkdir -p ~/.local/share/gnome-shell/extensions/
    cp -r $name $uuid
    gnome-shell-extension-tool enable $uuid

# Nix package manager
nix:
    curl -L https://nixos.org/nix/install | sh
    echo "bass source .nix-profile/etc/profile.d/nix.sh"

# Traceroute + ping gtk/curses
mtr: (gitclone "https://github.com/traviscross/mtr.git" "mtr")
    cd {{blddir}}/mtr && ./bootstrap.sh
    {{justrun}} autotool-install mtr

platformio: 
    {{justrun}} pipx-install platformio

lua:
    {{apt-install}} liblua5.4-dev lua5.4 luajit

luarocks: (gitclone "git://github.com/luarocks/luarocks.git" "luarocks")
    #!/usr/bin/env fish
    type -q lua || {{justrun}} lua
    cd {{blddir}}/luarocks
    ./configure --prefix="{{prefix}}/luarocks"
    and make
    and make install 
    and {{justrun}} link-pack luarocks

rock +args:
    type -q luarocks || {{justrun}} luarocks
    luarocks install {{args}}
    {{justrun}} link-pack luarocks

lua-format: 
    type -q luarocks || {{justrun}} luarocks
    luarocks install --server=https://luarocks.org/dev luaformatter
    and {{justrun}} link-pack luarocks

# Build a static executable of the mold linker (fast boi)
mold-stable: (gitclone "https://github.com/rui314/mold.git" "mold" "v0.9.1") (gitclone "https://github.com/oneapi-src/oneTBB.git" "mold/oneTBB" "v2020.3") (mkpackdir "mold")
    cd {{blddir}}/mold/oneTBB && make extra_inc=big_iron.inc
    tbb_bindir=(ls -d {{blddir}}/mold/oneTBB/build/linux_intel64_*_release) \
    cd {{blddir}}/mold && make \
        PREFIX={{prefix}}/mold \
        CC=clang-12 CXX=clang++-12 LTO=1 \
        EXTRA_CPPFLAGS=-IoneTBB/include \
        EXTRA_LDFLAGS="-fuse-ld=lld -static -L"(ls -d {{blddir}}/mold/oneTBB/build/linux_intel64_*_release) \
        install
    {{justrun}} link-pack mold

# Build a static executable of the mold linker (fast boi)
mold: (gitclone "https://github.com/rui314/mold.git" "mold" "main") (mkpackdir "mold")
    cd {{blddir}}/mold && make \
        PREFIX={{prefix}}/mold \
        CC=clang-12 CXX=clang++-12 LTO=1 \
        EXTRA_LDFLAGS="-fuse-ld=lld-12 -static" install
    {{justrun}} link-pack mold

# Use update-alternatives
alternative name alt="" prio="0": 
    test -n "{{alt}}" && \
    sudo update-alternatives --install (which {{name}}) {{name}} (realpath (which {{alt}})) {{prio}} \
    || true
    sudo update-alternatives --list {{name}}
altconfig name:
    sudo update-alternatives --config {{name}}

jemalloc: (gitclone "https://github.com/jemalloc/jemalloc.git" "jemalloc")
    #!/usr/bin/env fish
    cd {{blddir}}/jemalloc
    ./autogen.sh
    and ./configure --prefix="{{prefix}}/jemalloc"
    and make
    and make install
    and {{justrun}} link-pack jemalloc

mimalloc: (gitclone "https://github.com/microsoft/mimalloc.git" "mimalloc")
    #!/usr/bin/env fish
    cd {{blddir}}/mimalloc
    {{justrun}} cmake-install mimalloc
    and {{justrun}} link-pack mimalloc
