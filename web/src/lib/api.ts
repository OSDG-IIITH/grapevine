import coursesData from '$lib/mock/courses.json';
import courseDetailData from '$lib/mock/course_detail.json';
import courseReviewsData from '$lib/mock/course_reviews.json';
import facultyData from '$lib/mock/faculty.json';
import facultyDetailData from '$lib/mock/faculty_detail.json';
import advisorReviewsData from '$lib/mock/advisor_reviews.json';
import labsData from '$lib/mock/labs.json';
import labDetailData from '$lib/mock/lab_detail.json';
import type { CourseLean, CourseDetail, CourseReview, FacultyLean, FacultyDetail, AdvisorReview, LabLean, LabDetail } from './types';

export async function getCourses(q?: string): Promise<CourseLean[]> {
	const courses = coursesData as CourseLean[];
	if (!q?.trim()) return courses;
	const s = q.toLowerCase();
	return courses.filter(
		(c) => c.name.toLowerCase().includes(s) || c.code.toLowerCase().includes(s)
	);
}

export async function getCourse(codeOrId: string): Promise<CourseDetail> {
	const detail = courseDetailData as Record<string, CourseDetail>;
	if (detail[codeOrId]) return detail[codeOrId];
	const byId = Object.values(detail).find((c) => c.id === codeOrId);
	if (byId) return byId;
	throw new Error(`Course ${codeOrId} not found`);
}

export async function getCourseReviews(codeOrId: string): Promise<CourseReview[]> {
	const reviews = courseReviewsData as Record<string, CourseReview[]>;
	if (reviews[codeOrId]) return reviews[codeOrId];
	const detail = courseDetailData as Record<string, CourseDetail>;
	const course = Object.values(detail).find((c) => c.id === codeOrId);
	if (course) return reviews[course.code] ?? [];
	return [];
}

export async function getFaculty(q?: string): Promise<FacultyLean[]> {
	const faculty = facultyData as FacultyLean[];
	if (!q?.trim()) return faculty;
	const s = q.toLowerCase();
	return faculty.filter((f) => f.name.toLowerCase().includes(s));
}

export async function getFacultyMember(slug: string): Promise<FacultyDetail> {
	const detail = facultyDetailData as Record<string, FacultyDetail>;
	if (detail[slug]) return detail[slug];
	throw new Error(`Faculty ${slug} not found`);
}

export async function getAdvisorReviews(slug: string): Promise<AdvisorReview[]> {
	const reviews = advisorReviewsData as Record<string, AdvisorReview[]>;
	return reviews[slug] ?? [];
}

export async function getLabs(q?: string): Promise<LabLean[]> {
	const labs = labsData as LabLean[];
	if (!q?.trim()) return labs;
	const s = q.toLowerCase();
	return labs.filter((l) => l.name.toLowerCase().includes(s) || l.short.toLowerCase().includes(s));
}

export async function getLab(shortname: string): Promise<LabDetail> {
	const detail = labDetailData as Record<string, LabDetail>;
	if (detail[shortname]) return detail[shortname];
	throw new Error(`Lab ${shortname} not found`);
}

export async function getOfferingReviews(offeringId: string): Promise<CourseReview[]> {
	const all = courseReviewsData as Record<string, CourseReview[]>;
	for (const reviews of Object.values(all)) {
		const filtered = reviews.filter((r) => r.offering_id === offeringId);
		if (filtered.length) return filtered;
	}
	return [];
}
