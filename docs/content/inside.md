---
title: "🦏 Inside of Gixor"
description: "The inside of Gixor."
---

## 🚶 Process of Dump

Gixor is a tool to manage the `.gitignore` file for multiple repositories.
This document describes the inside of Gixor.
Gixor dumps the specified boilerplates in following process:

* 1️⃣ **Initialization** Gixor reads the configuration file and initializes the logger and the configuration.
  * The default location of the configuration file (`config.json`) is the standard directory.
    * macOS: `~/Library/Application Support/gixor/config.json`
    * Linux: `~/.config/gixor/config.json`
    * Windows: `%APPDATA%\gixor\config.json`
  * The configuration file has the boilerplate repositories and their local base directories.
  * If `config.json` was not found, Gixor creates a new configuration file with the default values.
* 2️⃣ **Repository Initialization** Gixor confirms the local boilerplate repositories and updates them if necessary.
  * Gixor clones the boilerplate repositories if they are not found in the local base directories.
  * Gixor updates the boilerplate repositories if the last update is older than the specified period.
* 3️⃣ **Finding** Gixor parses the command-line arguments and options.
  * Gixor assumes each name consists of the repository name and boilerplate names separated by a slash (`/`).  The repository name is optional, and the boilerplate name is mandatory.
    If the repository name is omitted, Gixor searches the boilerplate name from all repositories with case-insensitive.
* 4️⃣ **Dumping** Gixor dumps the specified boilerplates to the standard output.
  * Gixor finds the `.gitignore` file in the current directory if it exists,
    and reads it until the first boilerplate, which named prologue.
  * Then, Gixor write the prologue and the found boilerplates to the specified output.
* 5️⃣ **Finalization** Gixor writes the configuration file into the loaded location, if the configuration file was updated.

## 📛 The repository name

The repository name is the short identifier of the boilerplate repository.
And then, the user of Gixor should specify the boilerplate name with the repository name, such as `default/rust`.
However, the repository name is optional, and the user can specify the boilerplate name only, such as `rust`.
In this case, Gixor searches the boilerplate name from all repositories with case-insensitive
and dumps the first found boilerplate.

## 🌱 Prologue of `.gitignore`

The prologue is the first part of the `.gitignore` file in the current directory.
The first part means the content in the `.gitignore` file until the first boilerplate.
Gixor writes prologue to the output before the specified boilerplates.

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