## Build Instructions

To build for all targets platforms, it would be best to use a Mac. You will also need GNU `make`
installed, at a bare minimum. To ensure that your local machine has all the other necessary
tools installed to perform the builds, run the following from the root of the repo:

```bash
$ make mac-setup
```

### Performing builds

Builds can be performed for the following groups of targets:

- android
- apple
- wasm

For `android` and `apple`, builds will be performed for all supported architectures, whereas
for `wasm`, only a single target will be built. These names refer to Makefile targets that can be
used to build them. For example, to build all `android` targets, execute the following:

```bash
$ make android
```

To build all targets, execute the following:

```bash
$ make all
```

### Other useful targets

- `demo-player`: Build the demo player
- `clean`: Cleanup rust build artifacts
- `distclean`: Cleanup ALL build artifacts

More information can be found by using the `help` target:

```bash
$ make help
```

## Creating a Release

Manually execute the `Create Release PR` Github Action workflow to create a release PR. The will
include all changes since the last release. The repo uses [changesets](https://github.com/changesets/changesets) to determine the new release
version. The [knope](https://github.com/knope-dev/knope) tool can be installed locally and used to
simply the creation of changeset files.

The created release PR should be checked for correctness and then merged. Once merged, the `Release`
Github Actions workflow will be started automatically to do the work of actually creating the new
release and building & uploading the related release artifacts.
