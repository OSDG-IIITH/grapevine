Auth
  GET  /auth/login                  initiates direct CAS authentication
  GET  /auth/callback               handles CAS login callback
  POST /auth/register              registers a local user (body: { username, password })
  POST /auth/login/local           logs in a local user (body: { username, password })
  GET  /auth/verify                initiates CAS verification check for local user
  GET  /auth/verify/callback       handles verification return from CAS (sets user as verified)
  POST /auth/logout                logs out and invalidates the session (triggers CAS logout if CAS user)
  GET  /me                         returns current user info
                                     Response shape: { id, display_name, is_admin, verified, username (string | null), auth_method ("cas" | "local") }
  PATCH /me                        updates user display name (body: { display_name })
  GET  /me/reviews                 returns reviews created by current user
                                     Response shape: { course: CourseReview[], advisor: AdvisorReview[] }


Note on Access Control:
- The entire application (Courses, Course Reviews, Faculty, Advisor Reviews, Labs, Admin) is protected by a verification gate middleware.
- Unauthenticated requests to protected endpoints return 401 Unauthorized.
- Logged-in but unverified local accounts return 403 Forbidden on protected endpoints.
- `/me`, `/auth/verify`, `/auth/verify/callback`, and `/auth/logout` are open to unverified logged-in sessions.

Courses
  GET  /courses                          list + search
  GET  /courses/:id                      detail + offerings (each with faculty list)
  GET  /courses/:id/reviews              reviews across all offerings
  GET  /offerings/:id/reviews            reviews for one offering

Course Reviews
  POST   /offerings/:id/reviews          create
  PATCH  /reviews/course/:id             edit
  DELETE /reviews/course/:id             delete
  POST   /reviews/course/:id/vote        body: { value: 1 or -1 }
  DELETE /reviews/course/:id/vote        remove current user's vote
  POST   /reviews/course/:id/flag        flag review
  review fields include: score (net), upvotes, downvotes, user_vote

Faculty
  GET  /faculty                          list + search
  GET  /faculty/:id                      detail + offerings taught

Advisor Reviews
  POST   /faculty/:id/reviews            create
  PATCH  /reviews/advisor/:id            edit
  DELETE /reviews/advisor/:id            delete
  POST   /reviews/advisor/:id/vote       body: { value: 1 or -1 }
  DELETE /reviews/advisor/:id/vote       remove current user's vote
  POST   /reviews/advisor/:id/flag       flag review
  review fields include: score (net), upvotes, downvotes, user_vote

Labs
  GET  /labs                             list + search
  GET  /labs/:id                         detail + faculty in lab + aggregated advisor reviews

Admin
  GET    /admin/flags                    list all flagged reviews
  POST   /admin/flags/:id/dismiss        dismiss flag without deleting review
  POST   /admin/flags/:id/delete-review  delete flagged review and cascade its flags
  GET    /admin/export                   export database tables to JSON seeding format

