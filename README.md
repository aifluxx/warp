<a href="https://www.warp.dev">
    <img width="1024" alt="Warp Agentic Development Environment product preview" src="https://github.com/user-attachments/assets/9976b2da-2edd-4604-a36c-8fd53719c6d4" />
</a>
&nbsp;
<p align="center">
  <a href="https://warp.dev/factories"><img height="20" alt="Built with Warp" src="https://raw.githubusercontent.com/warpdotdev/brand-assets/main/Github/Built-With-Warp-Export@2x.png" /></a>
</p>

<p align="center">
  <a href="https://www.warp.dev">Website</a>
  ·
  <a href="https://www.warp.dev/code">Code</a>
  ·
  <a href="https://www.warp.dev/agents">Agents</a>
  ·
  <a href="https://www.warp.dev/terminal">Terminal</a>
  ·
  <a href="https://www.warp.dev/drive">Drive</a>
  ·
  <a href="https://docs.warp.dev">Docs</a>
  ·
  <a href="https://www.warp.dev/blog/how-warp-works">How Warp Works</a>
</p>

> [!NOTE]
> OpenAI is the founding sponsor of the new, open-source Warp repository, and the new agentic management workflows are powered by GPT models.

<h1></h1>

## About

[Warp](https://www.warp.dev) is an agentic development environment, born out of the terminal. Use Warp's built-in coding agent, or bring your own CLI agent (Claude Code, Codex, Gemini CLI, and others).

## Installation

You can [download Warp](https://www.warp.dev/download) and [read our docs](https://docs.warp.dev/) for platform-specific instructions.

## Warp Contributions Overview Dashboard

Explore [build.warp.dev](https://build.warp.dev) to:
- Watch thousands of [Warp Factory](warp.dev/factories) agents triage issues, write specs, implement changes, and review PRs
- View top contributors and in-flight features
- Track your own issues with GitHub sign-in
- Click into active agent sessions in a web-compiled Warp terminal

## Automate development with Warp Factories

This repository is driven by [Warp Factories](https://warp.dev/factories): open, flexible infrastructure for teams to build cloud software factories of their own.

Warp Factories are defined in code and easy to deploy on any model or harness, with evals, benchmarks, and self-improvement built in. [Get early access](https://warp.dev/factories) or [book a demo](https://warp.dev/get-started) for a tailored walk-through of the platform. Eligible teams can receive up to $10,000 in free factory usage.

## Licensing

Warp's UI framework (the `warpui_core` and `warpui` crates) are licensed under the [MIT license](LICENSE-MIT).

The rest of the code in this repository is licensed under the [AGPL v3](LICENSE-AGPL).

## Open Source & Contributing

Warp's client codebase is open source and lives in this repository. We welcome community contributions and have designed a lightweight workflow to help new contributors get started. For the full contribution flow, read our [CONTRIBUTING.md](CONTRIBUTING.md) guide.

> [!TIP]
> **Chat with contributors and the Warp team** in the [`#oss-contributors`](https://warpcommunity.slack.com/archives/C0B0LM8N4DB) Slack channel — a good place for ad-hoc questions, design discussion, and pairing with maintainers. New here? [Join the Warp Slack community](https://go.warp.dev/join-preview) first, then jump into `#oss-contributors`.

### Issue to PR

Before filing, [search existing issues](https://github.com/warpdotdev/warp/issues?q=is%3Aissue+is%3Aopen+sort%3Areactions-%2B1-desc) for your bug or feature request. If nothing exists, [file an issue](https://github.com/warpdotdev/warp/issues/new/choose) using our templates. Security vulnerabilities should be reported privately as described in [CONTRIBUTING.md](CONTRIBUTING.md#reporting-security-issues).

Once filed, a Warp maintainer reviews the issue and may apply a readiness label: [`ready-to-spec`](https://github.com/warpdotdev/warp/issues?q=is%3Aissue+is%3Aopen+label%3Aready-to-spec) signals the design is open for contributors to spec out, and [`ready-to-implement`](https://github.com/warpdotdev/warp/issues?q=is%3Aissue+is%3Aopen+label%3Aready-to-implement) signals the design is settled and code PRs are welcome. Anyone can pick up a labeled issue — mention **@oss-maintainers** on an issue if you'd like it considered for a readiness label.

### Building the Repo Locally

To build and run Warp from source:

```bash
./script/bootstrap   # platform-specific setup
./script/run         # build and run Warp
./script/presubmit   # fmt, clippy, and tests
```

See [AGENTS.md](AGENTS.md) for the full engineering guide, including coding style, testing, and platform-specific notes.

## Self-hosting the headless TUI

This repository is the Warp **client**. The files under [`deploy/vps/`](deploy/vps/) run the headless open-source terminal, `warp-tui-oss`, in Docker. That is the piece that can run on a Linux server. The setup was written for the AIFluxx Hostinger VPS `srv1307450.hstgr.cloud` (Ubuntu, Docker, the `aifluxx-dev` and `aifluxx-enterprise-dev` stacks already running). It does not deploy anything by itself.

### What this stack runs

- One container, `warp-oss-tui`, on the Compose project `warp-oss`.
- The `warp-tui-oss` binary (headless Warp Agent CLI) behind `ttyd`, bound to `127.0.0.1:8791` on the host and port `7681` in the container.
- A named volume, `warp-oss-tui-state`, for TUI config and local data.
- A private network, `warp-oss-net`.

Runtime cap in the example env: **1 GB RAM**, **1 CPU**, at most two web-terminal clients. The container name, network, volume, and port are deliberately different from the existing aifluxx stacks, and the port is not 80, 443, 3000, 5173, 5432, 6379, 8000, or 8080.

`docker/linux-dev/` is a desktop build container (SSH and X11). It is not this service.

### What cannot be self-hosted from this repo

- **warp-server** (accounts, GraphQL, the agent backend, Warp Drive, real-time sync, session sharing) is not in this repository. The OSS TUI is compiled against Warp's hosted service (`app.warp.dev` and the related RTC and session endpoints). A `WARP_API_KEY` logs this client into that hosted service. It does not start a replacement server.
- **Oz** (`oz.warp.dev`, ambient agents) is not in this repository.
- **`WITH_LOCAL_SERVER` / `SERVER_ROOT_URL`** in [AGENTS.md](AGENTS.md) only redirects a developer build at a warp-server checkout you already have. This fork does not contain that server, and the VPS compose file does not set those variables.
- **The GUI desktop app** (`./script/run`, `cargo run`) needs a display and GPU drivers. It is the wrong process for this headless box and is not part of the compose stack.
- Login, cloud agents, Drive sync, and team features stay unavailable until the client can reach Warp's cloud with a real Warp account. An empty VPS will show the TUI login screen and will not gain those features by being rebuilt.

### Resources

| Phase | RAM | Disk | CPU |
| --- | --- | --- | --- |
| Image build (`release-tui`) | Several GB free. Thin LTO can spike higher. Use `CARGO_BUILD_JOBS=1` or `WARP_CARGO_PROFILE=dev` if the kernel OOM-killer fires. | The verification build used about 11 GB of BuildKit cache and produced a 566 MB image. Do this on a machine with room, not on a host that is already at 66% RAM with 76 GB free. | 2 rustc jobs by default |
| Running container | Limit 1 GB, reservation 256 MB. Idle use after the welcome screen was about 16 MB. | The 566 MB image plus a few MB on `warp-oss-tui-state` after first launch. Loading the image does not need the build cache. | Limit 1 |

Do not build the image on `srv1307450` while aifluxx is up. Build it somewhere with at least 8 GB of free RAM and 40 GB of free disk, then load the image on the VPS.

### Install and run

On the build machine, from this repository:

```bash
cd deploy/vps
cp .env.example .env
docker compose build
docker save warp-oss-tui:1 | gzip > warp-oss-tui-1.tar.gz
```

Copy `warp-oss-tui-1.tar.gz` and this `deploy/vps/` directory to the VPS. On the VPS, as a user who can run Docker:

```bash
# Confirm the names and port are free before creating anything.
docker ps -a --format '{{.Names}}' | grep -E 'warp-oss|aifluxx' || true
ss -ltn | grep -E ':8791\b' || true

gunzip -c warp-oss-tui-1.tar.gz | docker load
cd deploy/vps
cp .env.example .env
docker compose up -d --no-build
docker compose ps
docker compose exec warp-tui /usr/local/bin/warp-oss-entrypoint version
```

`docker compose ps` should show `warp-oss-tui` healthy. From a laptop:

```bash
ssh -L 8791:127.0.0.1:8791 <user>@srv1307450.hstgr.cloud
```

Open `http://127.0.0.1:8791`. The page is a terminal running `warp-tui-oss`. With `WARP_API_KEY` left blank, the first screen says **Welcome to Warp**, **Log in with Warp**, and **Copy login URL**. That login talks to Warp's hosted service.

The container has no desktop Secret Service. The TUI logs that once, then uses its on-disk credential fallback under the `warp-oss-tui-state` volume. A source build with no `GIT_RELEASE_TAG` prints version `v0.0.0.0.0.0`.

Attach over SSH without the browser:

```bash
docker compose exec warp-tui /usr/local/bin/warp-oss-entrypoint tui
```

Stop it without touching the other stacks:

```bash
docker compose stop
```

`docker compose down` removes this stack's container and network. Add `-v` only when you also mean to delete the `warp-oss-tui-state` volume.

Set `TTYD_USERNAME` and `TTYD_PASSWORD` in `.env` before exposing the port past localhost. The committed template leaves them empty and publishes on `127.0.0.1` only.

## Company factory slice

`company_factory` is a separate company sign-in page. It is not Warp Factories, not the Warp contributions dashboard at `build.warp.dev`, and not the headless TUI above. It does not sign a person into Warp's hosted account service, and it does not open a shell.

Run it from this repository:

```bash
export FACTORY_SESSION_SECRET="$(openssl rand -base64 32)"
export FACTORY_PUBLIC_URL=http://127.0.0.1:8792
export FACTORY_BIND=127.0.0.1:8792
export FACTORY_REPO_ID=warp
export FACTORY_REPO_ROOT="$PWD"
export OIDC_ISSUER=https://your-company-identity-issuer
export OIDC_CLIENT_ID=your-client-id
export OIDC_CLIENT_SECRET=your-client-secret
cargo run -p company_factory
```

Open `http://127.0.0.1:8792`. Empty names in [`deploy/factory/.env.example`](deploy/factory/.env.example) are the settings still required. The process refuses to start without the session secret, public URL, repository id, and repository directory. Without `OIDC_ISSUER` and `OIDC_CLIENT_ID`, the page stays up and every workspace request is refused. The identity provider must issue RS256 ID tokens. No provider is included.

The personal model key is typed into the page after sign-in. It stays in memory for that sign-in only. There is no shared model key. The repository action lists visible names in `FACTORY_REPO_ROOT` and does not call a model provider. Restarting the process ends every sign-in. Nothing here changes DNS, certificates, or the public name `factory.aifluxx.co.za`.

## Joining the Team

Interested in joining the team? See our [open roles](https://www.warp.dev/careers).

## Support and Questions

1. See our [docs](https://docs.warp.dev/) for a comprehensive guide to Warp's features.
2. Join our [Slack Community](https://go.warp.dev/join-preview) to connect with other users and get help from the Warp team — contributors hang out in [`#oss-contributors`](https://warpcommunity.slack.com/archives/C0B0LM8N4DB).
3. Try our [Preview build](https://www.warp.dev/download-preview) to test the latest experimental features.
4. Mention **@oss-maintainers** on any issue to escalate to the team — for example, if you encounter problems with the automated agents.

## Code of Conduct

We ask everyone to be respectful and empathetic. Warp follows the [Code of Conduct](CODE_OF_CONDUCT.md). To report violations, email warp-coc at warp.dev.

## Open Source Dependencies

We'd like to call out a few of the [open source dependencies](https://docs.warp.dev/help/licenses) that have helped Warp to get off the ground:

- [Tokio](https://github.com/tokio-rs/tokio)
- [NuShell](https://github.com/nushell/nushell)
- [Fig Completion Specs](https://github.com/withfig/autocomplete)
- [Warp Server Framework](https://github.com/seanmonstar/warp)
- [Alacritty](https://github.com/alacritty/alacritty)
- [Hyper HTTP library](https://github.com/hyperium/hyper)
- [FontKit](https://github.com/servo/font-kit)
- [Core-foundation](https://github.com/servo/core-foundation-rs)
- [Smol](https://github.com/smol-rs/smol)
