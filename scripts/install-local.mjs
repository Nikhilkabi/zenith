import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

const root = process.cwd();
const build = spawnSync("npx", ["tauri", "build", "--no-bundle"], {
  cwd: root,
  stdio: "inherit",
  shell: true,
  env: { ...process.env, CARGO_TARGET_DIR: undefined },
});
if (build.status !== 0) process.exit(build.status ?? 1);

const src = join(root, "src-tauri", "target", "release", "bucket.exe");
const destDir = join(process.env.LOCALAPPDATA ?? "", "Programs", "Bucket");
mkdirSync(destDir, { recursive: true });
const dest = join(destDir, "Bucket.exe");
const zenith = join(destDir, "Zenith.exe");
function install() {
  copyFileSync(src, dest);
  copyFileSync(src, zenith);
}
try {
  install();
} catch {
  spawnSync("taskkill", ["/IM", "Bucket.exe", "/F"], { stdio: "ignore", shell: true });
  spawnSync("taskkill", ["/IM", "bucket.exe", "/F"], { stdio: "ignore", shell: true });
  spawnSync("taskkill", ["/IM", "Zenith.exe", "/F"], { stdio: "ignore", shell: true });
  install();
}
console.log(`Installed ${zenith}`);
