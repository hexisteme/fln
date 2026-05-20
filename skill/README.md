# FLN Claude Code skill

Install:

```bash
mkdir -p ~/.claude/skills
ln -sf "$(pwd)/skill" ~/.claude/skills/fln
```

After symlinking, Claude Code auto-loads the skill on FLN-relevant prompts
(thesis / falsifier / ledger / anchor / 영구 의사결정 등).
