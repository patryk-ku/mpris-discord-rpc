# Maintainer: Patryk Kurdziel <patryk.kurdziel@protonmail.com>

pkgname=mpris-discord-rpc
pkgver=0.3.0
pkgrel=1
pkgdesc='Linux Discord rich presence for music, using MPRIS with album cover and progress bar support.'
url="https://github.com/patryk-ku/${pkgname}"
license=('MIT')
arch=('x86_64')
source=(
    "${pkgname}-v${pkgver}::${url}/releases/download/v${pkgver}/${pkgname}"
    "https://raw.githubusercontent.com/patryk-ku/${pkgname}/refs/tags/v${pkgver}/mpris-discord-rpc.service"
    "https://raw.githubusercontent.com/patryk-ku/${pkgname}/refs/tags/v${pkgver}/LICENSE"
)
sha512sums=('SKIP' 'SKIP' 'SKIP')

package() {
	install -Dm755 "${pkgname}-v${pkgver}" "${pkgdir}/usr/bin/${pkgname}"
	install -Dm644 mpris-discord-rpc.service "${pkgdir}/usr/lib/systemd/user/mpris-discord-rpc.service"
	install -Dm644 LICENSE "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
}
