class CrossBash < Formula


    def install
        bin.install Dir[""]
    end

    test do
        system "#{bin}/mytool", "--version"
    end
end

class CrossBash < Formula
    desc "Crossbash is a cross-platform bash script framework"
    homepage "https://github.com/machinegenesis/crossbash"
    url "https://github.com/machinegenesis/crossbash/archive/v1.0.0.tar.gz"
    sha256 "11a22ae4e826dac5c5258dccee8f4bac270a2dedfbd5cbb0eb6e1af9ba4d6c4f"
    version "1.0.0"

    def install
        bin.install "myscript"
        (etc/"xbash").install "default.conf"
    end

    def post_install
        # Attempt to create directory and copy file (not recommended for actual use)
        system "sudo", "mkdir", "-p", "/etc/xbash"
        system "sudo", "cp", "#{etc}/xbash/default.conf", "/etc/xbash/env"
    end
end
