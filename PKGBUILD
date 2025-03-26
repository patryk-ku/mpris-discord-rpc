# Maintainer: Patryk Kurdziel <patryk.kurdziel@protonmail.com>

pkgname=mpris-discord-rpc
pkgver=0.2.2
pkgrel=1
pkgdesc='MPRIS Discord music rich presence status with support for album covers and progress bar.'
url="https://github.com/patryk-ku/$pkgname"
license=('MIT')
arch=('x86_64')
source=(
    "https://github.com/patryk-ku/$pkgname/releases/download/v$pkgver/$pkgname"
    "https://raw.githubusercontent.com/patryk-ku/$pkgname/main/LICENSE"
)
sha512sums=('SKIP' 'SKIP')

package() {
	install -Dm755 "mpris-discord-rpc" -t "$pkgdir/usr/bin"
	install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
