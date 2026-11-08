# Tasks: Company Google sign-in

**Input**: Design documents from `specs/002-company-google-sign-in/`
**Prerequisites**: plan.md, spec.md

**Tests**: Fixture coverage is included. It is not a Google login and it is not the parent browser check.

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Company sign-in (US1)

**Goal**: A verified `aifluxx.com` Google account can open the workspace entry. Any other Google account cannot.

**Independent Test**: The domain check and the HTTP fixture in `crates/company_factory`.

- [x] T001 [US1] Accept only a verified hosted domain and email domain in `crates/company_factory/src/oidc.rs`
- [x] T002 [US1] Require `FACTORY_ALLOWED_DOMAIN=aifluxx.com`, a Google client secret, and a loopback bind in `crates/company_factory/src/config.rs`
- [x] T003 [US1] Return 503 from `/login` and name `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, and `FACTORY_ALLOWED_DOMAIN` in `crates/company_factory/src/http.rs`
- [x] T004 [P] [US1] Cover `aifluxx.com`, `aifluxx.co.za`, and Gmail in `crates/company_factory/src/oidc_tests.rs` and `crates/company_factory/tests/sign_in.rs`

## Phase 2: Private page (US2)

**Goal**: The public name serves this page only. The headless terminal stays private.

**Independent Test**: `deploy/factory` publishes no port except the process bind `127.0.0.1:8792`.

- [x] T005 [US2] Record the loopback procedure in `deploy/factory/README.md` and `deploy/factory/compose.yaml`
- [ ] T006 [US2] Parent creates a new Google web client, without editing an existing client, and signs in from a local browser on this revision

## Phase 3: Fork check

**Purpose**: Give this pull request a check without starting Warp CI.

- [x] T007 Add `fmt`, Clippy, and `cargo test -p company_factory` in `.github/workflows/company-factory.yml`
- [ ] T008 The Company factory check on this pull request is green

## Notes

- T004 and the workflow use a fixture identity provider. They do not satisfy T006.
- Upstream `.github/workflows/ci.yml` stays limited to `warpdotdev`.
