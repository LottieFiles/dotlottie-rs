class Meson < Formula
  desc "Fast and user friendly build system"
  homepage "https://mesonbuild.com/"
  url "https://github.com/mesonbuild/meson/releases/download/1.6.0/meson-1.6.0.tar.gz"
  sha256 "999b65f21c03541cf11365489c1fad22e2418bb0c3d50ca61139f2eec09d5496"
  license "Apache-2.0"
  head "https://github.com/mesonbuild/meson.git", branch: "master"

  bottle do
    sha256 cellar: :any_skip_relocation, all: "77eb91483991e1b615c3fe1dcea843e43706ac3eb011b0556fcf70e3a3e097fb"
  end

  depends_on "ninja"
  depends_on "python@3.13"

  def install
    python3 = "python3.13"
    system python3, "-m", "pip", "install", *std_pip_args(build_isolation: true), "."

    bash_completion.install "data/shell-completions/bash/meson"
    zsh_completion.install "data/shell-completions/zsh/_meson"
    vim_plugin_dir = buildpath/"data/syntax-highlighting/vim"
    (share/"vim/vimfiles").install %w[ftdetect ftplugin indent syntax].map { |dir| vim_plugin_dir/dir }

    # Make the bottles uniform. This also ensures meson checks `HOMEBREW_PREFIX`
    # for fulfilling dependencies rather than just `/usr/local`.
    mesonbuild = prefix/Language::Python.site_packages(python3)/"mesonbuild"
    usr_local_files = %w[
      compilers/mixins/apple.py
      coredata.py
      dependencies/boost.py
      dependencies/cuda.py
      dependencies/misc.py
      dependencies/qt.py
      options.py
      scripts/python_info.py
      utils/universal.py
    ].map { |f| mesonbuild/f }
    usr_local_files << (bash_completion/"meson")

    # Passing `build.stable?` ensures a failed `inreplace` won't fail HEAD installs.
    inreplace usr_local_files, "/usr/local", HOMEBREW_PREFIX, audit_result: build.stable?

    opt_homebrew_files = %w[
      dependencies/boost.py
      dependencies/misc.py
      compilers/mixins/apple.py
    ].map { |f| mesonbuild/f }
    inreplace opt_homebrew_files, "/opt/homebrew", HOMEBREW_PREFIX, audit_result: build.stable?

    # Ensure meson uses our `var` directory.
    inreplace mesonbuild/"options.py", "'/var/local", "'#{var}", audit_result: build.stable?
  end

  test do
    (testpath/"helloworld.c").write <<~C
      #include <stdio.h>
      int main(void) {
        puts("hi");
        return 0;
      }
    C
    (testpath/"meson.build").write <<~EOS
      project('hello', 'c')
      executable('hello', 'helloworld.c')
    EOS

    system bin/"meson", "setup", "build"
    assert_predicate testpath/"build/build.ninja", :exist?

    system bin/"meson", "compile", "-C", "build", "--verbose"
    assert_equal "hi", shell_output("build/hello").chomp
  end
end
