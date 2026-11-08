# Feature Specification: Company address for the Warp client

**Feature Branch**: `001-factory-host`  
**Created**: 2026-11-08  
**Status**: Blocked  
**Input**: User description: "Greg approved hosting, a company address, and secure sign-in for the Warp fork at the company factory address on 8 Nov 2026. Publish only what this product can actually sign in to. Do not present a terminal stand-in as Warp's factory product. If that sign-in cannot be provided by this product, stop and report the gap before publishing a substitute."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Open the company address and sign in (Priority: P1)

An authorised person opens the company factory address and signs in with the sign-in this Warp client already supports. After sign-in they can use only the functions that client already provides.

**Why this priority**: A public address without a real sign-in would expose the wrong product.

**Independent Test**: From a computer Greg designates, open the company address, complete the client's own sign-in, and confirm the result is that client. The vendor factory marketing page does not count.

**Acceptance Scenarios**:

1. **Given** the company address is published, **When** an authorised person signs in with the client's supported sign-in, **Then** they reach that client and no claim is made that Warp's factory product is running there.
2. **Given** the person is not signed in, **When** they open the company address, **Then** they do not receive a shell or terminal.

---

### User Story 2 - Stop rather than publish a stand-in (Priority: P1)

If this product cannot complete the supported sign-in because the account service belongs to a different product, the company address stays unpublished. The gap is reported. A terminal wrapper is not put at that address in its place.

**Why this priority**: Greg approved hosting only for the real client. He did not approve a substitute.

**Independent Test**: Read this specification and the availability record. The company name does not resolve to a terminal, and the written gap names the missing account service.

**Acceptance Scenarios**:

1. **Given** the account service is not part of this product, **When** a request is made to publish the address, **Then** publication does not proceed and the missing service is stated.
2. **Given** Warp's own factory page is available on Warp's site, **When** someone checks the company address, **Then** that vendor page is not described as the company's running factory.

---

### Edge Cases

- The company name is removed or was never published. Visitors see a name failure, not a shell.
- An existing company site on another name keeps working.
- Truest is not changed.
- Someone proposes to publish the terminal first and add sign-in later. That proposal is refused.
- The vendor factory page loads on Warp's own site. That does not satisfy sign-in on the company address.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The company address MUST show only the Warp client this product already is, after that client's own sign-in.
- **FR-002**: The product MUST NOT add a new account system, a new factory service, or a new backend to imitate Warp's factory product.
- **FR-003**: A person who is not signed in MUST NOT receive a shell or terminal at the company address.
- **FR-004**: When the supported sign-in cannot be completed because the account service is a separate product, the address MUST stay unpublished and that gap MUST be reported before any substitute is published.
- **FR-005**: Existing company sites and Truest MUST stay as they are.
- **FR-006**: A reviewer MUST be able to tell the company address apart from Warp's public factory page.

### Key Entities *(include if feature involves data)*

- **Company address**: The name Greg approved for this client. It is unpublished until sign-in can succeed.
- **Warp client session**: The session this product already knows how to start after its supported sign-in.
- **Hosted account**: The account held by Warp's separate service. This product does not create or store that account.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A designated reviewer completes the client's own sign-in on the company address from their own computer, or the address is still unpublished and the written gap names the missing account service.
- **SC-002**: An unsigned visitor cannot open a shell from the company address. The check is a failed name lookup or a refusal before any terminal is offered.
- **SC-003**: A side-by-side check shows Warp's public factory page and the company address are not the same running product.
- **SC-004**: Existing company sites still answer as they did before this decision, and Truest is unchanged.

## Assumptions

- Greg's approval on 8 Nov 2026 covers hosting, the company address, and secure sign-in for the client this repository already is.
- The same approval does not cover a substitute terminal or a claim that Warp's factory product is running.
- The account service required for that sign-in is not in this repository. Publishing the address cannot create it.
- On 8 Nov 2026 the company factory name had no public address. That state must remain until SC-001 is met by a real sign-in, not by a stand-in.
- Platform context lives with dubai-agents DEV-215, pull request 200. This specification uses the next number in this repository, 001, and does not reuse that platform number.
- This specification is blocked. Planning must not turn it into a public terminal.
