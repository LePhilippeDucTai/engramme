# Installing the engramme-capture skill

The capture skill is a single file, [`SKILL.md`](SKILL.md), in the Agent Skills
format. The same file works for Claude Code, OpenAI Codex and GitHub Copilot.

## Before you start

- A vault: a directory with `notes/` and `inbox/` (see
  [the format](../docs/format.md)).
- `ENGRAMME_DIR`, the absolute path of the vault, set in the environment the
  agents start from.
  - macOS (zsh): `printf '\nexport ENGRAMME_DIR="$HOME/vault"\n' >> ~/.zshenv`
    (the leading newline protects a last line without a final newline)
  - Windows (PowerShell, untested):
    `[Environment]::SetEnvironmentVariable('ENGRAMME_DIR', "$HOME\vault", 'User')`,
    then reopen your terminals and VS Code.
- Optionally, the `engramme` command (see [the README](../README.md)): the
  skill runs `engramme check` on each proposal it writes.

## Where each agent looks

| Agent | Skill directory | Global instructions |
| --- | --- | --- |
| Claude Code | `~/.claude/skills/engramme-capture/` | `~/.claude/CLAUDE.md` |
| OpenAI Codex | `~/.agents/skills/engramme-capture/` | `~/.codex/AGENTS.md` (or `AGENTS.override.md` if it exists) |
| GitHub Copilot CLI | `~/.copilot/skills/` or `~/.agents/skills/engramme-capture/` | `~/.copilot/copilot-instructions.md` |
| GitHub Copilot in VS Code | `~/.copilot/skills/`, `~/.claude/skills/` or `~/.agents/skills/` | `~/.copilot/copilot-instructions.md` |

**The destination directory must be named `engramme-capture/`, never `skill/`:**
the directory name has to match the `name` of the skill.

## Install the skill

Installing into `~/.claude/skills` and `~/.agents/skills` covers all four rows
of the table:

```sh install-skill
# Run from a clone of the repository, or replace skill/SKILL.md by the path of
# the SKILL.md downloaded from a release.
for dir in ~/.claude/skills ~/.agents/skills; do
  mkdir -p "$dir/engramme-capture"
  cp skill/SKILL.md "$dir/engramme-capture/SKILL.md"
done
```

On Windows, for Copilot (PowerShell, untested):

```powershell
New-Item -ItemType Directory -Force "$HOME\.agents\skills\engramme-capture" | Out-Null
Copy-Item skill\SKILL.md "$HOME\.agents\skills\engramme-capture\SKILL.md"
```

Reloading: Claude Code picks the change up by itself (run `/reload-skills` if
`~/.claude/skills/` did not exist when the session started); Codex detects it
by itself, otherwise restart Codex; in Copilot CLI, run `/skills reload`.

## Add the global instruction line

Agents load a skill at their own discretion and may forget it. One line in the
global instructions of each agent reminds them:

```text
At the end of a task, consider the engramme-capture skill.
```

```sh add-instruction
line='At the end of a task, consider the engramme-capture skill.'
for file in ~/.claude/CLAUDE.md ~/.codex/AGENTS.md ~/.copilot/copilot-instructions.md; do
  mkdir -p "$(dirname "$file")"
  grep -qxF "$line" "$file" 2>/dev/null || printf '\n%s\n' "$line" >> "$file"
done
```

On Windows, for Copilot (PowerShell, untested):

