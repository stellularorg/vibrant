# 🌸 Vibrant CLI (vibsync)

Sync CLI for Vibrant projects.

> **During BETA, you'll have to build the CLI manually on Windows, MacOS, and Linux! See instructions below.** \

## Build

```bash
cargo install just
git clone https://code.stellular.org/stellular/vibrant
cd vibrant/cli
just build
```

## Configuration

Configure server: (`https://stellular.org` is default)

- Create a file named `.vibrant.toml`
- Set the global `server` key

## Usage

Authenticate with secondary token:

```bash
# command
vibsync token {token}

# example
vibsync token 0000000000
```

Link a project:

```bash
# command
vibsync init {project}

# example
vibsync init oss
```

Pull files:

```bash
# command
vibsync pull

# example
vibsync pull
```

Push files (create):

```bash
# command
vibsync push -c {files...}

# example
vibsync push -c index.html style.css index.js
```

Push files (update):

```bash
# command
vibsync push {files...}

# example
vibsync push index.html style.css index.js
```

Delete files:

```bash
# command
vibsync remove {files...}

# example
vibsync remove index.html style.css index.js
```

## Examples

Editing just `index.html`:

```bash
# (project and token already set)
vibsync pull # sync from server
# edit file locally...
vibsync push index.html # sync to server
# remove file locally...
vibsync remove index.html # sync to server
```
