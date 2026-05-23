Auth
  GET  /auth/login
  GET  /auth/callback
  POST /auth/logout

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
  POST   /admin/flags/:id/resolve        hide or dismiss
