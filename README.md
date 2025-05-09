# wing

`wing` is a command line utility that sends a notification to a Discord webhook when a command finishes executing.

## Installation

### From source

The only currently supported way of installing is to build from source using `cargo`. Installing is then done by cloning and running

```
cargo install --path <path-to-source>
```

`cargo` can be installed by [`rustup`](https://rustup.rs/) or your system package manager like `brew` (MacOS), `pacman` (Arch), `apt` (Debian) or `winget` (Windows).

## Usage

Before using `wing` you need to set a couple of environment variables to set the webhook you will be sending the completion message to.
For example a Discord webhook URL looks something like this:

```
https://discord.com/api/webhooks/WEBHOOK_ID/WEBHOOK_TOKEN
```

Then we set the following environment variables

- `WING_WEBHOOK_ID`: Set this to the webhook ID.
- `WING_WEBHOOK_TOKEN`: Set this to the webhook token.

Once configured, you can run commands like this:

```
wing <command> ...
```

When the given command completes, `wing` will send a notification to the webhook.
