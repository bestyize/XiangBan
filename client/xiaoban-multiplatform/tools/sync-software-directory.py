"""Refresh the app's checked-in software metadata from the reviewed forum catalog."""
import json
from pathlib import Path

mobile = Path(__file__).resolve().parents[1]
forum = mobile.parents[1] / "services/server-forum"
destination = mobile / "business/community-native/src/commonMain/composeResources/files"
destination.mkdir(parents=True, exist_ok=True)

manifest = json.loads((forum / "editorial/open-source-software.json").read_text())
entries = []
for project, source in manifest.items():
    entry = {field: source[field] for field in ("category", "platforms", "releaseRepo", "version", "language")}
    metadata = source.get("project", {})
    entry.update(project=project, name=metadata.get("name", ""), summary=metadata.get("summary", ""),
                 icon=metadata.get("icon_url", ""), stars=metadata.get("stars"), updatedAt=metadata.get("last_pushed_at"))
    entries.append(entry)

def write(name, value):
    (destination / name).write_text(json.dumps(value, ensure_ascii=False, separators=(",", ":")) + "\n")

write("software-directory.json", entries)

assets = json.loads((forum / "web/src/lib/softwareInitialAssets.json").read_text())
releases = {}
for entry in entries:
    repo = entry["releaseRepo"].lower()
    files = assets.get(repo, [])
    if files and entry["version"]:
        releases[entry["project"]] = {
            "tag_name": entry["version"],
            "assets": [{field: asset[field] for field in ("name", "browser_download_url", "size") if field in asset}
                       for asset in files],
        }
write("software-initial-releases.json", releases)
