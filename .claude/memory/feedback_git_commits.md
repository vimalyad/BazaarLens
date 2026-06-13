---
name: Git Commit Message Rules
description: Never mention Claude or add Co-Authored-By Claude in any git commit message
type: feedback
---

Do not reference Claude in any git commit message — no "Co-Authored-By: Claude", no "Generated with Claude Code", no Claude mention anywhere in commit subject or body.

**Why:** Teammates share this repo and commit history should be clean, professional, and author-neutral. Claude attribution in commits is unwanted.

**How to apply:** Every commit made in this project must omit the standard Claude co-author trailer. Write commit messages in conventional commit format only (`type(scope): description`).
