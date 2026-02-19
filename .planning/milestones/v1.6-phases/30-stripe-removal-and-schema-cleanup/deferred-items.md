# Deferred Items - Phase 30

## Pre-existing Test Failures (out of scope)

### Header.test.tsx - 4 failures (pre-existing from Phase 29)

**Files:** `frontend/__tests__/components/Header.test.tsx`

**Error:** `@clerk/nextjs: SignedOut can only be used within the <ClerkProvider /> component`

**Root cause:** Phase 29 (29-02) added Clerk's `SignedOut`/`SignedIn` components to the Header. The test file renders `<Header />` without wrapping it in `<ClerkProvider>`, causing Clerk to throw. This failure existed before Phase 30 began (verified by git stash comparison).

**Impact:** None on runtime behavior. Build succeeds, E2E tests work. Only unit test environment affected.

**Recommended fix:** Wrap Header renders in Header.test.tsx and dark-mode.test.tsx's Header tests with a mocked `<ClerkProvider>` or mock the Clerk hooks used by SignedOut/SignedIn.

**Discovered:** During Phase 30 Task 2 vitest run
**Status:** Deferred - pre-existing, out of scope for Phase 30
