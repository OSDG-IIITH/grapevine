/                       landing — search bar only

/courses                browse courses (filter by type)
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
