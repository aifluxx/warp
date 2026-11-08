# Warp SpecKit numbering

The canonical specify workflow in `.agent/workflows/speckit.specify.md` is unchanged. In this repository its Jira step does not apply.

Before a new spec is planned or coded:

1. Run `script/next_spec_number`.
2. Use that number as `specs/NNN-short-name/`.
3. Do not use a `DEV-` prefix or a Dubai issue number.
4. Check `origin/master` and open pull requests before taking the number. The script does that when `origin/master` is fetched and `gh` can read this repository.
5. Keep the copied templates byte-identical to `.agent/PROVENANCE.md`.

A slash command named `/speckit.specify` is not installed by these files. Presence of the workflow file is not a configured runtime. Treat the runtime as configured only after a person runs the workflow in an agent session and the new spec number matches `script/next_spec_number`.

Platform context for the factory decision is dubai-agents DEV-215, pull request 200. That number is not the next number in this repository.
