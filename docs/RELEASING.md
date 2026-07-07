# Releasing

This project has two automation layers:

- `CI` validates the code on every push and pull request.
- `Release Binaries` builds distributable archives for common host platforms.

The capture path is verified on Linux and WSL2 hardware. macOS and Windows
artifacts are build artifacts for integration work and should be hardware-tested
before claiming runtime support on those operating systems.

## Release Targets

| Package | Runner | Rust target | Archive |
| --- | --- | --- | --- |
| `ble-analyzer-pro-rs-linux-x86_64` | `ubuntu-latest` | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| `ble-analyzer-pro-rs-linux-aarch64` | `ubuntu-24.04-arm` | `aarch64-unknown-linux-gnu` | `.tar.gz` |
| `ble-analyzer-pro-rs-macos-x86_64` | `macos-15-intel` | `x86_64-apple-darwin` | `.tar.gz` |
| `ble-analyzer-pro-rs-macos-aarch64` | `macos-latest` | `aarch64-apple-darwin` | `.tar.gz` |
| `ble-analyzer-pro-rs-windows-x86_64` | `windows-latest` | `x86_64-pc-windows-msvc` | `.zip` |

Each package includes:

- CLI binary
- native C ABI library
- C header
- Python wrapper and examples
- README files, docs, license, and udev rules
- SHA-256 checksum file

## Manual Local Package

```bash
make package
```

Override the target and package name when needed:

```bash
make package TARGET=x86_64-unknown-linux-gnu PACKAGE=ble-analyzer-pro-rs-linux-x86_64
```

## GitHub Release

Create and push a semver-style tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The release workflow builds all packages, uploads workflow artifacts, and
attaches them to the GitHub Release for the tag. The workflow can also be run
manually from the Actions tab for preview artifacts.

## Platform Notes

Linux packages dynamically link against system `libusb-1.0`. Install the
runtime package on the target machine.

macOS packages are built with Homebrew `libusb`. Users may need Homebrew
`libusb` installed at runtime.

Windows packages are built with vcpkg `libusb:x64-windows` and include the
`libusb-1.0.dll` runtime when it is present in the runner image.
