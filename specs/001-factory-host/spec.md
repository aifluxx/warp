# Feature Specification: Company address for the Warp client

**Feature Branch**: `001-factory-host`  
**Created**: 2026-11-08  
**Status**: In review  
**Input**: User description: "On 8 Nov 2026 at 08:13 SAST Greg said to build, after being told the fork client still depends on Warp's hosted login and factory. Deliver the first usable company-hosted development slice toward the company factory address. Keep it distinct from Warp's proprietary factories. Use real sign-in, sign-out, and a protected workspace entry. Require each person's own model key. Refuse unsigned requests. Do not bypass Warp's login, do not pretend that login succeeded, and do not publish a shell."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Sign in to the company slice (Priority: P1)

An authorised person opens the company development slice, signs in with the company's own identity service, and sees a protected workspace entry for the one repository the operator assigned. The page identifies itself as the company slice.

**Why this priority**: Nothing else is safe to show before sign-in works.

**Independent Test**: On a computer the operator controls, sign in through the company identity service and reach the workspace entry. Warp's public factory page does not count as this sign-in.

**Acceptance Scenarios**:

1. **Given** the person is not signed in, **When** they request the workspace entry, **Then** the slice refuses them and offers company sign-in.
2. **Given** the person completes company sign-in, **When** the workspace entry loads, **Then** they see the assigned repository and a statement that this is not Warp's factory product and not Warp's contributions dashboard.
3. **Given** the person is signed in, **When** they sign out, **Then** the workspace entry refuses them again.

---

### User Story 2 - Use their own key on one repository (Priority: P1)

A signed-in person supplies their own model key and runs the safe repository action. Another signed-in person cannot reuse that key. A request that names a different repository is refused. If no personal key is present, the action is refused and no shared company key is substituted.

**Why this priority**: A shared key or a cross-repository read would mix people's work.

**Independent Test**: Sign in as two people. Set a key for the first only. The first person's action succeeds for the assigned repository and fails for any other repository. The second person's action fails until they set their own key, and the first person's key is not shown to them.

**Acceptance Scenarios**:

1. **Given** a signed-in person has not supplied a key, **When** they run the repository action, **Then** it is refused.
2. **Given** they supply their own key, **When** they run the action for the assigned repository, **Then** they receive only that repository's safe inventory.
3. **Given** they name a different repository, **When** they run the action, **Then** it is refused.
4. **Given** another signed-in person has not supplied a key, **When** they run the action, **Then** it is refused even though the first person already supplied one.

---

### User Story 3 - Keep the public name closed (Priority: P2)

The company factory name stays unpublished. No shell is offered to an unsigned visitor. Existing company sites and Truest stay as they are.

**Why this priority**: The slice can be reviewed on a private entry before any public name is attached.

**Independent Test**: The company factory name still fails to resolve. The private entry refuses unsigned workspace requests. Truest is unchanged.

**Acceptance Scenarios**:

1. **Given** this slice exists, **When** someone opens the public company factory name, **Then** it does not answer as this slice.
2. **Given** an unsigned visitor reaches the private entry, **When** they look for a shell, **Then** none is offered.

---

### Edge Cases

- Company identity settings are missing. Sign-in does not succeed and names the missing settings without inventing an account.
- The identity service returns an untrusted or altered proof. No session is created.
- The slice is restarted. Previous sign-ins are no longer accepted.
- A person asks for a file outside the assigned repository, or for a hidden secret name. The inventory does not include it.
- Warp's own factory page and Warp's contributions dashboard are open in another window. They are not this product.
- A shared model key is present in the environment. It is not used to pass the personal-key check.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The slice MUST let a person sign in through the company's existing identity service and MUST refuse the workspace entry when that sign-in is absent.
- **FR-002**: The slice MUST let a signed-in person sign out, after which the workspace entry refuses them.
- **FR-003**: The slice MUST NOT treat a stand-in, a bypass of Warp's login, or a missing identity service as a successful sign-in.
- **FR-004**: A repository action MUST run only for the one assigned repository, and only after the signed-in person has supplied their own model key.
- **FR-005**: The slice MUST NOT use a shared or platform model key when the person's own key is absent, and MUST NOT reveal one person's key to another person.
- **FR-006**: The workspace entry MUST be visibly distinct from Warp's factory product and from Warp's contributions dashboard.
- **FR-007**: The slice MUST NOT offer a shell to an unsigned visitor.
- **FR-008**: The public company factory name, existing company sites, Truest, and Warp's hosted account service MUST stay unchanged by this slice.

### Key Entities

- **Company session**: The signed-in person, the assigned repository, and whether that person has supplied their own key. It ends on sign-out or when the slice restarts.
- **Assigned repository**: The single repository the operator named for this slice.
- **Personal model key**: A key the signed-in person brings. It is not a Warp login and it is not a shared company key.
- **Company identity service**: The organisation's existing sign-in service. This repository does not include that service's secrets.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An unsigned request for the workspace entry is refused, and a person who signs in through the company identity service can open that entry.
- **SC-002**: After sign-out, the same person is refused again.
- **SC-003**: With two signed-in people, the repository action succeeds only for the person who supplied their own key, and only for the assigned repository.
- **SC-004**: A reviewer can tell this slice from Warp's factory product and from Warp's contributions dashboard in one reading of the entry page.
- **SC-005**: The public company factory name remains unpublished, and no shell is available from either that name or the private entry before sign-in.
- **SC-006**: A parent-computer signed-in browser test is still required before this slice is called working. A cloud-only run does not meet that test.

## Assumptions

- Greg's 08:13 SAST instruction approves building this company slice. It does not approve a public shell, a DNS change, or a claim that Warp's factory product is running.
- Warp's hosted account service remains necessary for Warp's own agent, Drive, and factories. This slice does not provide or bypass that service.
- The company identity service already exists outside this repository. Its addresses and secrets are supplied by the operator and are not stored here.
- Sign-in does not survive a restart. No database change is part of this slice.
- The safe action lists visible names in the assigned repository. It does not send the personal key to a model provider.
- Platform context stays on dubai-agents DEV-215, pull request 200. This repository keeps spec 001 and does not use Dubai issue numbers.