```powershell
$line = 'At the end of a task, consider the engramme-capture skill.'
$file = "$HOME\.copilot\copilot-instructions.md"
New-Item -ItemType Directory -Force (Split-Path $file) | Out-Null
if (-not ((Test-Path $file) -and (Select-String -Quiet -SimpleMatch $line $file))) {
  Add-Content $file "`n$line"
}
```

Codex reads `~/.codex/AGENTS.override.md` instead of `~/.codex/AGENTS.md` when
it exists: put the line there in that case. For the Local harness of VS Code,
enable `chat.useClaudeMdFile` and keep the line in `~/.claude/CLAUDE.md`
(documented), or create a user instructions file with "Chat: New Instructions
File" (not verified to apply to every request).

## Let the agent reach the vault (optional)

Agents usually work inside a project directory; the vault is elsewhere.

- Claude Code: add the vault to `~/.claude/settings.json`:

```json
{"permissions": {"additionalDirectories": ["/absolute/path/to/vault"]}}
```

- Codex: allow writes to the inbox in `~/.codex/config.toml`, with an absolute
  path and no variable:

```toml
[sandbox_workspace_write]
writable_roots = ["/absolute/path/to/vault/inbox"]
```

  Codex passes its environment to the commands it runs, unless
  `shell_environment_policy` restricts it (`core`, `none`, filters): keep
  `ENGRAMME_DIR` in that case. That the default `all` passes it is inferred,
  not documented.
- Copilot: accept the write when it asks for it (not verified).

## Optional: a Stop hook for Claude Code

Each time Claude stops, if `ENGRAMME_DIR` is a directory, this hook sends a
reminder of the skill that makes Claude continue once. It costs a short extra
turn per answer: remove it if it bothers you. It does not loop: it stays silent
when `stop_hook_active` is true, and Claude Code also caps such continuations at
8. It needs `sh` (macOS, Linux; Git Bash on Windows).

Add this to `~/.claude/settings.json`; if a `hooks` key already exists, add the
entry to its `Stop` array:

```json stop-hook
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "grep -Eq '\"stop_hook_active\"[[:space:]]*:[[:space:]]*true' || [ ! -d \"$ENGRAMME_DIR\" ] || echo '{\"hookSpecificOutput\":{\"hookEventName\":\"Stop\",\"additionalContext\":\"If the task you just finished used non-trivial knowledge worth remembering and you have not proposed it yet, use the engramme-capture skill. Otherwise end your turn without comment.\"}}'"
          }
        ]
      }
    ]
  }
}
```

To check it: `/hooks` lists it, and running the command by hand,
`printf '{"stop_hook_active":false}' | ENGRAMME_DIR="$ENGRAMME_DIR" sh -c "<command>"`,
prints the JSON of the reminder.

## Check the installation

- Claude Code: `engramme-capture` appears in the `/` menu.
- Codex: `/skills` lists it, or mention it as `$engramme-capture`.
- Copilot CLI: `/skills list`, then `/skills info engramme-capture`.
- VS Code: "Chat: Open Customizations".
- Then `engramme check "$ENGRAMME_DIR"` validates the vault.

## Uninstall

Delete `~/.claude/skills/engramme-capture/` and
`~/.agents/skills/engramme-capture/`, remove the instruction line from the three
instruction files, and remove the Stop hook from `~/.claude/settings.json`.

## Not verified

- The PowerShell commands.
- A possible double listing in VS Code when the skill is both in
  `~/.claude/skills/` and in `~/.agents/skills/`.
- Copilot permission prompts for writes outside the workspace.
- `~/.codex/skills/`, which codex-cli 0.163.0 reads but which is not
  documented: do not install the skill there as well, or it appears twice.
- Other agents, such as zcode.

## Sources (checked 2026-10-21)

- <https://code.claude.com/docs/en/skills>: personal skills in
  `~/.claude/skills/<skill-name>/SKILL.md`; changes apply without a restart
  (`/reload-skills` if `~/.claude/skills/` did not exist when the session
  started); `name` gives the `/` menu entry.
- <https://code.claude.com/docs/en/memory>: `~/.claude/CLAUDE.md`, personal
  preferences for all projects, loaded in every session.
- <https://code.claude.com/docs/en/hooks>: `Stop` takes no matcher; shell
  commands run through `sh -c` on macOS and Linux and Git Bash on Windows;
  `stop_hook_active`; `hookSpecificOutput.additionalContext`; at most 8
  continuations.
- <https://code.claude.com/docs/en/settings-reference>:
  `permissions.additionalDirectories`, an array of directory paths.
- <https://learn.chatgpt.com/docs/build-skills> (redirected from
  <https://developers.openai.com/codex/skills>): user scope
  `$HOME/.agents/skills`; `name` and `description` required; changes detected
  automatically, otherwise restart Codex; `/skills` or a `$` mention.
- <https://learn.chatgpt.com/docs/agent-configuration/agents-md> (redirected
  from <https://developers.openai.com/codex/guides/agents-md>): `AGENTS.md` in
  `CODEX_HOME` (default `~/.codex`), replaced by `AGENTS.override.md` if it
  exists; at most 32 KiB.
- <https://learn.chatgpt.com/docs/config-file/config-reference>:
  `sandbox_workspace_write.writable_roots`; `shell_environment_policy.inherit`.
- <https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/add-skills>:
  `~/.copilot/skills` or `~/.agents/skills`; `/skills list`, `/skills reload`,
  `/skills info`.
- <https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/add-custom-instructions>:
  `$HOME/.copilot/copilot-instructions.md`; `COPILOT_HOME`.
- <https://code.visualstudio.com/docs/copilot/customization/agent-skills>:
  `~/.copilot/skills/`, `~/.claude/skills/`, `~/.agents/skills/`; `name` must
  match the directory.
- <https://code.visualstudio.com/docs/copilot/customization/custom-instructions>:
  `~/.copilot/copilot-instructions.md` for personal always-on instructions;
  `~/.claude/CLAUDE.md` read when `chat.useClaudeMdFile` is enabled.
- <https://agentskills.io/specification>: constraints on `name` and
  `description`.
