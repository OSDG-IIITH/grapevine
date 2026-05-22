-- seed data for grapevine (IIIT Hyderabad)
-- run: psql $DATABASE_URL -f server/scripts/seed.sql

-- users (cas_id = iiit email)
INSERT INTO users (id, cas_id, display_name, is_admin) VALUES
  ('01JWA000000000000000000001', 'arjun.sharma@students.iiit.ac.in',   'Arjun Sharma',    false),
  ('01JWA000000000000000000002', 'priya.nair@students.iiit.ac.in',     'Priya Nair',      false),
  ('01JWA000000000000000000003', 'rohan.mehta@students.iiit.ac.in',    'Rohan Mehta',     false),
  ('01JWA000000000000000000004', 'ananya.iyer@students.iiit.ac.in',    'Ananya Iyer',     false),
  ('01JWA000000000000000000005', 'vikram.rao@students.iiit.ac.in',     'Vikram Rao',      false),
  ('01JWA000000000000000000006', 'divya.krishna@students.iiit.ac.in',  'Divya Krishna',   false),
  ('01JWA000000000000000000007', 'siddharth.jain@students.iiit.ac.in', 'Siddharth Jain',  false),
  ('01JWA000000000000000000008', 'neha.gupta@students.iiit.ac.in',     'Neha Gupta',      false),
  ('01JWA000000000000000000009', 'aditya.patel@students.iiit.ac.in',   'Aditya Patel',    false),
  ('01JWA00000000000000000000A', 'shreya.reddy@students.iiit.ac.in',   'Shreya Reddy',    false),
  ('01JWA00000000000000000000B', 'karan.bose@students.iiit.ac.in',     'Karan Bose',      false),
  ('01JWA00000000000000000000C', 'tanvi.desai@students.iiit.ac.in',    'Tanvi Desai',     false),
  ('01JWA00000000000000000000D', 'admin@iiit.ac.in',                   'Admin',           true);

-- labs
INSERT INTO labs (id, shortname, name, description) VALUES
  ('01JWB000000000000000000001', 'cvit',  'CVIT',  'Center for Visual Information Technology — computer vision, graphics, and image processing.'),
  ('01JWB000000000000000000002', 'ltrc',  'LTRC',  'Language Technologies Research Centre — NLP, speech, and Indian language computing.'),
  ('01JWB000000000000000000003', 'spcrc', 'SPCRC', 'Signal Processing and Communications Research Centre — wireless systems and signal processing.'),
  ('01JWB000000000000000000004', 'cstar', 'CSTAR', 'Centre for Security Theory and Algorithmic Research — cryptography and formal methods.'),
  ('01JWB000000000000000000005', 'serc',  'SERC',  'Software Engineering Research Centre — distributed systems and program analysis.'),
  ('01JWB000000000000000000006', 'kohli', 'KOHLI', 'Kohli Center on Intelligent Systems — AI, robotics, and HCI.'),
  ('01JWB000000000000000000007', 'idrbt', 'IDRBT', 'Institute for Development and Research in Banking Technology — fintech and security.');

