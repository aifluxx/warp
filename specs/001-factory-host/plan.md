# Implementation Plan: Company factory slice

**Branch**: `cursor/company-factory-slice-5f9e` | **Date**: 2026-11-08 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/001-factory-host/spec.md`

## Summary

Add a company-owned sign-in slice, separate from Warp Factories and from the headless TUI. People sign in with a standard identity-provider discovery flow configured by the operator, get an in-memory session, and can sign out. The protected workspace entry lists visible names in one assigned repository only after that person supplies their own model key. Unsigned requests are refused. The public name `factory.aifluxx.co.za` is not changed.

Warp's hosted account service, agent, Drive, and proprietary factories stay a residual dependency. This plan does not call them, bypass them, or claim they are running locally.

## Technical Context

**Language/Version**: Rust edition 2024, same workspace as the Warp client  
**Primary Dependencies**: axum and reqwest already in the workspace. RS256 ID-token checks use `jsonwebtoken`. Session cookies use HMAC-SHA256.  
**Storage**: Process memory only. No database and no migration.  
**Testing**: `cargo nextest run -p company_factory` plus Clippy for that package. Identity tests use a labelled local fixture provider that signs real RS256 tokens. They are not a production login.  
**Target Platform**: Linux, loopback HTTP for development  
**Project Type**: Small workspace crate plus a binary  
**Performance Goals**: One operator and a handful of signed-in people on a development entry  
**Constraints**: No public shell, no DNS change, no production deploy, no invented identity secrets, no platform model-key fallback, no external model call, AGPL-3.0-only like the non-UI crates  
**Scale/Scope**: One assigned repository and one private entry on `127.0.0.1:8792`

## Constitution Check

The Warp repository has no SpecKit constitution file. The gates that do apply are the repository rules: AGPL for this crate, no secrets in git, no unauthenticated shell, and no claim that a cloud browser test finishes the parent gate.

The dubai-agents constitution registries named by the canonical plan template are not in this repository. They were not copied and not invented.

## Project Structure

### Documentation (this feature)

```text
specs/001-factory-host/
├── spec.md
├── plan.md
└── checklists/requirements.md
```

### Source Code (repository root)

```text
crates/company_factory/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── lib.rs
    ├── config.rs
    ├── session.rs
    ├── oidc.rs
    ├── repo.rs
    ├── byok.rs
    └── http.rs

deploy/factory/.env.example
```

**Structure Decision**: A new `company_factory` crate under `crates/`, which the workspace already includes via `crates/*`. It does not link the GUI, the TUI, or `warp_server_client`, so it cannot silently reuse Warp's device login.

## Residual dependency

Company sign-in works only when the operator sets `OIDC_ISSUER`, `OIDC_CLIENT_ID`, and, for a confidential client, `OIDC_CLIENT_SECRET`, plus `FACTORY_SESSION_SECRET`, `FACTORY_PUBLIC_URL`, `FACTORY_REPO_ID`, and `FACTORY_REPO_ROOT`. The identity provider must issue RS256 ID tokens. No provider is bundled.

Warp's cloud (`app.warp.dev`, Oz, Drive, and Warp Factories) is unchanged and is not contacted. The TUI on `127.0.0.1:8791` remains a separate terminal and is not mounted here.

The personal model key is stored in memory for the signed-in person and is required before the repository inventory runs. It is not sent to a model provider.

## Skill & Tool Mapping *(mandatory — Constitution VII)*

Registries named by the canonical template (`backend/tools/__init__.py`, `backend/skills/__init__.py`, `backend/agents/registry.py`) are not part of this repository. No skill or tool from those registries is available to reuse. No new dubai-agents skill is added. Platform integration remains DEV-215 in dubai-agents pull request 200; this plan does not edit that repository.

### Existing Skills Reused

| Functional Requirement | Skill | Module | Justification |
| --- | --- | --- | --- |
| — | — | — | No dubai-agents skill registry exists in this repository |

### Existing Tools Reused

| Capability Needed | Tool | Justification |
| --- | --- | --- |
| — | — | No dubai-agents tool registry exists in this repository |

### New Skills Required *(only if no existing skill fits)*

| Skill Name | Domain Module | Required Tools | Why Existing Skills Don't Fit |
| --- | --- | --- | --- |
| — | — | — | None. This slice is a Warp repository crate, not a dubai-agents skill. |

### New Tools Required *(only if no existing tool fits)*

| Tool Name | Module | Why Existing Tools Don't Fit |
| --- | --- | --- |
| — | — | None added outside this crate. |

### Agent Architecture

| Agent | Lines Target | Skills Delegated To | Notes |
| --- | --- | --- | --- |
| — | — | — | No agent runtime is added. `/speckit.specify` is still not an installed command. |

## Complexity Tracking

No constitution file to violate. In-memory sessions are deliberate: a database migration was not approved.
