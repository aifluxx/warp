# Implementation Plan: Company Google sign-in

**Branch**: `cursor/company-factory-google-oidc-5f9e` | **Date**: 2026-11-08 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/002-company-google-sign-in/spec.md`

## Summary

Keep the company factory page. Require Google's standard identity issuer, RS256 tokens, and a verified email whose hosted domain matches `FACTORY_ALLOWED_DOMAIN`. Add a loopback-only compose file for the VPS. Do not publish the terminal, change DNS, or create a Google client from this session.

## Technical Context

**Language/Version**: Rust edition 2024  
**Primary Dependencies**: Existing `company_factory` crate  
**Storage**: Process memory. No database.  
**Testing**: `cargo test -p company_factory` and Clippy for that package, also run by `.github/workflows/company-factory.yml` on `aifluxx/warp` pull requests. The outsider account is a labelled fixture token, not a production Google login.  
**Target Platform**: Linux container on the existing VPS, published only on `127.0.0.1:8792`  
**Project Type**: Small workspace crate plus `deploy/factory`  
**Performance Goals**: One company page for the operator's team  
**Constraints**: No public shell, no Warp API key, no new Google organization, no DNS from this session, AGPL-3.0-only  
**Scale/Scope**: One domain and one repository

## Constitution Check

No SpecKit constitution file is in this repository. Dubai-agents registries named by the plan template are not in this repository and were not copied.

## Project Structure

### Documentation (this feature)

```text
specs/002-company-google-sign-in/
├── spec.md
├── plan.md
├── tasks.md
└── checklists/requirements.md
```

### Source Code (repository root)

```text
crates/company_factory/
deploy/factory/Dockerfile
deploy/factory/compose.yaml
deploy/factory/.env.example
deploy/factory/README.md
```

**Structure Decision**: Extend `company_factory`. Do not add a second service for the terminal.

## Residual dependency

The parent creates one new Google web client in the existing company cloud project. The factory redirect URLs are `http://127.0.0.1:8792/callback` and `https://factory.aifluxx.co.za/callback`. `FACTORY_ALLOWED_DOMAIN` is `aifluxx.com`. The process listens on `127.0.0.1:8792` only. Existing production clients stay unchanged. Warp's hosted agent, Drive, and proprietary factories are not contacted.

## Skill & Tool Mapping *(mandatory — Constitution VII)*

No dubai-agents skill or tool registry exists in this repository. None are added. DEV-215 remains in dubai-agents pull request 200 and is not edited.

### Existing Skills Reused

| Functional Requirement | Skill | Module | Justification |
| --- | --- | --- | --- |
| — | — | — | No dubai-agents skill registry exists here |

### Existing Tools Reused

| Capability Needed | Tool | Justification |
| --- | --- | --- |
| — | — | No dubai-agents tool registry exists here |

### New Skills Required *(only if no existing skill fits)*

| Skill Name | Domain Module | Required Tools | Why Existing Skills Don't Fit |
| --- | --- | --- | --- |
| — | — | — | None. This is a Warp crate change. |

### New Tools Required *(only if no existing tool fits)*

| Tool Name | Module | Why Existing Tools Don't Fit |
| --- | --- | --- |
| — | — | None. |

### Agent Architecture

| Agent | Lines Target | Skills Delegated To | Notes |
| --- | --- | --- | --- |
| — | — | — | No agent runtime is added. |

## Complexity Tracking

No database. Sessions stay in memory because a migration was not approved.