-- faculty
INSERT INTO faculty (id, slug, name, bio, lab_id) VALUES
  ('01JWC000000000000000000001', 'jayanthi-sivaswamy',   'Prof. Jayanthi Sivaswamy',   'Computer vision and medical imaging researcher, former CVIT director.', '01JWB000000000000000000001'),
  ('01JWC000000000000000000002', 'cv-jawahar',           'Prof. C.V. Jawahar',          'Computer vision, document analysis, and scene understanding.', '01JWB000000000000000000001'),
  ('01JWC000000000000000000003', 'radhika-mamidi',       'Prof. Radhika Mamidi',        'Computational linguistics, dialogue systems, and NLP.', '01JWB000000000000000000002'),
  ('01JWC000000000000000000004', 'vasudeva-varma',       'Prof. Vasudeva Varma',        'Information retrieval and knowledge graphs.', '01JWB000000000000000000002'),
  ('01JWC000000000000000000005', 'raghu-babu-duggisetty','Prof. Raghu Babu Duggisetty', 'Digital communications and 5G systems.', '01JWB000000000000000000003'),
  ('01JWC000000000000000000006', 'kishore-kothapalli',   'Prof. Kishore Kothapalli',    'Parallel algorithms and GPU computing.', '01JWB000000000000000000005'),
  ('01JWC000000000000000000007', 'sharat-chandran',      'Prof. Sharat Chandran',       'Computer graphics and computational geometry.', '01JWB000000000000000000001'),
  ('01JWC000000000000000000008', 'suresh-purini',        'Prof. Suresh Purini',         'Compilers, program analysis, and high-performance computing.', '01JWB000000000000000000005'),
  ('01JWC000000000000000000009', 'girish-varma',         'Prof. Girish Varma',          'Machine learning theory and fairness in AI.', '01JWB000000000000000000006'),
  ('01JWC00000000000000000000A', 'sachin-lodha',         'Prof. Sachin Lodha',          'Applied cryptography and privacy-preserving systems.', '01JWB000000000000000000004'),
  ('01JWC00000000000000000000B', 'bapi-raju',            'Prof. Bapi Raju',             'Cognitive science and computational neuroscience.', '01JWB000000000000000000006'),
  ('01JWC00000000000000000000C', 'anoop-namboodiri',     'Prof. Anoop Namboodiri',      'Document recognition and visual analytics.', '01JWB000000000000000000001'),
  ('01JWC00000000000000000000D', 'kavita-vemuri',        'Prof. Kavita Vemuri',         'HCI and affective computing.', '01JWB000000000000000000006');

-- courses
INSERT INTO courses (id, code, name, description, type) VALUES
  ('01JWD000000000000000000001', 'CS0.101', 'Introduction to Programming',      'Fundamentals of programming using Python. Loops, functions, data structures.', 'core'),
  ('01JWD000000000000000000002', 'CS1.301', 'Data Structures and Algorithms',   'Arrays, linked lists, trees, graphs, sorting, searching.', 'core'),
  ('01JWD000000000000000000003', 'CS2.301', 'Operating Systems',                'Processes, scheduling, memory management, file systems.', 'core'),
  ('01JWD000000000000000000004', 'CS2.302', 'Computer Networks',                'OSI model, TCP/IP, routing, socket programming.', 'core'),
  ('01JWD000000000000000000005', 'CS3.401', 'Machine Learning',                 'Supervised and unsupervised learning, neural networks, evaluation.', 'stream'),
  ('01JWD000000000000000000006', 'CS3.402', 'Computer Vision',                  'Image processing, feature extraction, object detection and recognition.', 'stream'),
  ('01JWD000000000000000000007', 'CS3.403', 'Natural Language Processing',      'Text classification, seq2seq, transformers, language models.', 'stream'),
  ('01JWD000000000000000000008', 'CS2.303', 'Database Systems',                 'Relational model, SQL, query optimization, transactions.', 'core'),
  ('01JWD000000000000000000009', 'CS3.301', 'Algorithm Design',                 'Dynamic programming, greedy algorithms, network flows, NP-completeness.', 'breadth'),
  ('01JWD00000000000000000000A', 'CS3.302', 'Compiler Design',                  'Lexing, parsing, semantic analysis, code generation.', 'breadth'),
  ('01JWD00000000000000000000B', 'MA1.101', 'Linear Algebra',                   'Vectors, matrices, eigenvalues, applications in CS.', 'math'),
  ('01JWD00000000000000000000C', 'MA1.102', 'Probability and Statistics',       'Distributions, expectation, hypothesis testing, regression.', 'math'),
  ('01JWD00000000000000000000D', 'HS1.101', 'Technical Communication',          'Writing reports, presentations, professional communication.', 'hs'),
  ('01JWD00000000000000000000E', 'SC1.101', 'Quantitative Biology',             'Biological systems through a computational lens.', 'sci'),
  ('01JWD00000000000000000000F', 'CS4.501', 'Deep Learning',                    'CNNs, RNNs, transformers, generative models, training at scale.', 'open'),
  ('01JWD00000000000000000000G', 'CS4.502', 'Information Retrieval',            'Indexing, ranking, evaluation, modern neural retrieval.', 'open'),
  ('01JWD00000000000000000000H', 'CS4.503', 'Parallel Computing',               'Threads, CUDA, distributed memory, performance analysis.', 'open'),
  ('01JWD00000000000000000000J', 'CS4.504', 'Cryptography',                     'Symmetric and asymmetric crypto, protocols, provable security.', 'open'),
  ('01JWD00000000000000000000K', 'CS5.601', 'Human-Computer Interaction',       'User research, prototyping, usability evaluation.', 'bouquet'),
  ('01JWD00000000000000000000M', 'CS5.602', 'Cognitive Science',                'Memory, attention, perception — through CS and neuroscience.', 'bouquet');

