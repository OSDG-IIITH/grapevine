export interface CourseLean {
	id: string;
	code: string;
	name: string;
	type: string;
	overall: number;
}

export interface FacultyLean {
	id: string;
	name: string;
	slug: string;
	labs: { id: string; name: string; short: string }[];
	overall: number;
}

export interface FacultyDetail {
	id: string;
	name: string;
	slug: string;
	bio: string;
	title: string;
	overall: number;
	labs: { id: string; name: string; short: string }[];
	offerings: { id: string; code: string; season: string; year: number; course: { id: string; code: string; name: string } }[];
}

export interface LabLean {
	id: string;
	name: string;
	short: string;
	description: string;
	facultycount: number;
	overall: number;
}

export interface LabDetail {
	id: string;
	name: string;
	short: string;
	description: string;
	facultycount: number;
	overall: number;
	axes: Record<string, number>;
	faculty: { id: string; name: string; slug: string; title: string; overall: number }[];
}

export interface Offering {
	id: string;
	code: string;
	season: string;
	year: number;
	faculty: { id: string; name: string; slug?: string }[];
}

export interface ProposedOfferingLean {
	id: string;
	season: string;
	year: number;
}

export interface CourseRef {
	id: string;
	code: string;
	name: string;
}

export interface CourseDetail {
	id: string;
	code: string;
	name: string;
	description: string;
	type: string;
	overall: number;
	predecessors: CourseRef[];
	successors: CourseRef[];
	offerings: Offering[];
	proposed_offerings: ProposedOfferingLean[];
}

export interface ProposedOfferingResponse {
	id: string;
	course_code: string;
	course_name: string;
	season: string;
	year: number;
	faculty: string[];
	reviews: ProposedReviewResponse[];
}

export interface ProposedReviewResponse {
	id: string;
	body: string;
	difficulty: number;
	teaching: number;
	grading: number;
	content: number;
	workload: number;
	author_name: string | null;
	created_at: string;
}

export interface CourseReview {
	id: string;
	offering_id: string;
	author: { id: string; display_name: string } | null;
	anonymous: boolean;
	difficulty: number;
	teaching: number;
	grading: number;
	content: number;
	workload: number;
	overall: number;
	body: string;
	score: number;
	upvotes: number;
	downvotes: number;
	user_vote: 1 | -1 | null;
	edited_at: string | null;
	created_at: string;
}

export interface AdvisorReview {
	id: string;
	author: { id: string; display_name: string } | null;
	anonymous: boolean;
	research: number;
	availability: number;
	mentorship: number;
	support: number;
	workload: number;
	overall: number;
	body: string;
	score: number;
	upvotes: number;
	downvotes: number;
	user_vote: 1 | -1 | null;
	edited_at: string | null;
	created_at: string;
}

export const COURSE_AXIS_ORDER = ['difficulty', 'workload', 'teaching', 'grading', 'content'] as const;

export const COURSE_AXIS_LABELS: Record<string, string> = {
	difficulty: 'Difficulty',
	workload: 'Workload',
	teaching: 'Teaching',
	grading: 'Grading',
	content: 'Content'
};

export const ADVISOR_AXIS_ORDER = ['research', 'availability', 'mentorship', 'support', 'workload'] as const;

export const ADVISOR_AXIS_LABELS: Record<string, string> = {
	research: 'Research',
	availability: 'Availability',
	mentorship: 'Mentorship',
	support: 'Support',
	workload: 'Workload'
};

export interface FlagResponse {
	id: string;
	reason: string;
	created_at: string;
	review_type: 'course' | 'advisor';
	review_id: string;
	review_body: string;
	reporter: { id: string; display_name: string };
	course_code?: string;
	offering_code?: string;
	faculty_name?: string;
}

export interface SearchResult {
	type: 'course' | 'faculty' | 'lab';
	name: string;
	code?: string;
	slug?: string;
	shortname?: string;
}

export interface AuthUser {
	id: string;
	display_name: string;
	is_admin: boolean;
	verified: boolean;
	username: string | null;
	auth_method: 'cas' | 'local';
	security_question: string | null;
	has_recovery_code: boolean;
}

export interface CreateCourseReview {
	anonymous: boolean;
	difficulty: number;
	teaching: number;
	grading: number;
	content: number;
	workload: number;
	body: string;
}

export interface EditCourseReview extends Partial<CreateCourseReview> {}

export interface CreateAdvisorReview {
	anonymous: boolean;
	research: number;
	availability: number;
	mentorship: number;
	support: number;
	workload: number;
	body: string;
}

export interface EditAdvisorReview extends Partial<CreateAdvisorReview> {}

export interface MyReviews {
	course: CourseReview[];
	advisor: AdvisorReview[];
}

export interface PatchCourse {
	name: string;
	description: string;
	type: string;
	predecessor_ids?: string[];
	successor_ids?: string[];
}

export interface PatchFaculty {
	name: string;
	slug: string;
	bio: string;
	lab_ids: string[];
}

export interface PatchLab {
	name: string;
	short: string;
	description: string;
}

export const COURSE_TYPES = ['core', 'open', 'breadth', 'stream', 'bouquet', 'hs', 'sci', 'math'] as const;
