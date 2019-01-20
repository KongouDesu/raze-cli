# raze-cli

An example project using the [Raze][1] library to make a simple CLI backup tool.
A bit hacked together but it should be documented well enough to understand what's going on.

This example and associated library is a proof of concept to show a working backup implementation. Refer to https://crates.io/crates/backblaze-b2 for a complete API implementation.

## Explanation
This is a simple CLI backup tool. You use your BackBlaze API key to authenticate and edit a file called "backuplist" to specify what you want to back up.
The backuplist file must have exactly one folder on each line. Everything in the folder and every subfolder will be backed up.

The tool provides a 'help' command and will automatically guide you through setting up the bucket used for backup. \
Credentials can also be provided through a file named "raze_credentials" containing a single line: "keyId:applicationKey" without quotes.

   [1]: https://github.com/KongouDesu/raze