-- offerings
INSERT INTO offerings (id, course_id, season, year) VALUES
  ('01JWE000000000000000000001', '01JWD000000000000000000001', 'M', 2022),
  ('01JWE000000000000000000002', '01JWD000000000000000000001', 'M', 2023),
  ('01JWE000000000000000000003', '01JWD000000000000000000001', 'M', 2024),
  ('01JWE000000000000000000004', '01JWD000000000000000000002', 'S', 2022),
  ('01JWE000000000000000000005', '01JWD000000000000000000002', 'S', 2023),
  ('01JWE000000000000000000006', '01JWD000000000000000000002', 'S', 2024),
  ('01JWE000000000000000000007', '01JWD000000000000000000003', 'M', 2022),
  ('01JWE000000000000000000008', '01JWD000000000000000000003', 'M', 2023),
  ('01JWE000000000000000000009', '01JWD000000000000000000004', 'S', 2022),
  ('01JWE00000000000000000000A', '01JWD000000000000000000004', 'S', 2023),
  ('01JWE00000000000000000000B', '01JWD000000000000000000005', 'M', 2022),
  ('01JWE00000000000000000000C', '01JWD000000000000000000005', 'M', 2023),
  ('01JWE00000000000000000000D', '01JWD000000000000000000005', 'S', 2024),
  ('01JWE00000000000000000000E', '01JWD000000000000000000006', 'S', 2022),
  ('01JWE00000000000000000000F', '01JWD000000000000000000006', 'S', 2023),
  ('01JWE00000000000000000000G', '01JWD000000000000000000007', 'M', 2023),
  ('01JWE00000000000000000000H', '01JWD000000000000000000007', 'M', 2024),
  ('01JWE00000000000000000000J', '01JWD000000000000000000008', 'S', 2022),
  ('01JWE00000000000000000000K', '01JWD000000000000000000008', 'S', 2023),
  ('01JWE00000000000000000000M', '01JWD000000000000000000009', 'M', 2022),
  ('01JWE00000000000000000000N', '01JWD000000000000000000009', 'M', 2023),
  ('01JWE00000000000000000000P', '01JWD00000000000000000000A', 'S', 2023),
  ('01JWE00000000000000000000Q', '01JWD00000000000000000000B', 'M', 2022),
  ('01JWE00000000000000000000R', '01JWD00000000000000000000B', 'M', 2023),
  ('01JWE00000000000000000000S', '01JWD00000000000000000000C', 'S', 2022),
  ('01JWE00000000000000000000T', '01JWD00000000000000000000C', 'S', 2023),
  ('01JWE00000000000000000000V', '01JWD00000000000000000000D', 'M', 2022),
  ('01JWE00000000000000000000W', '01JWD00000000000000000000D', 'M', 2023),
  ('01JWE00000000000000000000X', '01JWD00000000000000000000F', 'S', 2023),
  ('01JWE00000000000000000000Y', '01JWD00000000000000000000F', 'S', 2024),
  ('01JWE00000000000000000000Z', '01JWD00000000000000000000G', 'M', 2023),
  ('01JWE000000000000000000010', '01JWD00000000000000000000H', 'S', 2023),
  ('01JWE000000000000000000011', '01JWD00000000000000000000J', 'M', 2023),
  ('01JWE000000000000000000012', '01JWD00000000000000000000K', 'S', 2023),
  ('01JWE000000000000000000013', '01JWD00000000000000000000M', 'M', 2023);

