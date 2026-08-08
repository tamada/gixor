---
title: ":rhino: Inside of Gixor"
description: "The inside of Gixor."
date: "2025-09-27"
---

## :walking: Process of Dump

Gixor is a tool to manage the `.gitignore` file for multiple repositories.
This document describes the inside of Gixor.
Gixor dumps the specified boilerplates in following process:

* :one: **Initialization** Gixor reads the configuration file and initializes the logger and the configuration.
  * The default location of the configuration file (`config.json`) is the standard directory.
    * macOS: `~/Library/Application Support/gixor/config.json`
    * Linux: `~/.config/gixor/config.json`
    * Windows: `%APPDATA%\gixor\config.json`
  * The configuration file has the boilerplate repositories and their local base directories.
  * If `config.json` was not found, Gixor creates a new configuration file with the default values.
* :two: **Repository Initialization** Gixor confirms the local boilerplate repositories and updates them if necessary.
  * Gixor clones the boilerplate repositories if they are not found in the local base directories.
  * Gixor updates the boilerplate repositories if the last update is older than the specified period.
* :three: **Finding** Gixor parses the command-line arguments and options.
  * Gixor assumes each name consists of the repository name and boilerplate names separated by a slash (`/`).  The repository name is optional, and the boilerplate name is mandatory.
    If the repository name is omitted, Gixor searches the boilerplate name from all repositories with case-insensitive.
* :four: **Dumping** Gixor updates the `.gitignore` file in place.
  * Gixor reads the destination if it exists, taking the part before the first boilerplate as the prologue,
    and the boilerplates already listed there as the entries to keep. Adding to what is already
    there is the default; `--no-append` drops the entries, `--clear-prologue` drops the prologue,
    and `--clear` drops both.
  * An entry listed in the `.gitignore` that no longer resolves to a boilerplate, because it was
    renamed or removed upstream, is reported as a warning and dropped. A name given on the command
    line that does not resolve is an error, since that is a mistake worth stopping for.
  * Then, Gixor builds the prologue and the found boilerplates in memory, writes them to a
    temporary file next to the destination, and renames it over the destination. The whole
    content is known to be complete before anything is replaced, so a failure at any point leaves
    the existing `.gitignore` exactly as it was. Use `--dry-run` to see the result without writing.
* :five: **Finalization** Gixor writes the configuration file into the loaded location, if the configuration file was updated.

## :name_badge: The repository name

The repository name is the short identifier of the boilerplate repository.
And then, the user of Gixor should specify the boilerplate name with the repository name, such as `default/rust`.
However, the repository name is optional, and the user can specify the boilerplate name only, such as `rust`.
In this case, Gixor searches the boilerplate name from all repositories with case-insensitive
and dumps the first found boilerplate.

## :seedling: Prologue of `.gitignore`

The prologue is the first part of the `.gitignore` file in the current directory.
The first part means the content in the `.gitignore` file until the first boilerplate.
It is where your own rules live, so Gixor carries it over on every dump and writes it to the
output before the specified boilerplates. Pass `--clear-prologue` (or `--clear`) to drop it.

## `config.json`

### An example of `config.json`

```json
{
    "base-path": "boilerplates",
    "repositories": [
        {
            "url": "https://github.com/github/gitignore.git",
            "repo-name": "gitignore",
            "owner": "github",
            "name": "default",
            "path": "default"
        }, {
            "url": "https://github.com/tamada/gitignore.git",
            "repo-name": "gitignore",
            "owner": "tamada",
            "name": "tamada",
            "path": "tamada"
        }
    ]
}
```

### Schema of `config.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://example.com/product.schema.json",
  "title": "gixor config json schema",
  "description": "The configuration file of Gixor.",
  "type": "object",
  "properties": {
    "base-path": {
      "type": "string",
      "description": "The base path of the boilerplate repositories. If this path is forms in the relative path, the relatively from this configuration path."
    },
    "repositories": {
      "type": "array",
      "description": "The list of the boilerplate repositories.",
      "items": {
        "type": "object",
        "description": "The boilerplate repository.",
        "properties": {
          "url": {
            "type": "string",
            "description": "The Git URL of the boilerplate repository. The URL should forms in <PROTOCOL://HOST/OWNER/REPO-NAME>."
          },
          "repo-name": {
            "type": "string",
            "description": "The name of the boilerplate repository."
          },
          "owner": {
            "type": "string",
            "description": "The owner of the boilerplate repository."
          },
          "name": {
            "type": "string",
            "description": "The name of the boilerplate repository."
          },
          "path": {
            "type": "string",
            "description": "The local path of the boilerplate repository relatively from the base path defined in this configuration file."
          }
        }
      }
    }
  }
}
```
