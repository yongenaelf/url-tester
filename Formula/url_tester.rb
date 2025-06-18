class UrlTester < Formula
  desc "A CLI tool to test URLs with configurable options"
  homepage "https://github.com/yongenaelf/url-tester"
  version "0.1.0"
  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/yongenaelf/url-tester/releases/download/v0.1.0/url_tester-aarch64-apple-darwin.tar.xz"
      sha256 "15cb40ba2fd6e5343571b1ca6808a0242e9f869c95aa299aa8615155fdcafe7a"
    end
    if Hardware::CPU.intel?
      url "https://github.com/yongenaelf/url-tester/releases/download/v0.1.0/url_tester-x86_64-apple-darwin.tar.xz"
      sha256 "edb326b381a0fea1296056227e7d583a748de10b9def3b414026096039301276"
    end
  end
  if OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/yongenaelf/url-tester/releases/download/v0.1.0/url_tester-aarch64-unknown-linux-gnu.tar.xz"
      sha256 "631c73da936c5f5dcc7e96641b4d5470b66bdfd9c4d344910d5860b29b21e60a"
    end
    if Hardware::CPU.intel?
      url "https://github.com/yongenaelf/url-tester/releases/download/v0.1.0/url_tester-x86_64-unknown-linux-gnu.tar.xz"
      sha256 "1d0b18a85ddfaa25c67361c44bb67d88ceace833cc195384610f9af6ffe45fba"
    end
  end
  license any_of: ["MIT", "Apache-2.0"]

  BINARY_ALIASES = {
    "aarch64-apple-darwin":      {},
    "aarch64-unknown-linux-gnu": {},
    "x86_64-apple-darwin":       {},
    "x86_64-pc-windows-gnu":     {},
    "x86_64-unknown-linux-gnu":  {},
  }.freeze

  def target_triple
    cpu = Hardware::CPU.arm? ? "aarch64" : "x86_64"
    os = OS.mac? ? "apple-darwin" : "unknown-linux-gnu"

    "#{cpu}-#{os}"
  end

  def install_binary_aliases!
    BINARY_ALIASES[target_triple.to_sym].each do |source, dests|
      dests.each do |dest|
        bin.install_symlink bin/source.to_s => dest
      end
    end
  end

  def install
    bin.install "url_tester" if OS.mac? && Hardware::CPU.arm?
    bin.install "url_tester" if OS.mac? && Hardware::CPU.intel?
    bin.install "url_tester" if OS.linux? && Hardware::CPU.arm?
    bin.install "url_tester" if OS.linux? && Hardware::CPU.intel?

    install_binary_aliases!

    # Homebrew will automatically install these, so we don't need to do that
    doc_files = Dir["README.*", "readme.*", "LICENSE", "LICENSE.*", "CHANGELOG.*"]
    leftover_contents = Dir["*"] - doc_files

    # Install any leftover files in pkgshare; these are probably config or
    # sample files.
    pkgshare.install(*leftover_contents) unless leftover_contents.empty?
  end
end