-- offering_faculty
INSERT INTO offering_faculty (offering_id, faculty_id) VALUES
  -- CS0.101 — Girish Varma
  ('01JWE000000000000000000001', '01JWC000000000000000000009'),
  ('01JWE000000000000000000002', '01JWC000000000000000000009'),
  ('01JWE000000000000000000003', '01JWC000000000000000000009'),
  -- CS1.301 — Kishore Kothapalli
  ('01JWE000000000000000000004', '01JWC000000000000000000006'),
  ('01JWE000000000000000000005', '01JWC000000000000000000006'),
  ('01JWE000000000000000000006', '01JWC000000000000000000006'),
  -- CS2.301 OS — Suresh Purini
  ('01JWE000000000000000000007', '01JWC000000000000000000008'),
  ('01JWE000000000000000000008', '01JWC000000000000000000008'),
  -- CS2.302 Networks — Raghu Babu
  ('01JWE000000000000000000009', '01JWC000000000000000000005'),
  ('01JWE00000000000000000000A', '01JWC000000000000000000005'),
  -- CS3.401 ML — Girish Varma
  ('01JWE00000000000000000000B', '01JWC000000000000000000009'),
  ('01JWE00000000000000000000C', '01JWC000000000000000000009'),
  ('01JWE00000000000000000000D', '01JWC000000000000000000009'),
  -- CS3.402 CV — Jawahar + Jayanthi
  ('01JWE00000000000000000000E', '01JWC000000000000000000002'),
  ('01JWE00000000000000000000E', '01JWC000000000000000000001'),
  ('01JWE00000000000000000000F', '01JWC000000000000000000002'),
  -- CS3.403 NLP — Radhika
  ('01JWE00000000000000000000G', '01JWC000000000000000000003'),
  ('01JWE00000000000000000000H', '01JWC000000000000000000003'),
  -- CS2.303 DB — Suresh
  ('01JWE00000000000000000000J', '01JWC000000000000000000008'),
  ('01JWE00000000000000000000K', '01JWC000000000000000000008'),
  -- CS3.301 Algo — Kishore
  ('01JWE00000000000000000000M', '01JWC000000000000000000006'),
  ('01JWE00000000000000000000N', '01JWC000000000000000000006'),
  -- CS3.302 Compilers — Suresh
  ('01JWE00000000000000000000P', '01JWC000000000000000000008'),
  -- MA1.101 — Girish
  ('01JWE00000000000000000000Q', '01JWC000000000000000000009'),
  ('01JWE00000000000000000000R', '01JWC000000000000000000009'),
  -- MA1.102 — Girish
  ('01JWE00000000000000000000S', '01JWC000000000000000000009'),
  ('01JWE00000000000000000000T', '01JWC000000000000000000009'),
  -- HS1.101 — Radhika
  ('01JWE00000000000000000000V', '01JWC000000000000000000003'),
  ('01JWE00000000000000000000W', '01JWC000000000000000000003'),
  -- Deep Learning — Jawahar + Girish
  ('01JWE00000000000000000000X', '01JWC000000000000000000002'),
  ('01JWE00000000000000000000X', '01JWC000000000000000000009'),
  ('01JWE00000000000000000000Y', '01JWC000000000000000000002'),
  -- IR — Vasudeva
  ('01JWE00000000000000000000Z', '01JWC000000000000000000004'),
  -- Parallel — Kishore
  ('01JWE000000000000000000010', '01JWC000000000000000000006'),
  -- Crypto — Sachin
  ('01JWE000000000000000000011', '01JWC00000000000000000000A'),
  -- HCI — Kavita
  ('01JWE000000000000000000012', '01JWC00000000000000000000D'),
  -- CogSci — Bapi
  ('01JWE000000000000000000013', '01JWC00000000000000000000B');

