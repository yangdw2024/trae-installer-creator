#!/usr/bin/env node
const { spawn } = require("child_process");
const path = require("path");

const exe = path.join(__dirname, "win32-x64", "forgecc.exe");
const child = spawn(exe, process.argv.slice(2), { stdio: "inherit" });
child.on("exit", (code) => process.exit(code ?? 1));
child.on("error", (err) => {
  console.error(`Failed to launch forgecc.exe: ${err.message}`);
  process.exit(1);
});
