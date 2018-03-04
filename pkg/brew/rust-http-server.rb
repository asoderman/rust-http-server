# Documentation: https://docs.brew.sh/Formula-Cookbook
#                http://www.rubydoc.info/github/Homebrew/brew/master/Formula
# PLEASE REMOVE ALL GENERATED COMMENTS BEFORE SUBMITTING YOUR PULL REQUEST!
class RustHttpServer < Formula
  version "0.0.1"
  desc "A simple and fast HTTP server written in rust"
  homepage "https://github.com/asoderman/rust-http-server"
  url "https://github.com/asoderman/rust-http-server/releases/download/0.0.1/rust-http-server-x86_64-apple-darwin.tar.gz"
  sha256 "ee09e552be30bcdd9ccc3883d4cbcca2f305b4b568b2be77206b82ed08358c29"
   # depends_on "cmake" => :build
   def install
       bin.install "release/rust-http-server"
   end
end