-- course_reviews
INSERT INTO course_reviews (id, user_id, offering_id, anonymous, difficulty, teaching, grading, content, workload, body) VALUES
  ('01JWF000000000000000000001', '01JWA000000000000000000001', '01JWE000000000000000000001', false, 2, 5, 4, 4, 2, 'Great intro course. Girish explains concepts really clearly and the assignments are well-designed. Grading is lenient if you put in effort.'),
  ('01JWF000000000000000000002', '01JWA000000000000000000002', '01JWE000000000000000000001', false, 2, 4, 4, 3, 2, 'Good for absolute beginners. Pace was comfortable and the TA support was solid throughout.'),
  ('01JWF000000000000000000003', '01JWA000000000000000000003', '01JWE000000000000000000002', true,  3, 5, 3, 4, 3, 'Content is great but grading was unpredictable this semester. Prof is engaging though.'),
  ('01JWF000000000000000000004', '01JWA000000000000000000001', '01JWE000000000000000000004', false, 4, 4, 3, 5, 4, 'DSA is tough but Kishore makes it manageable. The coding assignments are well-structured and the exams are fair.'),
  ('01JWF000000000000000000005', '01JWA000000000000000000004', '01JWE000000000000000000004', false, 5, 3, 3, 5, 5, 'Challenging course with heavy workload. Content is excellent but be prepared for long nights before deadlines.'),
  ('01JWF000000000000000000006', '01JWA000000000000000000002', '01JWE000000000000000000005', true,  4, 4, 4, 5, 4, 'One of the best structured courses in the curriculum. Highly recommend doing all optional problems.'),
  ('01JWF000000000000000000007', '01JWA000000000000000000005', '01JWE000000000000000000005', false, 4, 5, 4, 5, 4, 'Kishore is among the best professors here. Incredibly clear explanations and always accessible outside class.'),
  ('01JWF000000000000000000008', '01JWA000000000000000000003', '01JWE000000000000000000007', false, 5, 3, 3, 4, 5, 'OS is inherently hard and this course does not make it easy. Labs are painful but you learn a lot.'),
  ('01JWF000000000000000000009', '01JWA000000000000000000006', '01JWE000000000000000000007', true,  4, 3, 2, 4, 5, 'Grading felt arbitrary, especially for the labs. Content is solid though.'),
  ('01JWF00000000000000000000A', '01JWA000000000000000000007', '01JWE000000000000000000008', false, 5, 4, 3, 4, 5, 'Suresh improved the course this year. xv6 labs are still a grind but the teaching quality went up.'),
  ('01JWF00000000000000000000B', '01JWA000000000000000000008', '01JWE000000000000000000009', false, 3, 3, 3, 3, 3, 'Average course. Content is important but teaching was dry. Socket programming assignment was fun.'),
  ('01JWF00000000000000000000C', '01JWA000000000000000000001', '01JWE00000000000000000000B', false, 4, 5, 4, 5, 4, 'Excellent course. Girish brings a rigorous theoretical perspective that is rare. A must-take.'),
  ('01JWF00000000000000000000D', '01JWA000000000000000000009', '01JWE00000000000000000000B', true,  3, 5, 4, 5, 3, 'Theory-heavy ML. Not the pytorch tutorial kind. Actually teaches you what is happening under the hood.'),
  ('01JWF00000000000000000000E', '01JWA000000000000000000004', '01JWE00000000000000000000C', false, 4, 5, 4, 5, 4, 'Same great quality. The project component this year was particularly well-scoped.'),
  ('01JWF00000000000000000000F', '01JWA000000000000000000002', '01JWE00000000000000000000E', false, 4, 5, 4, 5, 4, 'Jawahar is legendary. The CV course covers a lot of ground — classical and modern. Take it if you can.'),
  ('01JWF00000000000000000000G', '01JWA00000000000000000000A', '01JWE00000000000000000000E', true,  3, 4, 4, 4, 3, 'Good course overall. Some lectures felt rushed. Assignments are interesting.'),
  ('01JWF00000000000000000000H', '01JWA000000000000000000005', '01JWE00000000000000000000G', false, 3, 4, 4, 4, 3, 'Radhika is approachable and knows the domain deeply. Good mix of linguistic theory and implementation.'),
  ('01JWF00000000000000000000J', '01JWA000000000000000000006', '01JWE00000000000000000000J', false, 3, 4, 4, 4, 3, 'Practical and well-taught. The project was the best part — building something end to end is very satisfying.'),
  ('01JWF00000000000000000000K', '01JWA00000000000000000000B', '01JWE00000000000000000000J', true,  3, 3, 4, 4, 3, 'Content is useful but lectures were occasionally repetitive.'),
  ('01JWF00000000000000000000M', '01JWA000000000000000000007', '01JWE00000000000000000000M', false, 5, 4, 3, 5, 5, 'Hard course but rewarding. The problem sets push you to think, not just code.'),
  ('01JWF00000000000000000000N', '01JWA000000000000000000008', '01JWE00000000000000000000P', false, 4, 4, 4, 5, 4, 'Compilers finally made sense to me here. Implementation-heavy but that is the point.'),
  ('01JWF00000000000000000000P', '01JWA000000000000000000009', '01JWE00000000000000000000Q', false, 3, 4, 4, 4, 3, 'Good linear algebra course. Girish connects theory to ML applications well.'),
  ('01JWF00000000000000000000Q', '01JWA00000000000000000000A', '01JWE00000000000000000000S', false, 3, 4, 3, 4, 3, 'Essential course. Reasonably taught. Some topics felt rushed near the end.'),
  ('01JWF00000000000000000000R', '01JWA000000000000000000001', '01JWE00000000000000000000X', false, 4, 5, 4, 5, 4, 'Best course I took at IIIT. Jawahar and Girish together is a dream combo. Transformers, diffusion, everything.'),
  ('01JWF00000000000000000000S', '01JWA00000000000000000000C', '01JWE00000000000000000000X', true,  5, 5, 3, 5, 5, 'Very demanding but absolutely worth it. The final project is where you really learn.'),
  ('01JWF00000000000000000000T', '01JWA000000000000000000003', '01JWE00000000000000000000Z', false, 3, 4, 4, 4, 3, 'Vasudeva is a great teacher. Course covers both classical IR and modern neural methods.'),
  ('01JWF00000000000000000000V', '01JWA00000000000000000000B', '01JWE000000000000000000010', false, 4, 4, 4, 4, 4, 'CUDA programming is painful at first but Kishore is patient. Good content for anyone doing ML.'),
  ('01JWF00000000000000000000W', '01JWA000000000000000000004', '01JWE000000000000000000011', false, 4, 5, 4, 5, 3, 'Sachin is an excellent teacher. Rigorous proofs but he makes them accessible. Great elective.'),
  ('01JWF00000000000000000000X', '01JWA000000000000000000006', '01JWE000000000000000000012', false, 2, 5, 5, 4, 2, 'Light workload, very enjoyable. Kavita brings real empathy to HCI. Good break from heavy CS courses.'),
  ('01JWF00000000000000000000Y', '01JWA000000000000000000005', '01JWE000000000000000000013', false, 3, 4, 4, 4, 3, 'Bapi is passionate about the subject. Interdisciplinary and unlike anything else in the curriculum.');

