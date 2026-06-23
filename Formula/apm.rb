class Apm < Formula
  desc "Agent package manager for Claude Code"
  homepage "https://github.com/zanets/apm"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/zanets/apm/releases/download/v#{version}/apm-aarch64-apple-darwin.tar.gz"
      sha256 "0fa136879bc3e0eae91d21d714e8549a6b7e3e985ca53007a7d966a60eed0527"
    end

    on_intel do
      url "https://github.com/zanets/apm/releases/download/v#{version}/apm-x86_64-apple-darwin.tar.gz"
      sha256 "6e96b077a489b63b5f6c3d9600e3623a0b995cd4ac0efb5837291b639488fc09"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/zanets/apm/releases/download/v#{version}/apm-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "1c7162069a45c11fd47fcd526b823af908098d0d38bd76112ebc78abcb050725"
    end

    on_intel do
      url "https://github.com/zanets/apm/releases/download/v#{version}/apm-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "8ec274758e5c66d3957fc09bd15f12ef41df9ff2b074f4cac49f686f808dad7c"
    end
  end

  def install
    bin.install "apm"
  end

  test do
    assert_match "Agent package manager", shell_output("#{bin}/apm --help")
  end
end
