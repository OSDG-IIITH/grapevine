/login                  public landing / login / register page
                          - wordmark, short description
                          - username/password form with toggle
                          - "Continue with CAS" button
                          - redirects to / or /verify on successful session creation

/verify                 one-time CAS verification page
                          - for local unverified accounts
                          - explains anonymity guarantees, initiates verification flow

/                       authenticated landing/home — search bar only (gated)

/courses                browse courses (filter by type) (gated)
/courses/[id]           course detail
                          - name, description, type, ratings block
                          - offerings as tabs (first tab = all offerings combined)
                          - each tab contains reviews for that offering

/faculty                browse faculty
/faculty/[id]           faculty detail
                          - two tabs: "As Instructor" (course reviews) and "As Advisor" (advisor reviews)

/labs                   browse labs
/labs/[id]              lab overview
                          - faculty members listed
                          - aggregated advisor reviews for all faculty in the lab

/review                 polymorphic review page
                          - user selects entity type + entity to review
                          - if navigated from a course/faculty page, pre-selects that entity

/admin                  all admin tools — flagged reviews, hide/dismiss actions

Global Layout Guard (web/src/routes/+layout.svelte):
- Checks authentication status via `getMe()` on mount.
- If not logged in: redirects to `/login` (unless already on `/login`).
- If logged in but unverified: redirects to `/verify` (unless on `/verify` or `/login`).
- If verified: redirects away from `/login` and `/verify` to `/`.
- Replaces individual soft-guards on pages like `/profile` and `/review`.