-- advisor_reviews
INSERT INTO advisor_reviews (id, user_id, faculty_id, anonymous, research, availability, mentorship, support, workload, body) VALUES
  ('01JWG000000000000000000001', '01JWA000000000000000000001', '01JWC000000000000000000002', false, 5, 4, 5, 4, 4, 'Jawahar is an exceptional advisor. Gives you freedom to explore while keeping you on track. Very well-connected in the CV community.'),
  ('01JWG000000000000000000002', '01JWA000000000000000000002', '01JWC000000000000000000009', false, 5, 5, 5, 5, 3, 'Girish is the best advisor I have had. Deeply technical, always available, and genuinely cares about your growth.'),
  ('01JWG000000000000000000003', '01JWA000000000000000000003', '01JWC000000000000000000003', true,  4, 3, 4, 3, 4, 'Good research direction. Sometimes takes time to respond but meetings are always productive.'),
  ('01JWG000000000000000000004', '01JWA000000000000000000004', '01JWC000000000000000000006', false, 5, 4, 4, 4, 5, 'Kishore pushes you hard. High workload but you come out much stronger. Very knowledgeable.'),
  ('01JWG000000000000000000005', '01JWA000000000000000000005', '01JWC000000000000000000004', false, 4, 4, 4, 4, 3, 'Vasudeva is supportive and has great industry connections. Good advisor for NLP and IR work.'),
  ('01JWG000000000000000000006', '01JWA000000000000000000006', '01JWC000000000000000000001', true,  4, 3, 4, 3, 3, 'Jayanthi has deep expertise in medical imaging. Can be hard to reach but when available is very helpful.'),
  ('01JWG000000000000000000007', '01JWA000000000000000000007', '01JWC000000000000000000008', false, 4, 4, 4, 4, 4, 'Suresh is methodical and precise. Good for systems research. He expects rigor in writing which makes you better.'),
  ('01JWG000000000000000000008', '01JWA000000000000000000008', '01JWC00000000000000000000A', false, 5, 5, 5, 5, 3, 'Sachin is phenomenal. Always available, deeply insightful on crypto theory, and very encouraging.'),
  ('01JWG000000000000000000009', '01JWA000000000000000000009', '01JWC00000000000000000000B', true,  3, 4, 3, 4, 3, 'Bapi is kind and easy to work with. Research direction could be more focused though.'),
  ('01JWG00000000000000000000A', '01JWA00000000000000000000A', '01JWC00000000000000000000D', false, 4, 5, 5, 5, 2, 'Kavita is extremely supportive and the HCI lab has a great culture. Very low-stress environment.'),
  ('01JWG00000000000000000000B', '01JWA00000000000000000000B', '01JWC000000000000000000002', true,  5, 3, 4, 4, 5, 'World-class researcher. Very busy so availability is limited, but the guidance you get is gold.'),
  ('01JWG00000000000000000000C', '01JWA00000000000000000000C', '01JWC000000000000000000009', false, 5, 5, 5, 5, 3, 'Girish is everything you could want in an advisor. Patient, technically brilliant, and invests in your career.');

