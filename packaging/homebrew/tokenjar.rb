class TokenJar < Formula
  desc "Zero-cost, zero-latency token optimization engine and intelligent MCP middleware"
  homepage "https://github.com/Farukes/TokenJar"
  version "1.1.0"
  license "BUSL-1.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Farukes/TokenJar/releases/download/v1.0.3/tokenjar-darwin-arm64.tar.gz"
      sha256 "9003d12f616077ccd13ff5b649dea347fcdaa287de74b617d1fbbd4da53e1618"
    else
      url "https://github.com/Farukes/TokenJar/releases/download/v1.0.3/tokenjar-darwin-x64.tar.gz"
      sha256 "d2149e96002442eeb35b95260259c392a478eef0646fe1cec3e527f1feebd980"
    end
  end

  on_linux do
    url "https://github.com/Farukes/TokenJar/releases/download/v1.0.3/tokenjar-linux-x64.tar.gz"
    sha256 "83c7f71f1edb7c55fe493c657041eec90247fc2a9ff2bfb2cc0bcd4f7d01d6ad"
  end

  def install
    bin.install "tokenjar"
  end

  test do
    assert_match "tokenjar", shell_output("#{bin}/tokenjar --help")
  end
end
