#!/usr/bin/env bash
set -euo pipefail

target="${1:?target triple is required}"
package="${2:?package name is required}"
profile_dir="target/${target}/release"
dist_dir="dist"
root="${dist_dir}/${package}"

case "${target}" in
  *-apple-darwin)
    dylib="libble_analyzer_pro.dylib"
    ;;
  *-linux-gnu)
    dylib="libble_analyzer_pro.so"
    ;;
  *)
    echo "unsupported Unix target: ${target}" >&2
    exit 2
    ;;
esac

rm -rf "${root}"
mkdir -p "${root}/bin" "${root}/lib" "${root}/include" "${root}/python" "${root}/examples" "${root}/docs"

cp "${profile_dir}/ble-analyzer-pro" "${root}/bin/"
cp "${profile_dir}/${dylib}" "${root}/lib/"
cp include/ble_analyzer_pro.h "${root}/include/"
cp python/ble_analyzer_pro.py "${root}/python/"
cp examples/*.py "${root}/examples/"
cp docs/*.md "${root}/docs/"
cp README.md README.zh-CN.md LICENSE Cargo.toml 99-wch-ble-analyzer.rules "${root}/"

chmod +x "${root}/bin/ble-analyzer-pro"

archive="${dist_dir}/${package}.tar.gz"
tar -czf "${archive}" -C "${dist_dir}" "${package}"

if command -v sha256sum >/dev/null 2>&1; then
  (cd "${dist_dir}" && sha256sum "${package}.tar.gz" > "${package}.tar.gz.sha256")
else
  (cd "${dist_dir}" && shasum -a 256 "${package}.tar.gz" > "${package}.tar.gz.sha256")
fi

echo "wrote ${archive}"
