import { toast } from 'svelte-sonner';
import { PUBLIC_API_URL } from '$env/static/public';
import type {
	CourseLean, CourseDetail, CourseReview, Offering,
	FacultyLean, FacultyDetail, AdvisorReview,
	LabLean, LabDetail, FlagResponse, ReportResponse, ReportTarget, AuthUser,
	SearchResult, MyReviews,
	CreateCourse, CreateFaculty, CreateLab,
	CreateCourseReview, EditCourseReview,
	CreateAdvisorReview, EditAdvisorReview,
	PatchCourse, PatchFaculty, PatchLab,
	ProposedOfferingResponse, ProposedOfferingLean,
	AuditLog, LegacyCourseReview, LegacyAdvisorReview
} from './types';

const BASE = PUBLIC_API_URL || '/grapevine/api';

async function apifetch<T>(path: string, options?: RequestInit, silent = false): Promise<T | null> {
	try {
		const res = await fetch(`${BASE}${path}`, { credentials: 'include', ...options });
		if (!res.ok) {
			if (res.status !== 401 && !silent) toast.error(await res.text());
			return null;
		}
		return res.json();
	} catch {
		if (!silent) toast.error('Network error — is the backend running?');
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

const P = 'gv_';
function lswrite(key: string, data: unknown) {
	try { localStorage.setItem(P + key, JSON.stringify(data)); } catch {}
}
function lsread<T>(key: string): T | null {
	try { const v = localStorage.getItem(P + key); return v ? JSON.parse(v) : null; } catch { return null; }
}
function lsclear(prefix?: string) {
	try {
		const full = P + (prefix ?? '');
		const keys = Object.keys(localStorage).filter(k => k.startsWith(full));
		keys.forEach(k => localStorage.removeItem(k));
	} catch {}
}

const listCache = {
	courses: new Map<string, CourseLean[]>(),
	faculty: new Map<string, FacultyLean[]>(),
	labs: null as LabLean[] | null,
	clear() {
		this.courses.clear();
		this.faculty.clear();
		this.labs = null;
		lsclear();
	}
};

export async function getCourses(params?: { q?: string; instructor?: string; sort?: string }): Promise<CourseLean[] | null> {
	const p = new URLSearchParams();
	if (params?.q?.trim()) p.set('q', params.q.trim());
	if (params?.instructor) p.set('instructor', params.instructor);
	if (params?.sort) p.set('sort', params.sort);
	const qs = p.size ? `?${p}` : '';

	if (listCache.courses.has(qs)) {
		return listCache.courses.get(qs)!;
	}
	const data = await apifetch<CourseLean[]>(`/courses${qs}`);
	if (data) {
		listCache.courses.set(qs, data);
		lswrite(`courses${qs}`, data);
		return data;
	}
	const cached = lsread<CourseLean[]>(`courses${qs}`);
	if (cached) { toast.info('Offline — showing cached courses'); return cached; }
	return null;
}

export async function getCourse(code: string): Promise<CourseDetail | null> {
	return apifetch<CourseDetail>(`/courses/${encodeURIComponent(code)}`);
}

export async function getCourseReviews(code: string): Promise<CourseReview[] | null> {
	return apifetch<CourseReview[]>(`/courses/${encodeURIComponent(code)}/reviews`);
}

export async function getLegacyCourseReviews(code: string): Promise<LegacyCourseReview[] | null> {
	return apifetch<LegacyCourseReview[]>(`/courses/${encodeURIComponent(code)}/legacy-reviews`);
}

export async function getFaculty(params?: { q?: string; sort?: string }): Promise<FacultyLean[] | null> {
	const p = new URLSearchParams();
	if (params?.q?.trim()) p.set('q', params.q.trim());
	if (params?.sort) p.set('sort', params.sort);
	const qs = p.size ? `?${p}` : '';

	if (listCache.faculty.has(qs)) {
		return listCache.faculty.get(qs)!;
	}
	const data = await apifetch<FacultyLean[]>(`/faculty${qs}`);
	if (data) {
		listCache.faculty.set(qs, data);
		lswrite(`faculty${qs}`, data);
		return data;
	}
	const cached = lsread<FacultyLean[]>(`faculty${qs}`);
	if (cached) { toast.info('Offline — showing cached faculty'); return cached; }
	return null;
}

export async function getFacultyMember(slug: string): Promise<FacultyDetail | null> {
	return apifetch<FacultyDetail>(`/faculty/${slug}`);
}

export async function getAdvisorReviews(slug: string): Promise<AdvisorReview[] | null> {
	return apifetch<AdvisorReview[]>(`/faculty/${slug}/reviews`);
}

export async function getLegacyAdvisorReviews(slug: string): Promise<LegacyAdvisorReview[] | null> {
	return apifetch<LegacyAdvisorReview[]>(`/faculty/${slug}/legacy-reviews`);
}

export async function getLabs(q?: string): Promise<LabLean[] | null> {
	const qs = q?.trim() ? `?q=${encodeURIComponent(q.trim())}` : '';
	if (!qs && listCache.labs) return listCache.labs;
	const data = await apifetch<LabLean[]>(`/labs${qs}`);
	if (data) {
		if (!qs) listCache.labs = data;
		lswrite(`labs${qs}`, data);
		return data;
	}
	const cached = lsread<LabLean[]>(`labs${qs}`);
	if (cached) { toast.info('Offline — showing cached labs'); return cached; }
	return null;
}

export async function search(q: string): Promise<SearchResult[]> {
	if (!q.trim()) return [];
	try {
		const res = await fetch(`${BASE}/search?q=${encodeURIComponent(q.trim())}`, { credentials: 'include' });
		if (!res.ok) return [];
		return res.json();
	} catch {
		return [];
	}
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

export async function registerLocal(
	username: string,
	password: string,
	recovery_code?: string,
	security_question?: string,
	security_answer?: string
): Promise<AuthUser | null> {
	return apifetch<AuthUser>('/auth/register', json({ username, password, recovery_code, security_question, security_answer }), true);
}

export async function recoveryInfo(username: string): Promise<{ has_recovery_code: boolean; security_question: string | null } | null> {
	return apifetch(`/auth/recovery/${encodeURIComponent(username)}`, undefined, true);
}

export async function resetPassword(
	username: string,
	new_password: string,
	opts: { recovery_code?: string; security_answer?: string }
): Promise<{ new_recovery_code: string | null } | null> {
	return apifetch('/auth/reset-password', json({ username, new_password, ...opts }), true);
}

export async function loginLocal(username: string, password: string): Promise<AuthUser | null> {
	// silent: the login page shows these errors inline, no toast.
	return apifetch<AuthUser>('/auth/login/local', json({ username, password }), true);
}

export function casLoginUrl(): string {
	return `${BASE}/auth/login`;
}

export function casConfirmUrl(): string {
	return `${BASE}/auth/cas/confirm`;
}

export function casVerifyUrl(): string {
	return `${BASE}/auth/verify`;
}

export async function updateMe(display_name: string): Promise<AuthUser | null> {
	return apifetch<AuthUser>('/me', json({ display_name }, 'PATCH'));
}

export async function getMyReviews(): Promise<MyReviews | null> {
	return apifetch<MyReviews>('/me/reviews');
}

export async function createCourseReview(offeringId: string, body: CreateCourseReview): Promise<CourseReview | null> {
	const res = await apifetch<CourseReview>(`/offerings/${offeringId}/reviews`, json(body));
	if (res) listCache.clear();
	return res;
}

export async function editCourseReview(id: string, body: EditCourseReview): Promise<CourseReview | null> {
	const res = await apifetch<CourseReview>(`/reviews/course/${id}`, json(body, 'PATCH'));
	if (res) listCache.clear();
	return res;
}

export async function deleteCourseReview(id: string): Promise<boolean> {
	const res = await apivoid(`/reviews/course/${id}`, { method: 'DELETE' });
	if (res) listCache.clear();
	return res;
}

export async function createAdvisorReview(slug: string, body: CreateAdvisorReview): Promise<AdvisorReview | null> {
	const res = await apifetch<AdvisorReview>(`/faculty/${slug}/reviews`, json(body));
	if (res) listCache.clear();
	return res;
}

export async function editAdvisorReview(id: string, body: EditAdvisorReview): Promise<AdvisorReview | null> {
	const res = await apifetch<AdvisorReview>(`/reviews/advisor/${id}`, json(body, 'PATCH'));
	if (res) listCache.clear();
	return res;
}

export async function deleteAdvisorReview(id: string): Promise<boolean> {
	const res = await apivoid(`/reviews/advisor/${id}`, { method: 'DELETE' });
	if (res) listCache.clear();
	return res;
}

export async function voteCourseReview(id: string, value: 1 | -1): Promise<boolean> {
	const res = await apivoid(`/reviews/course/${id}/vote`, json({ value }));
	if (res) listCache.clear();
	return res;
}

export async function unvoteCourseReview(id: string): Promise<boolean> {
	const res = await apivoid(`/reviews/course/${id}/vote`, { method: 'DELETE' });
	if (res) listCache.clear();
	return res;
}

export async function voteAdvisorReview(id: string, value: 1 | -1): Promise<boolean> {
	const res = await apivoid(`/reviews/advisor/${id}/vote`, json({ value }));
	if (res) listCache.clear();
	return res;
}

export async function unvoteAdvisorReview(id: string): Promise<boolean> {
	const res = await apivoid(`/reviews/advisor/${id}/vote`, { method: 'DELETE' });
	if (res) listCache.clear();
	return res;
}

export async function voteLegacyCourseReview(id: string, value: 1 | -1): Promise<boolean> {
	return apivoid(`/reviews/legacy/course/${id}/vote`, json({ value }));
}

export async function unvoteLegacyCourseReview(id: string): Promise<boolean> {
	return apivoid(`/reviews/legacy/course/${id}/vote`, { method: 'DELETE' });
}

export async function voteLegacyAdvisorReview(id: string, value: 1 | -1): Promise<boolean> {
	return apivoid(`/reviews/legacy/advisor/${id}/vote`, json({ value }));
}

export async function unvoteLegacyAdvisorReview(id: string): Promise<boolean> {
	return apivoid(`/reviews/legacy/advisor/${id}/vote`, { method: 'DELETE' });
}

export async function flagCourseReview(id: string, reason: string): Promise<boolean> {
	return apivoid(`/reviews/course/${id}/flag`, json({ reason }));
}

export async function flagAdvisorReview(id: string, reason: string): Promise<boolean> {
	return apivoid(`/reviews/advisor/${id}/flag`, json({ reason }));
}

export async function submitReport(target_type: ReportTarget, target_id: string, reason: string, faculty_ids?: string[]): Promise<boolean> {
	return apivoid('/reports', json({ target_type, target_id, reason, faculty_ids }));
}

export async function createOffering(code: string, body: { season: string; year: number }): Promise<Offering | null> {
	const res = await apifetch<Offering>(`/courses/${encodeURIComponent(code)}/offerings`, json(body));
	if (res) listCache.clear();
	return res;
}

export async function deleteOffering(id: string): Promise<boolean> {
	const res = await apivoid(`/offerings/${id}`, { method: 'DELETE' });
	if (res) listCache.clear();
	return res;
}

export async function updateOfferingFaculty(id: string, faculty_ids: string[]): Promise<Offering | null> {
	const res = await apifetch<Offering>(`/offerings/${id}`, json({ faculty_ids }, 'PATCH'));
	if (res) listCache.clear();
	return res;
}

export async function createCourse(body: CreateCourse): Promise<CourseDetail | null> {
	const res = await apifetch<CourseDetail>('/courses', json(body));
	if (res) listCache.clear();
	return res;
}

export async function updateCourse(code: string, body: PatchCourse): Promise<CourseDetail | null> {
	const res = await apifetch<CourseDetail>(`/courses/${encodeURIComponent(code)}`, json(body, 'PATCH'));
	if (res) listCache.clear();
	return res;
}

export async function createFaculty(body: CreateFaculty): Promise<FacultyDetail | null> {
	const res = await apifetch<FacultyDetail>('/faculty', json(body));
	if (res) listCache.clear();
	return res;
}

export async function updateFaculty(slug: string, body: PatchFaculty): Promise<FacultyDetail | null> {
	const res = await apifetch<FacultyDetail>(`/faculty/${slug}`, json(body, 'PATCH'));
	if (res) listCache.clear();
	return res;
}

export async function createLab(body: CreateLab): Promise<LabDetail | null> {
	const res = await apifetch<LabDetail>('/labs', json(body));
	if (res) listCache.clear();
	return res;
}

export async function updateLab(shortname: string, body: PatchLab): Promise<LabDetail | null> {
	const res = await apifetch<LabDetail>(`/labs/${shortname}`, json(body, 'PATCH'));
	if (res) listCache.clear();
	return res;
}

export function casLinkUrl(): string {
	return `${BASE}/auth/link`;
}

export async function unlinkCas(username: string, password: string): Promise<boolean> {
	return apivoid('/auth/unlink', json({ username, password }));
}

export async function changePassword(current_password: string, new_password: string): Promise<boolean> {
	return apivoid('/me/password', json({ current_password, new_password }));
}

export async function newRecoveryCode(): Promise<{ code: string } | null> {
	return apifetch<{ code: string }>('/me/recovery-code', { method: 'POST' });
}

export async function updateSecurityQuestion(question: string | null, answer: string | null): Promise<boolean> {
	return apivoid('/me/security-question', json({ question, answer }, 'PATCH'));
}

export async function exportSeedData(): Promise<{ labs: unknown[]; faculty: unknown[]; courses: unknown[]; offerings: unknown[] } | null> {
	return apifetch('/admin/export');
}

export async function getFlags(): Promise<FlagResponse[] | null> {
	return apifetch<FlagResponse[]>('/admin/flags');
}

export async function dismissFlag(id: string): Promise<boolean> {
	return apivoid(`/admin/flags/${id}/dismiss`, { method: 'POST' });
}

export async function getReports(): Promise<ReportResponse[] | null> {
	return apifetch<ReportResponse[]>('/admin/reports');
}

export async function dismissReport(id: string): Promise<boolean> {
	return apivoid(`/admin/reports/${id}/dismiss`, { method: 'POST' });
}

export async function approveReport(id: string): Promise<boolean> {
	return apivoid(`/admin/reports/${id}/approve`, { method: 'POST' });
}

export async function deleteFlaggedReview(id: string): Promise<boolean> {
	const res = await apivoid(`/admin/flags/${id}/delete-review`, { method: 'POST' });
	if (res) listCache.clear();
	return res;
}

export async function getProposedReviews(code: string): Promise<CourseReview[] | null> {
	return apifetch<CourseReview[]>(`/courses/${encodeURIComponent(code)}/proposed-reviews`);
}

export async function proposeOffering(code: string, season: string, year: number, faculty_ids?: string[]): Promise<Offering | null> {
	const res = await apifetch<Offering>(`/courses/${encodeURIComponent(code)}/propose-offering`, json({ season, year, faculty_ids }));
	if (res) listCache.clear();
	return res;
}

export async function proposeReview(
	code: string,
	body: {
		season: string;
		year: number;
		anonymous: boolean;
		difficulty: number;
		teaching: number;
		grading: number;
		content: number;
		workload: number;
		overall: number;
		body: string;
		faculty_ids?: string[];
	}
): Promise<CourseReview | null> {
	const res = await apifetch<CourseReview>(`/courses/${encodeURIComponent(code)}/propose-review`, json(body));
	if (res) listCache.clear();
	return res;
}

export async function deleteCourse(code: string): Promise<boolean> {
	const res = await apivoid(`/courses/${encodeURIComponent(code)}`, { method: 'DELETE' });
	if (res) listCache.clear();
	return res;
}

export async function getDeletedCourses(): Promise<{ code: string; name: string; deleted_at: string }[] | null> {
	return apifetch('/admin/deleted-courses');
}

export async function restoreCourse(code: string): Promise<boolean> {
	const res = await apivoid(`/admin/deleted-courses/${encodeURIComponent(code)}/restore`, { method: 'POST' });
	if (res) listCache.clear();
	return res;
}

export async function deleteFaculty(slug: string): Promise<boolean> {
	const res = await apivoid(`/faculty/${encodeURIComponent(slug)}`, { method: 'DELETE' });
	if (res) listCache.clear();
	return res;
}

export async function getDeletedFaculty(): Promise<{ slug: string; name: string; deleted_at: string }[] | null> {
	return apifetch('/admin/deleted-faculty');
}

export async function restoreFaculty(slug: string): Promise<boolean> {
	const res = await apivoid(`/admin/deleted-faculty/${encodeURIComponent(slug)}/restore`, { method: 'POST' });
	if (res) listCache.clear();
	return res;
}

export async function deleteLab(shortname: string): Promise<boolean> {
	const res = await apivoid(`/labs/${encodeURIComponent(shortname)}`, { method: 'DELETE' });
	if (res) listCache.clear();
	return res;
}

export async function getDeletedLabs(): Promise<{ shortname: string; name: string; deleted_at: string }[] | null> {
	return apifetch('/admin/deleted-labs');
}

export async function restoreLab(shortname: string): Promise<boolean> {
	const res = await apivoid(`/admin/deleted-labs/${encodeURIComponent(shortname)}/restore`, { method: 'POST' });
	if (res) listCache.clear();
	return res;
}

export async function getProposedOfferings(): Promise<ProposedOfferingResponse[] | null> {
	return apifetch<ProposedOfferingResponse[]>('/admin/proposed');
}

export async function approveProposedOffering(id: string): Promise<boolean> {
	const res = await apivoid(`/admin/proposed/${id}/approve`, { method: 'POST' });
	if (res) listCache.clear();
	return res;
}

export async function rejectProposedOffering(id: string): Promise<boolean> {
	const res = await apivoid(`/admin/proposed/${id}/reject`, { method: 'POST' });
	if (res) listCache.clear();
	return res;
}

export async function getAuditLogs(): Promise<AuditLog[] | null> {
	return apifetch<AuditLog[]>('/admin/audit-logs');
}

export async function getDeletedOfferings(): Promise<{ id: string; course_code: string; course_name: string; season: string; year: number; deleted_at: string }[] | null> {
	return apifetch('/admin/deleted-offerings');
}

export async function restoreReview(type: 'course' | 'advisor', id: string): Promise<boolean> {
	const res = await apivoid(`/admin/reviews/${type}/${id}/restore`, { method: 'POST' });
	if (res) listCache.clear();
	return res;
}

export async function restoreOffering(id: string): Promise<boolean> {
	const res = await apivoid(`/admin/offerings/${id}/restore`, { method: 'POST' });
	if (res) listCache.clear();
	return res;
}

export type Moderator = { id: string; display_name: string; username: string | null; cas_id: string | null; created_at: string };

export async function getModerators(): Promise<Moderator[] | null> {
	return apifetch<Moderator[]>('/admin/moderators');
}

export async function addCasModerator(casid: string): Promise<boolean> {
	return apivoid('/admin/moderators/cas', json({ cas_id: casid }));
}

export async function addLocalModerator(username: string, displayName?: string, password?: string): Promise<boolean> {
	return apivoid('/admin/moderators/local', json({ username, display_name: displayName, password }));
}

export async function demoteModerator(id: string): Promise<boolean> {
	return apivoid(`/admin/moderators/${id}/demote`, { method: 'POST' });
}
