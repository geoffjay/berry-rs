# Ollama

These instructions are for Ollama on macOS. This is how I run it on my machine, it may be different on yours.

## Install

```sh
brew install ollama
```

## Configure

Create a plist file in `~/Library/LaunchAgents` with the following contents:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>com.ollama.serve</string>

    <key>ProgramArguments</key>
    <array>
      <string>/opt/homebrew/bin/ollama</string>
      <string>serve</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>EnvironmentVariables</key>
    <dict>
      <key>HOME</key>
      <string>/Users/geoff</string> <!-- change to your home directory -->

      <key>PATH</key>
      <string>/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin</string>

      <key>LANG</key>
      <string>en_US.UTF-8</string>

      <key>OLLAMA_HOST</key>
      <string>0.0.0.0:11434</string>

      <key>OLLAMA_KV_CACHE_TYPE</key>
      <string>q4_0</string>

      <key>OLLAMA_FLASH_ATTENTION</key>
      <string>1</string>
    </dict>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>/opt/homebrew/var/log/ollama/stdout.log</string>

    <key>StandardErrorPath</key>
    <string>/opt/homebrew/var/log/ollama/stderr.log</string>
  </dict>
</plist>
```

## Pre-requisites

```sh
mkdir /opt/homebrew/var/log/ollama
```

## Load

```sh
launchctl load ~/Library/LaunchAgents/com.ollama.serve.plist
```

## Start

```sh
launchctl start com.ollama.serve
```

## Stop

```sh
launchctl unload ~/Library/LaunchAgents/com.ollama.serve.plist
```
