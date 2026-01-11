class Palrun < Formula
  desc "AI command palette for your terminal - discover and run project commands instantly"
  homepage "https://github.com/GLINCKER/palrun"
  version "0.2.0-beta.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/GLINCKER/palrun/releases/download/v#{version}/palrun-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
    on_intel do
      url "https://github.com/GLINCKER/palrun/releases/download/v#{version}/palrun-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/GLINCKER/palrun/releases/download/v#{version}/palrun-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER"
    end
    on_intel do
      url "https://github.com/GLINCKER/palrun/releases/download/v#{version}/palrun-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  def install
    bin.install "palrun"
    bin.install "pal"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/palrun --version")
  end
end
