# Playtests shared through the repo

Recordings live in the user folder (`%APPDATA%/Aegis/emberwake/playtests/<id>.json`, note images in `<id>/`). To
share one, use the Scene Runner's **Playtests**, then **Copy to repo**: the recording and its note images go to
`inbox/<id>/`. Push it on any branch and tell the agent the id.

| Folder | Holds |
| --- | --- |
| `inbox/<id>/` | user recordings copied here (`<id>.json` with its `note_<n>.jpg` images beside it) |
| `repro/` | agent repro recordings (`<id>.json`, or a folder like the inbox) |

Both folders are listed in the Playtests browser with the badge `repo`. Replay one headless with
`aegis replay games/emberwake/playtests/inbox/<id>/<id>.json` (`engine/README.md`, "Playtest recordings and replay").
The planner removes handled inbox folders in a cleanup pass (git history keeps them).
