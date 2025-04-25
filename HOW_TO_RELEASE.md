<!--
SPDX-FileCopyrightText: 2022 Helsing GmbH

SPDX-License-Identifier: Apache-2.0
-->

To release a new version:

1. Update the version tag in `Cargo.toml`
2. Create the release tag on GitHub. Pypi release is automated in the CI.
3. Release on Crates.io: `cargo publish`.
