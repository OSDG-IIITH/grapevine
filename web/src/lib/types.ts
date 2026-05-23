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
	lab: string | null;
	overall: number;
}

export interface FacultyDetail {
	id: string;
	name: string;
	slug: string;
	bio: string;
	title: string;
	overall: number;
	lab: { id: string; name: string; short: string } | null;
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

export interface CourseDetail {
	id: string;
	code: string;
	name: string;
	description: string;
	type: string;
	overall: number;
	offerings: Offering[];
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
	user_vote: 1 | -1 | 0;
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
	user_vote: 1 | -1 | 0;
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
