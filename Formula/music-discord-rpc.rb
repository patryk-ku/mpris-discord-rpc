class MusicDiscordRpc < Formula
  desc "Cross-platform Discord rich presence for music with album cover and progress bar"
  homepage "https://github.com/patryk-ku/music-discord-rpc"
  version "0.6.0"
  license "MIT"

  depends_on "media-control"

  on_intel do
    url "https://github.com/patryk-ku/music-discord-rpc/releases/download/v#{version}/music-discord-rpc-macos-amd64.tar.gz"
    sha256 "a3f6a36627ac4a4c40ce563daf3e908b4f53efac9ec909b12f7c77ae213d7dc1"
  end

  on_arm do
    url "https://github.com/patryk-ku/music-discord-rpc/releases/download/v#{version}/music-discord-rpc-macos-arm64.tar.gz"
    sha256 "214eecce687f1545ef56c95d2bd2a8d6dfd2e083ae9f8ea5089ea31ef552fa9b"
  end

  def install
    bin.install "music-discord-rpc"
  end

  service do
    run [opt_bin/"music-discord-rpc"]
    keep_alive true
    environment_variables PATH: "#{HOMEBREW_PREFIX}/bin:/usr/bin:/bin"
    log_path var/"log/music-discord-rpc.log"
    error_log_path var/"log/music-discord-rpc.error.log"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/music-discord-rpc --version")
  end
end
