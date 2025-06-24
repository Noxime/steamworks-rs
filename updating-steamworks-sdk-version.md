# How to update the Steamworks SDK version
This file is so that I remember how to do it.

# 1. Download the latest version
https://partner.steamgames.com/downloads/steamworks_sdk.zip

# 2. Extract the contents
Copy the contents of "sdk/" in the archive to the steamworks-sys/lib/steam directory.

# 3. Rebuild the bindings
Commit the changes so far and push to a new branch. Go to the rebuild-bindings
action and run it on your new branch.
https://github.com/Noxime/steamworks-rs/actions/workflows/rebuild-bindings.yml

You can then download the newly generated bindings from the artifacts section.
Replace steamworks-sys/src/*_bindings.rs with the new bindings.

# 4. Build steamworks-sys
cargo build -p steamworks-sys

# 5. Upgrade rest of the crate
Now build the main crate and fix any issues that arise with the new version.
