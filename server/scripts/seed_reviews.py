import os, time, random, psycopg2
from datetime import datetime, timezone, timedelta

ADJECTIVES = [
    "Quiet", "Swift", "Calm", "Bold", "Bright", "Gentle", "Lazy",
    "Clever", "Grumpy", "Happy", "Sleepy", "Brave", "Witty", "Proud",
]
ANIMALS = [
    "Fox", "Owl", "Bear", "Wolf", "Deer", "Hawk", "Frog",
    "Crow", "Lynx", "Mole", "Newt", "Puma", "Toad", "Wren",
    "Ibis", "Vole", "Bison", "Crane", "Stoat", "Finch",
]

def gen_names(n):
    pairs = [f"{a} {b}" for a in ADJECTIVES for b in ANIMALS]
    return random.sample(pairs, n)

NAMES = gen_names(30)

COURSE_TEXTS = [
    "this course is amazing",
    "i think this course could be better",
    "i love this course",
    "this course sucks",
    "pretty decent, learned a lot",
    "not worth the workload honestly",
    "would recommend to anyone",
    "the assignments were all over the place",
    "one of my favourites so far",
    "okay course, nothing special",
    "hard but worth it",
    "the grading felt unfair",
    "really enjoyed this one",
    "could use better structure",
    "surprisingly good",
]

ADVISOR_TEXTS = [
    "great advisor, very supportive",
    "communication could definitely be better",
    "i learned a lot working with them",
    "not the easiest to work with but you grow",
    "would recommend as an advisor",
    "decent, nothing extraordinary",
    "very approachable and helpful",
    "hard to get responses sometimes",
    "one of the better advisors i've had",
    "okay experience overall",
]

CROCKFORD = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"

def new_ulid():
    ts = int(time.time() * 1000)
    t = ""
    for _ in range(10):
        t = CROCKFORD[ts % 32] + t
        ts //= 32
    return t + "".join(random.choices(CROCKFORD, k=16))

def clamp(v, lo, hi):
    return max(lo, min(hi, v))

def rand_axes(keys):
    tier = random.choices([1, 2, 3, 4, 5], weights=[1, 2, 4, 2, 1])[0]
    return {k: clamp(tier + random.randint(-1, 1), 1, 5) for k in keys}

def rand_ts():
    now = datetime.now(timezone.utc)
    delta = timedelta(seconds=random.randint(0, 2 * 365 * 24 * 3600))
    dt = now - delta
    return dt.strftime("%Y-%m-%d %H:%M:%S+00")

def sql_str(v):
    if v is None:
        return "NULL"
    return "'" + str(v).replace("'", "''") + "'"

conn = psycopg2.connect(os.environ["DATABASE_URL"])
cur = conn.cursor()

cur.execute("SELECT o.id, c.name FROM offerings o JOIN courses c ON c.id = o.course_id")
offerings = cur.fetchall()  # [(offering_id, course_name), ...]

cur.execute("SELECT id, name FROM faculty")
faculty = cur.fetchall()  # [(faculty_id, faculty_name), ...]

cur.close()
conn.close()

# build user id map
user_ids = {name: new_ulid() for name in NAMES}

user_rows = []
for name, uid in user_ids.items():
    cas = name.lower().replace(" ", "") + "@students.iiit.ac.in"
    user_rows.append(f"({sql_str(uid)}, {sql_str(cas)}, {sql_str(name)}, false, {sql_str(rand_ts())})")

course_review_rows = []
for offering_id, course_name in offerings:
    picked = random.sample(NAMES, 10)
    for name in picked:
        uid = user_ids[name]
        axes = rand_axes(["difficulty", "teaching", "grading", "content", "workload"])
        body = random.choice(COURSE_TEXTS)
        anon = "true"
        ts = rand_ts()
        course_review_rows.append(
            f"({sql_str(new_ulid())}, {sql_str(uid)}, {sql_str(offering_id)}, {anon}, "
            f"{axes['difficulty']}, {axes['teaching']}, {axes['grading']}, {axes['content']}, {axes['workload']}, "
            f"{sql_str(body)}, NULL, {sql_str(ts)})"
        )

advisor_review_rows = []
for faculty_id, faculty_name in faculty:
    picked = random.sample(NAMES, 10)
    for name in picked:
        uid = user_ids[name]
        axes = rand_axes(["research", "availability", "mentorship", "support", "workload"])
        body = random.choice(ADVISOR_TEXTS)
        anon = "true"
        ts = rand_ts()
        advisor_review_rows.append(
            f"({sql_str(new_ulid())}, {sql_str(uid)}, {sql_str(faculty_id)}, {anon}, "
            f"{axes['research']}, {axes['availability']}, {axes['mentorship']}, {axes['support']}, {axes['workload']}, "
            f"{sql_str(body)}, NULL, {sql_str(ts)})"
        )

seed_user_ids = ", ".join(sql_str(uid) for uid in user_ids.values())

print("BEGIN;")
print()
print("-- wipe previous seed reviews")
print(f"DELETE FROM course_reviews WHERE user_id IN ({seed_user_ids});")
print(f"DELETE FROM advisor_reviews WHERE user_id IN ({seed_user_ids});")
print()
print("-- users")
print("INSERT INTO users (id, cas_id, display_name, is_admin, created_at) VALUES")
print(",\n".join(user_rows))
print("ON CONFLICT DO NOTHING;")
print()
print("-- course reviews")
print("INSERT INTO course_reviews (id, user_id, offering_id, anonymous, difficulty, teaching, grading, content, workload, body, edited_at, created_at) VALUES")
print(",\n".join(course_review_rows) + ";")
print()
print("-- advisor reviews")
print("INSERT INTO advisor_reviews (id, user_id, faculty_id, anonymous, research, availability, mentorship, support, workload, body, edited_at, created_at) VALUES")
print(",\n".join(advisor_review_rows) + ";")
print()
print("COMMIT;")
