# quick-flash

A simple and portable program which pulls built firmware binaries from an [S3-compatible object storage API](https://github.com/durch/rust-s3) and downloads them onto the target using a [probe-rs](https://github.com/probe-rs/probe-rs) compatible debug probe.

## Storage setup

Currently only tested with Cloudflare R2, but other providers are easy to incorporate. The tool expects to find the built firmware in an ELF format under `name/version/firmware.elf`, where the `name` can be used to differentiate projects and the `version` can be a semver, a git hash or anything you like.

```
/
└── name/
    └── version/
        ├── firmware.elf
        └── manifest.json
```

The `manifest.json` file currently only specifies the target chip name

```json
{
  "chip": "STM32L053R8Tx"
}
```