-- course_review_votes (no id needed — uses DEFAULT gen_random_uuid()::text)
INSERT INTO course_review_votes (user_id, review_id, vote) VALUES
  ('01JWA000000000000000000002', '01JWF000000000000000000001', 1),
  ('01JWA000000000000000000003', '01JWF000000000000000000001', 1),
  ('01JWA000000000000000000004', '01JWF000000000000000000001', 1),
  ('01JWA000000000000000000001', '01JWF000000000000000000004', 1),
  ('01JWA000000000000000000002', '01JWF000000000000000000004', 1),
  ('01JWA000000000000000000006', '01JWF000000000000000000004', 1),
  ('01JWA000000000000000000007', '01JWF000000000000000000005', -1),
  ('01JWA000000000000000000001', '01JWF000000000000000000007', 1),
  ('01JWA000000000000000000003', '01JWF000000000000000000007', 1),
  ('01JWA000000000000000000009', '01JWF000000000000000000007', 1),
  ('01JWA00000000000000000000A', '01JWF000000000000000000007', 1),
  ('01JWA000000000000000000002', '01JWF00000000000000000000C', 1),
  ('01JWA000000000000000000004', '01JWF00000000000000000000C', 1),
  ('01JWA000000000000000000006', '01JWF00000000000000000000C', 1),
  ('01JWA000000000000000000008', '01JWF00000000000000000000C', 1),
  ('01JWA00000000000000000000A', '01JWF00000000000000000000C', 1),
  ('01JWA000000000000000000003', '01JWF00000000000000000000F', 1),
  ('01JWA000000000000000000005', '01JWF00000000000000000000F', 1),
  ('01JWA000000000000000000007', '01JWF00000000000000000000F', 1),
  ('01JWA000000000000000000009', '01JWF00000000000000000000F', 1),
  ('01JWA00000000000000000000B', '01JWF00000000000000000000F', 1),
  ('01JWA000000000000000000001', '01JWF00000000000000000000R', 1),
  ('01JWA000000000000000000003', '01JWF00000000000000000000R', 1),
  ('01JWA000000000000000000005', '01JWF00000000000000000000R', 1),
  ('01JWA000000000000000000007', '01JWF00000000000000000000R', 1),
  ('01JWA000000000000000000009', '01JWF00000000000000000000R', 1),
  ('01JWA00000000000000000000B', '01JWF00000000000000000000R', 1),
  ('01JWA000000000000000000002', '01JWF000000000000000000009', -1),
  ('01JWA000000000000000000005', '01JWF000000000000000000009', -1),
  ('01JWA000000000000000000004', '01JWF00000000000000000000W', 1),
  ('01JWA000000000000000000006', '01JWF00000000000000000000W', 1),
  ('01JWA000000000000000000008', '01JWF00000000000000000000W', 1),
  ('01JWA000000000000000000001', '01JWF00000000000000000000X', 1),
  ('01JWA000000000000000000003', '01JWF00000000000000000000X', 1),
  ('01JWA000000000000000000007', '01JWF00000000000000000000X', 1),
  ('01JWA000000000000000000009', '01JWF00000000000000000000M', 1),
  ('01JWA00000000000000000000A', '01JWF00000000000000000000M', -1),
  ('01JWA00000000000000000000C', '01JWF00000000000000000000N', 1),
  ('01JWA000000000000000000002', '01JWF00000000000000000000P', 1),
  ('01JWA000000000000000000006', '01JWF00000000000000000000P', 1);

