use std::collections::HashMap;
use serde::Deserialize;
use sqlx::PgPool;
use ulid::Ulid;

#[derive(Deserialize)]
struct Lab {
    name: String,
    shortname: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct Faculty {
    name: String,
    slug: String,
    #[serde(default)]
    labs: Vec<String>,
    bio: Option<String>,
}

#[derive(Deserialize)]
struct Course {
    code: String,
    name: String,
    #[serde(rename = "type")]
    kind: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct Offering {
    course: String,
    season: String,
    year: i16,
    faculty: Vec<String>,
}

fn read<T: for<'de> Deserialize<'de>>(path: &str) -> T {
    let s = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("[seed] cannot read {path}: {e}"));
    serde_json::from_str(&s)
        .unwrap_or_else(|e| panic!("[seed] cannot parse {path}: {e}"))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&url).await.expect("failed to connect to database");

    let labs: Vec<Lab> = read("seed/labs.json");
    let faculty: Vec<Faculty> = read("seed/faculty.json");
    let courses: Vec<Course> = read("seed/courses.json");
    let offerings: Vec<Offering> = read("seed/offerings.json");

    // labs
    println!("[seed] inserting {} labs...", labs.len());
    let mut lab_ids: HashMap<String, String> = HashMap::new();
    for lab in &labs {
        let id = Ulid::new().to_string();
        let row = sqlx::query!(
            "INSERT INTO labs (id, name, shortname, description)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (shortname) DO UPDATE SET name = EXCLUDED.name, description = EXCLUDED.description
             RETURNING id, shortname",
            id, lab.name, lab.shortname, lab.description
        )
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|e| panic!("[seed] failed inserting lab {}: {e}", lab.shortname));
        lab_ids.insert(row.shortname, row.id);
    }

    // faculty
    println!("[seed] inserting {} faculty...", faculty.len());
    let mut faculty_ids: HashMap<String, String> = HashMap::new();
    for f in &faculty {
        let id = Ulid::new().to_string();
        let row = sqlx::query!(
            "INSERT INTO faculty (id, name, slug, bio)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name, bio = EXCLUDED.bio
             RETURNING id, slug",
            id, f.name, f.slug, f.bio
        )
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|e| panic!("[seed] failed inserting faculty {}: {e}", f.slug));
        faculty_ids.insert(row.slug.clone(), row.id.clone());

        for lab_short in &f.labs {
            let lab_id = lab_ids.get(lab_short)
                .unwrap_or_else(|| panic!("[seed] ERROR: lab shortname \"{lab_short}\" in faculty.json not found in labs.json"))
                .clone();
            sqlx::query!(
                "INSERT INTO faculty_labs (faculty_id, lab_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                row.id, lab_id
            )
            .execute(&pool)
            .await
            .unwrap_or_else(|e| panic!("[seed] failed inserting faculty_labs for {}: {e}", f.slug));
        }
    }

    // courses
    println!("[seed] inserting {} courses...", courses.len());
    let mut course_ids: HashMap<String, String> = HashMap::new();
    for c in &courses {
        let id = Ulid::new().to_string();
        let row = sqlx::query!(
            "INSERT INTO courses (id, code, name, type, description)
             VALUES ($1, $2, $3, $4::course_type, $5)
             ON CONFLICT (code) DO UPDATE SET name = EXCLUDED.name, type = EXCLUDED.type, description = EXCLUDED.description
             RETURNING id, code",
            id, c.code, c.name, c.kind as _, c.description
        )
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|e| panic!("[seed] failed inserting course {}: {e}", c.code));
        course_ids.insert(row.code, row.id);
    }

    // offerings
    println!("[seed] inserting {} offerings...", offerings.len());
    for o in &offerings {
        let course_id = course_ids.get(&o.course)
            .unwrap_or_else(|| panic!("[seed] ERROR: course code \"{}\" in offerings.json not found in courses.json", o.course))
            .clone();
        let id = Ulid::new().to_string();
        let row = sqlx::query!(
            "INSERT INTO offerings (id, course_id, season, year)
             VALUES ($1, $2, $3::offering_season, $4)
             ON CONFLICT (course_id, season, year) DO UPDATE SET course_id = EXCLUDED.course_id
             RETURNING id",
            id, course_id, o.season as _, o.year
        )
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|e| panic!("[seed] failed inserting offering {}/{}/{}: {e}", o.course, o.season, o.year));
        let offering_id = row.id;
        for slug in &o.faculty {
            let faculty_id = faculty_ids.get(slug)
                .unwrap_or_else(|| panic!("[seed] ERROR: faculty slug \"{slug}\" in offerings.json not found in faculty.json"))
                .clone();
            sqlx::query!(
                "INSERT INTO offering_faculty (offering_id, faculty_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                offering_id, faculty_id
            )
            .execute(&pool)
            .await
            .unwrap_or_else(|e| panic!("[seed] failed inserting offering_faculty for {slug}: {e}"));
        }
    }

    println!("[seed] done.");
}
