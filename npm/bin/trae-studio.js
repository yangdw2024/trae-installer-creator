#!/usr/bin/env node
const { spawn } = require("child_process");
const path = require("path");

const exe = path.join(__dirname, "win32-x64", "trae-studio.exe");
const child = spawn(exe, process.argv.slice(2), { detached: true, stdio: "ignore" });
child.unref();
