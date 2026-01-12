class Palrun < Formula
  desc "AI command palette for your terminal - discover and run project commands instantly"
  homepage "https://github.com/GLINCKER/palrun"
  version "0.2.0-beta.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/GLINCKER/palrun/releases/download/v0.2.0-beta.2/palrun-aarch64-apple-darwin.tar.gz"
      sha256 "987b340061744cbd1caa65a22bd1cbd0b54227c11e16d0b62fa27d3104732747"
    end
    on_intel do
      url "https://github.com/GLINCKER/palrun/releases/download/v0.2.0-beta.2/palrun-x86_64-apple-darwin.tar.gz"
      sha256 "139d9ca9b45a7081ebe0d5f1e1106c7cf962e5eddce607c1c53a818e47751a9d"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/GLINCKER/palrun/releases/download/v0.2.0-beta.2/palrun-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "0ae856b13543eb4bdbfbfa69845403cc8d05008cc371d0682799683c7fdd8878"
    end
    on_intel do
      url "https://github.com/GLINCKER/palrun/releases/download/v0.2.0-beta.2/palrun-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "6ab92fb50869a8483f192b30d24c67a1699cb5b45b58888bdc21d1fe05f7d696"
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
