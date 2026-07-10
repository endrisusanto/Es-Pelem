#!/usr/bin/env bash
# ponytail: Clean release script checking paths dynamically to support both desktop and mobile configs
set -euo pipefail

bump="${1:-patch}"

if [ -n "$(git status --porcelain)" ]; then
  git add -A
  git commit -m "chore: update before release"
fi

version="$(node - "$bump" <<'JS'
const bump = process.argv[2];
const fs = require("fs");

const pkgPath = "package.json";
if (!fs.existsSync(pkgPath)) {
  console.error("Error: package.json not found");
  process.exit(1);
}

const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
let [major, minor, patch] = (pkg.version || "0.1.0").split(".").map(Number);

if (bump === "major") {
  major++; minor = 0; patch = 0;
} else if (bump === "minor") {
  minor++; patch = 0;
} else if (bump === "patch") {
  patch++;
} else if (/^\d+\.\d+\.\d+$/.test(bump)) {
  [major, minor, patch] = bump.split(".").map(Number);
} else {
  console.error("use: release.sh [patch|minor|major|x.y.z]");
  process.exit(1);
}

pkg.version = `${major}.${minor}.${patch}`;
fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");

const tauriPath = "src-tauri/tauri.conf.json";
if (fs.existsSync(tauriPath)) {
  const tauri = JSON.parse(fs.readFileSync(tauriPath, "utf8"));
  tauri.version = pkg.version;
  fs.writeFileSync(tauriPath, JSON.stringify(tauri, null, 2) + "\n");
}

const cargoPath = "src-tauri/Cargo.toml";
if (fs.existsSync(cargoPath)) {
  let cargo = fs.readFileSync(cargoPath, "utf8");
  cargo = cargo.replace(/^version = ".*"$/m, `version = "${pkg.version}"`);
  fs.writeFileSync(cargoPath, cargo);
}

const gradlePath = "android/app/build.gradle";
if (fs.existsSync(gradlePath)) {
  let android = fs.readFileSync(gradlePath, "utf8");
  android = android
    .replace(/versionCode \d+/, `versionCode ${major * 10000 + minor * 100 + patch}`)
    .replace(/versionName ".*"/, `versionName "${pkg.version}"`);
  fs.writeFileSync(gradlePath, android);
}

console.log(pkg.version);
JS
)"

if [ -f "src-tauri/Cargo.toml" ]; then
  cargo check --manifest-path src-tauri/Cargo.toml
fi

if git rev-parse "v${version}" >/dev/null 2>&1; then
  echo "tag v${version} already exists" >&2
  exit 1
fi

# Track files dynamically
GIT_FILES=("package.json")
if [ -f "src-tauri/Cargo.toml" ]; then GIT_FILES+=("src-tauri/Cargo.toml"); fi
if [ -f "src-tauri/tauri.conf.json" ]; then GIT_FILES+=("src-tauri/tauri.conf.json"); fi
if [ -f "android/app/build.gradle" ]; then GIT_FILES+=("android/app/build.gradle"); fi

git add "${GIT_FILES[@]}"
git commit -m "chore: release v${version}" || true
git tag "v${version}"
git push
git push origin "v${version}"

echo "released v${version}"
