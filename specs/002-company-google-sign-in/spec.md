# Feature Specification: Company Google sign-in for the factory page

**Feature Branch**: `002-company-google-sign-in`  
**Created**: 2026-11-08  
**Status**: In review  
**Input**: User description: "Prepare the company factory page at factory.aifluxx.co.za for Google Workspace sign-in restricted to the aifluxx.com hosted domain, and a VPS procedure. Do not publish it from this session. Do not accept any Google account. Do not use Warp's account, a Warp API key, or an existing production sign-in client. Keep the headless terminal private."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Sign in with the company Google account (Priority: P1)

A person at the company opens the factory page and signs in with a new company Google client. They reach the workspace entry only when the signed-in address is a verified `aifluxx.com` address. The public site name is not that domain.

**Why this priority**: A public page that accepts every Google account would mix outsiders into the company workspace.

**Independent Test**: Sign in with a company address and reach the workspace entry. Sign in with a personal Google account and be refused.

**Acceptance Scenarios**:

1. **Given** company sign-in is configured, **When** a person with a verified `aifluxx.com` address completes Google sign-in, **Then** the workspace entry opens.
2. **Given** the same page, **When** a personal Google account, or an account on another domain, completes the Google prompt, **Then** no session is created.
3. **Given** the person is signed in, **When** they sign out, **Then** the workspace entry refuses them.

---

### User Story 2 - Publish only the page (Priority: P2)

The operator can place the page on the company server behind the company name. The headless terminal is not part of that address. Existing sites stay as they are.

**Why this priority**: The public name must not become a shell.

**Independent Test**: The company name reaches the factory page. The terminal port is not that page. Other company sites still answer.

**Acceptance Scenarios**:

1. **Given** the page is published, **When** an unsigned visitor opens the company name, **Then** they see the factory page and no shell.
2. **Given** the terminal is still installed, **When** someone opens the company name, **Then** they do not get the terminal.

---

### Edge Cases

- The company domain setting is missing. Sign-in does not start, and the sign-in page names the missing settings.
- An address at `aifluxx.co.za` is not an `aifluxx.com` company account.
- The Google proof is altered, unsigned, or for a different application. No session is created.
- The hosted domain and the email domain disagree. No session is created.
- An address in a look-alike domain is refused.
- The page restarts. Previous sign-ins end.
- Warp's own factory page remains a different product.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Sign-in MUST use a new company Google client and MUST require a verified email in the `aifluxx.com` hosted domain.
- **FR-002**: A Google account outside that hosted domain MUST NOT receive a session. An existing production sign-in client MUST stay unchanged.
- **FR-003**: The page MUST NOT treat a Warp login or a Warp API key as company sign-in.
- **FR-004**: Unsigned visitors MUST NOT receive a shell or the workspace inventory.
- **FR-005**: The public company name MUST serve this page only, and MUST NOT serve the headless terminal.
- **FR-006**: Existing company sites, Truest, and other sign-in clients MUST stay unchanged.

### Key Entities

- **Company domain**: `aifluxx.com`, the hosted domain on the company Google account. The site name `factory.aifluxx.co.za` is a different name.
- **Company session**: A signed-in company address. It ends on sign-out or restart.
- **Factory page**: The company page. It is not Warp's factory product and not the terminal.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An `aifluxx.com` Google account can open the workspace entry, and a personal Google account cannot.
- **SC-002**: An unsigned request for the workspace is refused.
- **SC-003**: The company name, once published, shows this page and not the terminal.
- **SC-004**: A parent-computer signed-in browser test of this revision is still required before it is called working. Publishing the name is a later parent action.

## Assumptions

- The company already has Google Workspace. This work does not create a Google organization or edit another sign-in client.
- The operator supplies the Google client id, client secret, and company domain. None of those secrets are stored in the repository.
- Warp's hosted account service remains separate.
- No database change is included.
- Platform context stays on dubai-agents DEV-215, pull request 200. This repository uses spec 002.
