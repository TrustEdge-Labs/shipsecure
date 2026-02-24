# Requirements: ShipSecure

**Defined:** 2026-02-24
**Core Value:** Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow.

## v1.7 Requirements

Requirements for v1.7 Frontend Polish. Each maps to roadmap phases.

### Touch Targets

- [ ] **TOUCH-01**: Header nav links and buttons have minimum 44px touch target height
- [ ] **TOUCH-02**: Logo link has expanded hit area via padding (p-2 -m-2 pattern)

### Accessibility

- [ ] **A11Y-01**: Scan form checkbox is visually larger (w-5 h-5) with cursor-pointer and padding wrapper
- [ ] **A11Y-02**: Dashboard table rows use proper link pattern that doesn't duplicate links for screen readers

### Hydration

- [ ] **HYDR-01**: React hydration mismatch investigated and resolved (Clerk appearance prop or suppressHydrationWarning)

### UX

- [ ] **UX-01**: Scan form email field has explanatory copy setting expectation ("We'll email your results")
- [ ] **UX-02**: Dashboard polls for active scan updates at 5-10 second intervals via router.refresh() or client-side polling

### Design Consistency

- [ ] **DESIGN-01**: --card-radius design token defined and applied consistently across all card/panel elements
- [ ] **DESIGN-02**: PageContainer shared max-width layout component used on all pages

### Analytics

- [ ] **ANLYT-01**: Plausible script tag includes data-domain="shipsecure.ai" attribute

## Future Requirements

None — this is a focused polish milestone.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Backend API changes | Frontend-only milestone, no Rust changes |
| New pages or features | Polish existing UI only |
| Dark mode redesign | Already working via prefers-color-scheme from v1.3 |
| Test updates for UI changes | CI will validate; add tests only if existing ones break |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| TOUCH-01 | Phase 36 | Pending |
| TOUCH-02 | Phase 36 | Pending |
| A11Y-01 | Phase 36 | Pending |
| A11Y-02 | Phase 36 | Pending |
| HYDR-01 | Phase 37 | Pending |
| UX-01 | Phase 37 | Pending |
| UX-02 | Phase 37 | Pending |
| DESIGN-01 | Phase 38 | Pending |
| DESIGN-02 | Phase 38 | Pending |
| ANLYT-01 | Phase 38 | Pending |

**Coverage:**
- v1.7 requirements: 10 total
- Mapped to phases: 10
- Unmapped: 0

---
*Requirements defined: 2026-02-24*
*Last updated: 2026-02-24 after roadmap creation — all 10 requirements mapped*
