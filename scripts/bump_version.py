import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKAGE_JSON = ROOT / "package.json"
TAURI_CONF = ROOT / "src-tauri" / "tauri.conf.json"


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def bump_patch(version: str) -> str:
    parts = version.split(".")
    if len(parts) != 3 or not all(p.isdigit() for p in parts):
        raise ValueError(f"Unsupported version format: {version}")
    major, minor, patch = (int(p) for p in parts)
    return f"{major}.{minor}.{patch + 1}"


def main() -> None:
    package = read_json(PACKAGE_JSON)
    tauri = read_json(TAURI_CONF)

    current = tauri.get("version") or package.get("version")
    if not isinstance(current, str):
        raise ValueError("Version not found in tauri.conf.json or package.json")

    next_version = bump_patch(current)
    package["version"] = next_version
    tauri["version"] = next_version

    write_json(PACKAGE_JSON, package)
    write_json(TAURI_CONF, tauri)

    print(f"Bumped version: {current} -> {next_version}")


if __name__ == "__main__":
    main()
