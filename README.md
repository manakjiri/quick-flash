# quick-flash

[![Continuous Deployment](https://github.com/manakjiri/quick-flash/actions/workflows/release-plz.yml/badge.svg)](https://github.com/manakjiri/quick-flash/actions/workflows/release-plz.yml)

[![Demo video](https://raw.githubusercontent.com/manakjiri/quick-flash/main/docs/media/demo-preview.png)](https://raw.githubusercontent.com/manakjiri/quick-flash/main/docs/media/demo.mp4)

A simple and portable program which pulls built firmware binaries from an [S3-compatible object storage API](https://github.com/durch/rust-s3) and downloads them onto the target using a [probe-rs](https://github.com/probe-rs/probe-rs) compatible debug probe.

The idea is to solve the problem of distributing up-to-date binaries within a hardware dev-team, which should aid testing and debugging efforts.

> But what value does this tool bring? I can flash my boards from my IDE or using existing tooling.

Of course, us firmware developers are comfortable with git, the build toolchain and the debugger tooling, but a hardware designer may just want to flash a version and continue troubleshooting.

Other uses may include

- flashing known firmware version as a part of a HIL testing pipeline
- distributing CI/CD artifacts built in well defined environment as opposed to using binaries built by developers locally

## Overview

The tool performs simple, read-only operations on the remote storage to list the available firmware versions, downloads them and caches them locally. It does not rely on a manifest file to discover available versions.

This means that new firmware binaries can be uploaded by existing tools from a CI pipeline of your choice or even by hand. With that, you can also set an expiration time for these artifacts, without introducing inconsistencies.

Initial setup steps are described below.

## 1. Install

### Binary

The tool is available as a binary for all major platforms and can be installed easily with one command. Refer to the [Releases page](https://github.com/manakjiri/quick-flash/releases) for instructions.

### From source

Alternatively you can also compile it locally using cargo

```sh
cargo install quick-flash
```

**On Linux, make sure you have the `libudev` library installed** prior to building. See [probe-rs documentation](https://github.com/probe-rs/probe-rs?tab=readme-ov-file#building) for more details.

### Check that it works

In either case, verify that the tool was installed correctly by executing `quick-flash --version`. If this is your first Rust program, you may need to add `~/.cargo/bin/` to `PATH`.

To upgrade, simply repeat the install step.

## 2. Storage setup

_Currently only tested with Cloudflare R2, but other providers are easy to incorporate._

The tool expects to find the built firmware in an **ELF** format under `name/version/firmware.elf`, where the `name` can be used to differentiate projects and the `version` can be a semver, a git hash or anything you like (avoid using spaces and slashes).

You should produce a directory structure in the root of the bucket that looks something like this:

```
/
└── name/
    └── version/
        ├── firmware.elf
        └── manifest.json
```

The `manifest.json` file is also required and currently only specifies the target chip name, which is passed to `probe-rs`. Refer to the [probe-rs target list](https://probe.rs/targets) to pick the correct entry.

```json
{
  "chip": "STM32L053R8Tx"
}
```

Once done, create an object read-only API token, ideally scoped at that specific bucket containing the firmware and nothing else. For this tool to be useful, it is expected that these credentials will be shared and stored on other machines.

## 3. Credentials

Once you run the tool for the first time, it will ask you to input credential for the storage

```
➜  ~ quick-flash --list
Input credentials for the R2 bucket below:
Storage Name: <the name of the bucket>
Storage Account ID: <your account ID>
Storage Access Key: <bucket access key>
Storage Secret Key: <bucket secret key>
Saving credentials to /home/<user>/.config/quick-flash/credentials.toml...
```

simply copy-paste each field. The `credentials.toml` file location is dependent on the host OS. Using the `--list` option confirms that the storage connection works (see below). If your bucket is empty, the tool will display an error message.

If you input something incorrectly, you can modify the credentials file directly or abort the prompt by `Ctrl+C` and run the program again. You can remove stored credentials using `--clear-credentials`.

## 4. Basic usage

Below are basic usage examples running against a private demo bucket.

**List all firmware names (aka top level directories in the storage bucket)**

```
quick-flash --list
```

may output

```
Listing 1 available firmware name:
  - blinky
```

**List all known versions for specific firmware (aka `/<firmware name>/*`)**

```
quick-flash blinky --list
```

may output

```
Listing 2 versions of firmware "blinky"
  - fast
  - slow
```

**Flash the firmware**

```
quick-flash blinky fast
```

see the demonstration video at the top of this page.
