class MprisDiscordRpc < Formula
  desc "Cross-platform Discord rich presence for music with album cover and progress bar"
  homepage "https://github.com/patryk-ku/mpris-discord-rpc"
  version "0.5.1"
  license "MIT"

  depends_on "media-control"

  on_intel do
    url "https://github.com/patryk-ku/mpris-discord-rpc/releases/download/v#{version}/mpris-discord-rpc-macos-amd64.tar.gz"
    sha256 ""
  end

  on_arm do
    url "https://github.com/patryk-ku/mpris-discord-rpc/releases/download/v#{version}/mpris-discord-rpc-macos-arm64.tar.gz"
    sha256 ""
  end

  def install
    bin.install "mpris-discord-rpc"
  end

  service do
    run [opt_bin/"mpris-discord-rpc"]
    keep_alive true
    environment_variables PATH: "#{HOMEBREW_PREFIX}/bin:/usr/bin:/bin"
    log_path var/"log/mpris-discord-rpc.log"
    error_log_path var/"log/mpris-discord-rpc.error.log"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/mpris-discord-rpc --version")
  end
end
