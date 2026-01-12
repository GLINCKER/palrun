class Palrun < Formula
  desc "AI command palette for your terminal - discover and run project commands instantly"
  homepage "https://github.com/GLINCKER/palrun"
  version "0.2.0-beta.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/GLINCKER/palrun/releases/download/v0.2.0-beta.1/palrun-aarch64-apple-darwin.tar.gz"
      sha256 "caa24ca6a912b9ec66e03c485af655bad4465651166a5b64718211adef7a5bf0"
    end
    on_intel do
      url "https://github.com/GLINCKER/palrun/releases/download/v0.2.0-beta.1/palrun-x86_64-apple-darwin.tar.gz"
      sha256 "5eaa0f012c1ee60a3ab1e4b20bc65c5fd91b786dcb41fd47929af321dfef4f7a"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/GLINCKER/palrun/releases/download/v0.2.0-beta.1/palrun-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "598397d64db70355826256224fac7a33f846134ab3dd4531e498759f8125ef23"
    end
    on_intel do
      url "https://github.com/GLINCKER/palrun/releases/download/v0.2.0-beta.1/palrun-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "64a5454ae7c81f76457adb7fc3a2319c9605815a722f44f4c8755d5ff9c0ec7f"
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
