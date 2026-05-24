import { toast } from 'svelte-sonner';
import { env } from '$env/dynamic/public';
import type {
	CourseLean, CourseDetail, CourseReview,
	FacultyLean, FacultyDetail, AdvisorReview,
	LabLean, LabDetail, FlagResponse, AuthUser,
	CreateCourseReview, EditCourseReview,
	CreateAdvisorReview, EditAdvisorReview
} from './types';

const BASE = env.PUBLIC_API_URL;

async function apifetch<T>(path: string, options?: RequestInit): Promise<T | null> {
	try {
		const res = await fetch(`${BASE}${path}`, { credentials: 'include', ...options });
		if (!res.ok) {
			if (res.status !== 401) toast.error(await res.text());
			return null;
		}
		return res.json();
	} catch {
		toast.error('Network error — is the backend running?');
		return null;
	}
}

async function apivoid(path: string, options?: RequestInit): Promise<boolean> {
	try {
		const res = await fetch(`${BASE}${path}`, { credentials: 'include', ...options });
		if (!res.ok) {
			if (res.status !== 401) toast.error(await res.text());
			return false;
		}
		return true;
	} catch {
		toast.error('Network error — is the backend running?');
		return false;
	}
}

function json(body: unknown, method = 'POST'): RequestInit {
	return { method, headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) };
}

export async function getCourses(params?: { q?: string; instructor?: string; sort?: 'rating_asc' | 'rating_desc' }): Promise<CourseLean[] | null> {
	const p = new URLSearchParams();
	if (params?.q?.trim()) p.set('q', params.q.trim());
	if (params?.instructor) p.set('instructor', params.instructor);
	if (params?.sort) p.set('sort', params.sort);
	const qs = p.size ? `?${p}` : '';
	return apifetch<CourseLean[]>(`/courses${qs}`);
}

export async function getCourse(code: string): Promise<CourseDetail | null> {
	return apifetch<CourseDetail>(`/courses/${encodeURIComponent(code)}`);
}

export async function getCourseReviews(code: string): Promise<CourseReview[] | null> {
	return apifetch<CourseReview[]>(`/courses/${encodeURIComponent(code)}/reviews`);
}

export async function getFaculty(params?: { q?: string; sort?: 'rating_asc' | 'rating_desc' }): Promise<FacultyLean[] | null> {
	const p = new URLSearchParams();
	if (params?.q?.trim()) p.set('q', params.q.trim());
	if (params?.sort) p.set('sort', params.sort);
	const qs = p.size ? `?${p}` : '';
	return apifetch<FacultyLean[]>(`/faculty${qs}`);
}

export async function getFacultyMember(slug: string): Promise<FacultyDetail | null> {
	return apifetch<FacultyDetail>(`/faculty/${slug}`);
}

export async function getAdvisorReviews(slug: string): Promise<AdvisorReview[] | null> {
	return apifetch<AdvisorReview[]>(`/faculty/${slug}/reviews`);
}

export async function getLabs(q?: string): Promise<LabLean[] | null> {
	const qs = q?.trim() ? `?q=${encodeURIComponent(q.trim())}` : '';
	return apifetch<LabLean[]>(`/labs${qs}`);
}

export async function searchAll(q: string): Promise<{ courses: CourseLean[]; faculty: FacultyLean[]; labs: LabLean[] }> {
	const [courses, faculty, labs] = await Promise.all([
		getCourses({ q }),
		getFaculty({ q }),
		getLabs(q),
	]);
	return { courses: courses ?? [], faculty: faculty ?? [], labs: labs ?? [] };
}

export async function getLab(shortname: string): Promise<LabDetail | null> {
	return apifetch<LabDetail>(`/labs/${shortname}`);
}

export async function getOfferingReviews(offeringId: string): Promise<CourseReview[] | null> {
	return apifetch<CourseReview[]>(`/offerings/${offeringId}/reviews`);
}

export async function getMe(): Promise<AuthUser | null> {
	return apifetch<AuthUser>('/me');
}

export async function createCourseReview(offeringId: string, body: CreateCourseReview): Promise<CourseReview | null> {
	return apifetch<CourseReview>(`/offerings/${offeringId}/reviews`, json(body));
}

export async function editCourseReview(id: string, body: EditCourseReview): Promise<CourseReview | null> {
	return apifetch<CourseReview>(`/reviews/course/${id}`, json(body, 'PATCH'));
}

export async function deleteCourseReview(id: string): Promise<boolean> {
	return apivoid(`/reviews/course/${id}`, { method: 'DELETE' });
}

export async function createAdvisorReview(slug: string, body: CreateAdvisorReview): Promise<AdvisorReview | null> {
	return apifetch<AdvisorReview>(`/faculty/${slug}/reviews`, json(body));
}

export async function editAdvisorReview(id: string, body: EditAdvisorReview): Promise<AdvisorReview | null> {
	return apifetch<AdvisorReview>(`/reviews/advisor/${id}`, json(body, 'PATCH'));
}

export async function deleteAdvisorReview(id: string): Promise<boolean> {
	return apivoid(`/reviews/advisor/${id}`, { method: 'DELETE' });
}

export async function voteCourseReview(id: string, value: 1 | -1): Promise<boolean> {
	return apivoid(`/reviews/course/${id}/vote`, json({ value }));
}

export async function unvoteCourseReview(id: string): Promise<boolean> {
	return apivoid(`/reviews/course/${id}/vote`, { method: 'DELETE' });
}

export async function voteAdvisorReview(id: string, value: 1 | -1): Promise<boolean> {
	return apivoid(`/reviews/advisor/${id}/vote`, json({ value }));
}

export async function unvoteAdvisorReview(id: string): Promise<boolean> {
	return apivoid(`/reviews/advisor/${id}/vote`, { method: 'DELETE' });
}

export async function flagCourseReview(id: string, reason: string): Promise<boolean> {
	return apivoid(`/reviews/course/${id}/flag`, json({ reason }));
}

export async function flagAdvisorReview(id: string, reason: string): Promise<boolean> {
	return apivoid(`/reviews/advisor/${id}/flag`, json({ reason }));
}

export async function getFlags(): Promise<FlagResponse[] | null> {
	return apifetch<FlagResponse[]>('/admin/flags');
}

export async function dismissFlag(id: string): Promise<boolean> {
	return apivoid(`/admin/flags/${id}/dismiss`, { method: 'POST' });
}

export async function deleteFlaggedReview(id: string): Promise<boolean> {
	return apivoid(`/admin/flags/${id}/delete-review`, { method: 'POST' });
}