-- advisor_review_votes
INSERT INTO advisor_review_votes (user_id, review_id, vote) VALUES
  ('01JWA000000000000000000003', '01JWG000000000000000000001', 1),
  ('01JWA000000000000000000005', '01JWG000000000000000000001', 1),
  ('01JWA000000000000000000007', '01JWG000000000000000000001', 1),
  ('01JWA000000000000000000001', '01JWG000000000000000000002', 1),
  ('01JWA000000000000000000003', '01JWG000000000000000000002', 1),
  ('01JWA000000000000000000005', '01JWG000000000000000000002', 1),
  ('01JWA000000000000000000007', '01JWG000000000000000000002', 1),
  ('01JWA000000000000000000009', '01JWG000000000000000000002', 1),
  ('01JWA00000000000000000000B', '01JWG000000000000000000002', 1),
  ('01JWA000000000000000000006', '01JWG000000000000000000004', 1),
  ('01JWA000000000000000000008', '01JWG000000000000000000004', 1),
  ('01JWA000000000000000000002', '01JWG000000000000000000008', 1),
  ('01JWA000000000000000000004', '01JWG000000000000000000008', 1),
  ('01JWA000000000000000000006', '01JWG000000000000000000008', 1),
  ('01JWA00000000000000000000A', '01JWG000000000000000000008', 1),
  ('01JWA00000000000000000000C', '01JWG00000000000000000000C', 1),
  ('01JWA000000000000000000004', '01JWG00000000000000000000C', 1),
  ('01JWA000000000000000000001', '01JWG000000000000000000003', -1),
  ('01JWA00000000000000000000A', '01JWG00000000000000000000A', 1),
  ('01JWA000000000000000000008', '01JWG00000000000000000000A', 1);

-- course_review_flags
INSERT INTO course_review_flags (user_id, review_id, reason) VALUES
  ('01JWA00000000000000000000C', '01JWF000000000000000000009', 'contains personal attack on the professor'),
  ('01JWA00000000000000000000B', '01JWF000000000000000000005', 'seems exaggerated and unfair'),
  ('01JWA00000000000000000000A', '01JWF00000000000000000000K', 'not a genuine review'),
  ('01JWA000000000000000000008', '01JWF00000000000000000000B', 'too vague to be useful, possibly spam');

-- advisor_review_flags
INSERT INTO advisor_review_flags (user_id, review_id, reason) VALUES
  ('01JWA000000000000000000007', '01JWG000000000000000000003', 'identifies the reviewer despite being anonymous'),
  ('01JWA000000000000000000009', '01JWG000000000000000000009', 'seems like a personal grudge, not a review'),
  ('01JWA000000000000000000002', '01JWG000000000000000000006', 'misleading — reviewer was not actually their student');
