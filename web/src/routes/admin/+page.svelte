<script lang="ts">
	import { onMount } from 'svelte';
	import { slide } from 'svelte/transition';
	import { base } from '$app/paths';
	import {
		getFlags, dismissFlag, deleteFlaggedReview, exportSeedData,
		getReports, dismissReport, approveReport,
		getProposedOfferings, approveProposedOffering, rejectProposedOffering,
		getDeletedCourses, restoreCourse,
		getDeletedFaculty, restoreFaculty,
		getDeletedLabs, restoreLab,
		getDeletedOfferings, restoreOffering,
		getAuditLogs, restoreReview,
		getModerators, addCasModerator, addLocalModerator, demoteModerator,
		type Moderator
	} from '$lib/api';
	import { currentUser } from '$lib/stores';
	import type { FlagResponse, ReportResponse, ProposedOfferingResponse, AuditLog } from '$lib/types';
	import Crumbs from '$lib/components/Crumbs.svelte';
	import Pager from '$lib/components/Pager.svelte';
	import IconArrowBackUp from '@tabler/icons-svelte/icons/arrow-back-up';

	type DeletedOffering = { id: string; course_code: string; course_name: string; season: string; year: number; deleted_at: string };

	let items = $state<FlagResponse[]>([]);
	let reports = $state<ReportResponse[]>([]);
	let proposed = $state<ProposedOfferingResponse[]>([]);
	let deleted = $state<{ code: string; name: string; deleted_at: string }[]>([]);
	let deletedfaculty = $state<{ slug: string; name: string; deleted_at: string }[]>([]);
	let deletedlabs = $state<{ shortname: string; name: string; deleted_at: string }[]>([]);
	let deletedofferings = $state<DeletedOffering[]>([]);
	let auditlogs = $state<AuditLog[]>([]);
	let audittotal = $state(0);
	let audithasmore = $state(false);
	let auditoffset = $state(0);
	let auditloading = $state(false);
	let auditfilters = $state({ adminid: '', action: '', targettype: '' });
	let mobilemenuopen = $state(false);
	let moderators = $state<Moderator[]>([]);
	let modtype = $state<'cas' | 'local'>('cas');
	let modinput = $state({ casid: '', username: '' });
	let modsubmitting = $state(false);

	let dialog = $state<{ msg: string; label: string; danger: boolean; fn: () => void } | null>(null);

	function withconfirm(msg: string, fn: () => void, label = 'Confirm', danger = false) {
		dialog = { msg, fn, label, danger };
	}

	const restoreactionmap: Record<string, string> = {
		DELETE_REVIEW: 'RESTORE_REVIEW',
		DELETE_OFFERING: 'RESTORE_OFFERING',
		REJECT_PROPOSED: 'RESTORE_OFFERING',
		DELETE_COURSE: 'RESTORE_COURSE',
		DELETE_FACULTY: 'RESTORE_FACULTY',
		DELETE_LAB: 'RESTORE_LAB',
	};
	let loading = $state(true);
	let error = $state(false);
	let activetab = $state<'reports' | 'flagged' | 'proposed' | 'deleted' | 'audit' | 'moderators'>('audit');

	const AUDIT_LIMIT = 20;

	const AUDIT_ACTIONS = [
		'CREATE_COURSE', 'UPDATE_COURSE', 'DELETE_COURSE',
		'CREATE_FACULTY', 'UPDATE_FACULTY', 'DELETE_FACULTY',
		'CREATE_LAB', 'UPDATE_LAB', 'DELETE_LAB',
		'CREATE_OFFERING', 'DELETE_OFFERING', 'UPDATE_OFFERING_FACULTY',
		'APPROVE_REPORT', 'APPROVE_PROPOSED', 'REJECT_PROPOSED',
		'DELETE_REVIEW', 'DISMISS_FLAG', 'DISMISS_REPORT',
		'RESTORE_COURSE', 'RESTORE_FACULTY', 'RESTORE_LAB',
		'RESTORE_REVIEW', 'RESTORE_OFFERING',
		'ADD_MODERATOR', 'REMOVE_MODERATOR', 'EXPORT_SEED_DATA',
	];
	const AUDIT_TARGET_TYPES = ['course', 'faculty', 'lab', 'offering', 'course_review', 'advisor_review', 'user', 'flag', 'database'];

	async function loadauditlogs() {
		auditloading = true;
		const page = await getAuditLogs(AUDIT_LIMIT, auditoffset, {
			admin_id: auditfilters.adminid || undefined,
			action: auditfilters.action || undefined,
			target_type: auditfilters.targettype || undefined,
		});
		if (page) {
			auditlogs = page.logs;
			audittotal = page.total;
			audithasmore = page.has_more;
		}
		auditloading = false;
	}

	function onauditfilter() {
		auditoffset = 0;
		loadauditlogs();
	}

	onMount(async () => {
		const [flags, reportitems, props, del, delfac, dellabs, deloff, logs, mods] = await Promise.all([
			getFlags(), getReports(), getProposedOfferings(),
			getDeletedCourses(), getDeletedFaculty(), getDeletedLabs(), getDeletedOfferings(),
			getAuditLogs(AUDIT_LIMIT, 0), getModerators()
		]);
		if (flags === null || reportitems === null || props === null) error = true;
		else {
			items = flags;
			reports = reportitems;
			proposed = props;
			deleted = del ?? [];
			deletedfaculty = delfac ?? [];
			deletedlabs = dellabs ?? [];
			deletedofferings = deloff ?? [];
			if (logs) {
				auditlogs = logs.logs;
				audittotal = logs.total;
				audithasmore = logs.has_more;
			}
			moderators = mods ?? [];
		}
		loading = false;
	});

	async function doRestore(code: string) {
		if (await restoreCourse(code)) deleted = deleted.filter((d) => d.code !== code);
	}

	async function doRestoreFaculty(slug: string) {
		if (await restoreFaculty(slug)) deletedfaculty = deletedfaculty.filter((d) => d.slug !== slug);
	}

	async function doRestoreLab(shortname: string) {
		if (await restoreLab(shortname)) deletedlabs = deletedlabs.filter((d) => d.shortname !== shortname);
	}

	async function doRestoreOffering(id: string) {
		if (await restoreOffering(id)) deletedofferings = deletedofferings.filter((d) => d.id !== id);
	}

	async function dismiss(id: string) {
		if (await dismissFlag(id)) items = items.filter((i) => i.id !== id);
	}

	async function dismissInfoReport(id: string) {
		if (await dismissReport(id)) reports = reports.filter((report) => report.id !== id);
	}

	async function approveInfoReport(id: string) {
		if (await approveReport(id)) reports = reports.filter((report) => report.id !== id);
	}

	async function deleteReview(id: string) {
		if (await deleteFlaggedReview(id)) items = items.filter((i) => i.id !== id);
	}

	async function approveProp(id: string) {
		if (await approveProposedOffering(id)) proposed = proposed.filter((p) => p.id !== id);
	}

	async function rejectProp(id: string) {
		if (await rejectProposedOffering(id)) proposed = proposed.filter((p) => p.id !== id);
	}

	async function doRestoreAudit(log: AuditLog) {
		let ok = false;
		let restoreaction = '';
		if (log.action === 'DELETE_REVIEW') {
			ok = await restoreReview(log.target_type === 'course_review' ? 'course' : 'advisor', log.target_id);
			restoreaction = 'RESTORE_REVIEW';
		} else if (log.action === 'DELETE_OFFERING' || log.action === 'REJECT_PROPOSED') {
			ok = await restoreOffering(log.target_id);
			if (ok) deletedofferings = deletedofferings.filter((d) => d.id !== log.target_id);
			restoreaction = 'RESTORE_OFFERING';
		} else if (log.action === 'DELETE_COURSE') {
			ok = await restoreCourse(log.target_id);
			if (ok) deleted = deleted.filter((d) => d.code !== log.target_id);
			restoreaction = 'RESTORE_COURSE';
		} else if (log.action === 'DELETE_FACULTY') {
			ok = await restoreFaculty(log.target_id);
			if (ok) deletedfaculty = deletedfaculty.filter((d) => d.slug !== log.target_id);
			restoreaction = 'RESTORE_FACULTY';
		} else if (log.action === 'DELETE_LAB') {
			ok = await restoreLab(log.target_id);
			if (ok) deletedlabs = deletedlabs.filter((d) => d.shortname !== log.target_id);
			restoreaction = 'RESTORE_LAB';
		}
		if (ok && $currentUser) {
			auditlogs = [{
				id: crypto.randomUUID(),
				admin_id: $currentUser.id,
				admin_name: $currentUser.display_name,
				action: restoreaction,
				target_type: log.target_type,
				target_id: log.target_id,
				target_name: log.target_name,
				target_course_code: log.target_course_code,
				previous_state: restoreaction === 'RESTORE_REVIEW' ? log.previous_state : null,
				created_at: new Date().toISOString()
			}, ...auditlogs];
		}
	}

	function isrestorable(log: AuditLog): boolean {
		const ra = restoreactionmap[log.action];
		if (!ra) return false;
		return !auditlogs.some(l => l.action === ra && l.target_id === log.target_id && l.created_at > log.created_at);
	}

	function actionlabel(action: string): string {
		const labels: Record<string, string> = {
			CREATE_COURSE: 'created course', UPDATE_COURSE: 'updated course', DELETE_COURSE: 'deleted course',
			CREATE_FACULTY: 'created faculty', UPDATE_FACULTY: 'updated faculty', DELETE_FACULTY: 'deleted faculty',
			CREATE_LAB: 'created lab', UPDATE_LAB: 'updated lab', DELETE_LAB: 'deleted lab',
			CREATE_OFFERING: 'created offering', DELETE_OFFERING: 'deleted offering', UPDATE_OFFERING_FACULTY: 'updated offering faculty', APPROVE_REPORT: 'approved instructor change',
			APPROVE_PROPOSED: 'approved proposed', REJECT_PROPOSED: 'rejected proposed',
			DELETE_REVIEW: 'deleted a review', DISMISS_FLAG: 'dismissed flag', DISMISS_REPORT: 'dismissed information report',
			RESTORE_COURSE: 'restored course', RESTORE_FACULTY: 'restored faculty', RESTORE_LAB: 'restored lab',
			RESTORE_REVIEW: 'restored a review', RESTORE_OFFERING: 'restored offering',
			ADD_MODERATOR: 'added moderator', REMOVE_MODERATOR: 'removed moderator',
		};
		return labels[action] ?? action.toLowerCase().replace(/_/g, ' ');
	}

	function auditlink(log: AuditLog): string | null {
		if (log.target_type === 'course') return `/courses/${log.target_id}`;
		if (log.target_type === 'faculty') return `/faculty/${log.target_id}`;
		if (log.target_type === 'lab') return `/labs/${log.target_id}`;
		if (log.target_type === 'offering' && log.target_course_code) return `/courses/${log.target_course_code}`;
		return null;
	}

	async function doAddMod() {
		modsubmitting = true;
		let ok = false;
		if (modtype === 'cas') {
			ok = await addCasModerator(modinput.casid.trim());
		} else {
			ok = await addLocalModerator(modinput.username.trim());
		}
		if (ok) {
			modinput = { casid: '', username: '' };
			moderators = (await getModerators()) ?? moderators;
		}
		modsubmitting = false;
	}

	async function doDemote(id: string) {
		if (await demoteModerator(id)) moderators = moderators.filter((m) => m.id !== id);
	}

	let exporting = $state(false);

	async function doexport() {
		exporting = true;
		const data = await exportSeedData();
		exporting = false;
		if (!data) return;
		for (const key of ['labs', 'faculty', 'courses', 'offerings'] as const) {
			const blob = new Blob([JSON.stringify(data[key], null, 2)], { type: 'application/json' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url; a.download = `${key}.json`; a.click();
			URL.revokeObjectURL(url);
			await new Promise((r) => setTimeout(r, 80));
		}
	}

	function reltime(iso: string): string {
		const date = new Date(iso);
		const now = new Date();
		const isToday = date.toDateString() === now.toDateString();
		if (isToday) {
			const hh = String(date.getHours()).padStart(2, '0');
			const mm = String(date.getMinutes()).padStart(2, '0');
			return `${hh}:${mm}`;
		}
		const d = Math.floor((now.getTime() - date.getTime()) / 86400000);
		if (d <= 1) return 'yesterday';
		if (d < 7) return `${d} days ago`;
		if (d < 30) return `${Math.floor(d / 7)}w ago`;
		return date.toLocaleDateString('en-IN', { day: 'numeric', month: 'short' });
	}

	function auditvalue(value: unknown): string {
		if (Array.isArray(value)) {
			return value.map((item) => typeof item === 'object' && item !== null && 'name' in item ? String(item.name) : String(item)).join(', ') || 'none';
		}
		if (typeof value === 'object' && value !== null) return JSON.stringify(value);
		return String(value);
	}

	function reporttypelabel(type: ReportResponse['target_type']): string {
		return `${type} information`;
	}

	let totaldeleted = $derived(deleted.length + deletedfaculty.length + deletedlabs.length + deletedofferings.length);

	const tablabels: Record<string, string> = {
		audit: 'audit log', reports: 'information reports', flagged: 'flagged reviews',
		proposed: 'proposed offerings', deleted: 'deleted', moderators: 'moderators'
	};

	let tabs = $derived([
		{ id: 'audit', label: 'Audit log', count: audittotal },
		{ id: 'reports', label: 'Reports', count: reports.length },
		{ id: 'flagged', label: 'Flagged', count: items.length },
		{ id: 'proposed', label: 'Proposed', count: proposed.length },
		{ id: 'deleted', label: 'Deleted', count: totaldeleted },
		{ id: 'moderators', label: 'Moderators', count: moderators.length }
	]);
</script>

<svelte:head>
	<title>Admin · grapevine</title>
</svelte:head>

<div class="mx-auto w-full px-8 pb-[120px] pt-10" style="max-width: 1120px; animation: fadeUp 280ms cubic-bezier(.2,.6,.2,1) both;">
	<Crumbs items={[{ label: 'grapevine', href: '/' }, { label: 'admin' }, { label: tablabels[activetab] ?? activetab }]} />

	{#if loading}
		<div class="text-[13px] text-[var(--fg-3)]">Loading…</div>

	{:else if error || !$currentUser?.is_admin}
		<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]"
			style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
			Access denied.
		</div>

	{:else}
		<div class="mb-8 flex flex-wrap items-start justify-between gap-6">
			<div>
				<h1 class="mb-2 font-normal text-[var(--fg)]" style="font-family: var(--serif); font-size: 56px; line-height: 1.05; letter-spacing: -0.015em;">
					Moderator inbox
				</h1>
				<div class="flex items-center gap-[14px] text-[13px] text-[var(--fg-2)]">
					<span>{reports.length + items.length + proposed.length} pending items</span>
				</div>
			</div>
			<button
				type="button"
				onclick={doexport}
				disabled={exporting}
				class="inline-flex items-center gap-[6px] self-start whitespace-nowrap rounded-[7px] border border-[var(--border-strong)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] font-medium text-[var(--fg-2)] transition-colors duration-[120ms] hover:bg-[var(--bg-3)] hover:text-[var(--fg)] disabled:opacity-50"
			>
				<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
					<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
				</svg>
				{exporting ? 'Exporting…' : 'Export seed data'}
			</button>
		</div>

		<div class="flex items-start gap-6">
			<!-- Desktop sidebar -->
			<nav class="hidden md:flex flex-col gap-[2px] w-[176px] shrink-0 pt-[2px]">
				{#each tabs as tab}
					<button
						type="button"
						onclick={() => { activetab = tab.id as typeof activetab; }}
						class="flex items-center justify-between rounded-[7px] px-3 py-[7px] text-[13px] text-left transition-colors {activetab === tab.id ? 'bg-[var(--bg-3)] text-[var(--fg)] font-medium' : 'text-[var(--fg-3)] hover:bg-[var(--bg-2)] hover:text-[var(--fg-2)]'}"
					>
						<span>{tab.label}</span>
						{#if tab.count > 0}
							<span class="text-[11px] text-[var(--fg-4)] tabular-nums" style="font-family: var(--mono);">{tab.count}</span>
						{/if}
					</button>
				{/each}
			</nav>

			<div class="flex-1 min-w-0">
				<!-- Mobile accordion -->
				<div class="md:hidden mb-4 rounded-[8px] border border-[var(--border)] bg-[var(--bg-2)] overflow-hidden">
					<button
						type="button"
						onclick={() => mobilemenuopen = !mobilemenuopen}
						aria-label="Select section"
						class="flex w-full items-center justify-between px-4 py-[10px] text-[13px] text-[var(--fg)]"
					>
						<span class="font-medium">{tabs.find(t => t.id === activetab)?.label ?? activetab}</span>
						<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" class="text-[var(--fg-3)] transition-transform duration-200 {mobilemenuopen ? 'rotate-180' : ''}"><polyline points="6 9 12 15 18 9"/></svg>
					</button>
					{#if mobilemenuopen}
						<div transition:slide={{ duration: 180 }}>
							{#each tabs.filter(t => t.id !== activetab) as tab}
								<button
									type="button"
									onclick={() => { activetab = tab.id as typeof activetab; mobilemenuopen = false; }}
									class="flex w-full items-center justify-between border-t border-[var(--border)] px-4 py-[9px] text-[13px] text-left transition-colors text-[var(--fg-3)] hover:bg-[var(--bg-3)] hover:text-[var(--fg-2)]"
								>
									<span>{tab.label}</span>
									{#if tab.count > 0}
										<span class="text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">{tab.count}</span>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
				</div>

		{#if activetab === 'audit'}
			<div class="mb-3 flex flex-wrap gap-2">
				<select
					bind:value={auditfilters.adminid}
					onchange={onauditfilter}
					class="rounded-[7px] border border-[var(--border)] bg-[var(--bg-2)] px-3 py-[6px] text-[12px] text-[var(--fg-2)] outline-none focus:border-[var(--border-strong)] cursor-pointer"
				>
					<option value="">All moderators</option>
					{#each moderators as mod}
						<option value={mod.id}>{mod.display_name}</option>
					{/each}
				</select>
				<select
					bind:value={auditfilters.action}
					onchange={onauditfilter}
					class="rounded-[7px] border border-[var(--border)] bg-[var(--bg-2)] px-3 py-[6px] text-[12px] text-[var(--fg-2)] outline-none focus:border-[var(--border-strong)] cursor-pointer"
				>
					<option value="">All actions</option>
					{#each AUDIT_ACTIONS as a}
						<option value={a}>{actionlabel(a)}</option>
					{/each}
				</select>
				<select
					bind:value={auditfilters.targettype}
					onchange={onauditfilter}
					class="rounded-[7px] border border-[var(--border)] bg-[var(--bg-2)] px-3 py-[6px] text-[12px] text-[var(--fg-2)] outline-none focus:border-[var(--border-strong)] cursor-pointer"
				>
					<option value="">All target types</option>
					{#each AUDIT_TARGET_TYPES as t}
						<option value={t}>{t.replace('_', ' ')}</option>
					{/each}
				</select>
			</div>
			{#if auditlogs.length === 0 && !auditloading}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]"
					style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
					No audit log entries yet.
				</div>
			{:else}
				{#each auditlogs as log (log.id)}
					<div class="mb-2 flex items-start gap-4 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[18px] py-[14px] transition-[border-color] duration-[120ms] hover:border-[var(--border-strong)]">
						<div class="flex-1 min-w-0">
							<div class="flex flex-wrap items-baseline gap-x-2 gap-y-1 text-[13px]">
								<span class="font-medium text-[var(--fg)]">{log.admin_name}</span>
								<span class="text-[var(--fg-3)]">{actionlabel(log.action)}</span>
								{#if log.previous_state?.['course_code']}
									<span class="text-[var(--fg-3)]">on</span>
									<a href={`${base}/courses/${log.previous_state['course_code']}`} class="text-[var(--fg)] underline decoration-[var(--border-strong)] underline-offset-4 hover:text-[var(--accent-2)]">{log.previous_state['course_name'] as string}</a>
								{:else if log.previous_state?.['faculty_slug']}
									<span class="text-[var(--fg-3)]">on</span>
									<a href={`${base}/faculty/${log.previous_state['faculty_slug']}`} class="text-[var(--fg)] underline decoration-[var(--border-strong)] underline-offset-4 hover:text-[var(--accent-2)]">{log.previous_state['faculty_name'] as string}</a>
								{:else if log.target_name}
									{#if auditlink(log)}
										<a href={`${base}${auditlink(log)}`} class="text-[var(--fg)] underline decoration-[var(--border-strong)] underline-offset-4 hover:text-[var(--accent-2)]">{log.target_name}</a>
									{:else}
										<span class="text-[var(--fg)]">{log.target_name}</span>
									{/if}
									<span class="rounded-[4px] border border-[var(--border-strong)] px-[6px] py-[1px] text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">{log.target_id}</span>
								{:else}
									<span class="rounded-[4px] border border-[var(--border-strong)] px-[6px] py-[1px] text-[11px] text-[var(--fg-2)]" style="font-family: var(--mono);">{log.target_id}</span>
								{/if}
							</div>
							{#if log.previous_state}
								{@const hasdiff = 'before' in log.previous_state && 'after' in log.previous_state}
								{@const isreview = log.target_type === 'course_review' || log.target_type === 'advisor_review'}
								{#if hasdiff}
									{@const before = log.previous_state['before'] as Record<string, unknown>}
									{@const after = log.previous_state['after'] as Record<string, unknown>}
									<div class="mt-1 flex flex-wrap gap-x-3 gap-y-[2px] text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">
										{#each Object.keys(before) as key}
											{#if String(before[key]) !== String(after[key])}
												<span>{key}: <s class="text-[var(--fg-4)] opacity-60">{auditvalue(before[key])}</s> → <span class="text-[var(--fg-3)]">{auditvalue(after[key])}</span></span>
											{/if}
										{/each}
									</div>
								{:else if isreview && log.previous_state['body']}
									<blockquote class="mt-2 border-l-2 border-[var(--border-strong)] pl-3 text-[12px] leading-[1.6] text-[var(--fg-3)]">
										{log.previous_state['body'] as string}
									</blockquote>
								{:else}
									{@const skip = new Set(['course_code', 'course_name', 'faculty_slug', 'faculty_name'])}
									{@const entries = Object.entries(log.previous_state).filter(([k]) => !skip.has(k))}
									{#if entries.length > 0}
										<div class="mt-1 text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">
											{entries.map(([k, v]) => `${k}: ${auditvalue(v)}`).join(' · ')}
										</div>
									{/if}
								{/if}
							{/if}
						</div>
						<div class="flex items-center gap-2 shrink-0">
							<span class="text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">{reltime(log.created_at)}</span>
							{#if isrestorable(log)}
								<button
									type="button"
									onclick={() => doRestoreAudit(log)}
									class="inline-flex items-center rounded-[6px] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] p-[4px] text-[var(--accent-2)] transition-[background-color,transform] duration-[120ms] hover:bg-[rgba(107,143,111,0.14)] active:scale-[0.93]"
								><IconArrowBackUp size={14} /></button>
							{/if}
						</div>
					</div>
				{/each}
				{#if audittotal > AUDIT_LIMIT}
					<Pager
						page={Math.floor(auditoffset / AUDIT_LIMIT) + 1}
						totalpages={Math.ceil(audittotal / AUDIT_LIMIT)}
						totalitems={audittotal}
						onchange={(p) => { auditoffset = (p - 1) * AUDIT_LIMIT; loadauditlogs(); }}
					/>
				{/if}
			{/if}

		{:else if activetab === 'reports'}
			{#if reports.length === 0}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]"
					style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
					Inbox zero. No inaccurate information reported.
				</div>
			{:else}
				{#each reports as report (report.id)}
					<div class="mb-3 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] p-[22px]" style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
						<div class="mb-3 flex flex-wrap items-center gap-3 text-[12px] text-[var(--fg-3)]">
							<span class="rounded px-2 py-[2px] text-[11px] uppercase text-[var(--accent-2)] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)]" style="font-family: var(--mono);">{reporttypelabel(report.target_type)}</span>
							{#if report.course_code}
								<a href={`${base}/courses/${encodeURIComponent(report.course_code)}`} class="text-[var(--fg)] underline decoration-[var(--border-strong)] underline-offset-4 transition-colors hover:text-[var(--accent-2)]">{report.target_label}</a>
							{:else}
								<span class="text-[var(--fg)]">{report.target_label}</span>
							{/if}
							<span class="ml-auto text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">reported {reltime(report.created_at)}</span>
						</div>
						<div class="mb-3 text-[12px] text-[var(--fg-3)]">reported by <span class="text-[var(--fg-2)]">{report.reporter_name}</span></div>
						{#if report.has_faculty_suggestion}
							<div class="mb-3 grid gap-3 rounded-[8px] border border-[var(--border)] bg-[var(--bg-3)] p-3 text-[12px] sm:grid-cols-2">
								<div>
									<div class="mb-1 uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono); font-size: 10px;">Current instructors</div>
									<div class="text-[var(--fg-2)]">{report.current_faculty.length ? report.current_faculty.map((member) => member.name).join(', ') : 'None listed'}</div>
								</div>
								<div>
									<div class="mb-1 uppercase tracking-[0.08em] text-[var(--accent-2)]" style="font-family: var(--mono); font-size: 10px;">Suggested instructors</div>
									<div class="text-[var(--fg)]">{report.suggested_faculty.length ? report.suggested_faculty.map((member) => member.name).join(', ') : 'No instructors'}</div>
								</div>
							</div>
						{/if}
						{#if report.reason}<p class="m-0 whitespace-pre-wrap text-[14px] leading-[1.65] text-[var(--fg)]">{report.reason}</p>{/if}
						<div class="mt-4 flex flex-wrap gap-2">
							{#if report.has_faculty_suggestion}
								<button type="button" onclick={() => approveInfoReport(report.id)} class="inline-flex items-center rounded-[7px] border border-[rgba(107,143,111,0.3)] bg-[var(--accent-bg)] px-[14px] py-2 text-[13px] font-medium text-[var(--accent-2)] transition-[background-color,transform] duration-[120ms] hover:bg-[rgba(107,143,111,0.14)] active:scale-[0.98]">Approve change</button>
							{/if}
							<button type="button" onclick={() => withconfirm('Dismiss this information report?', () => dismissInfoReport(report.id), 'Dismiss')} class="inline-flex items-center rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] text-[var(--fg)] transition-[background-color,border-color,transform] duration-[120ms] hover:bg-[var(--bg-3)] hover:border-[var(--border-strong)] active:scale-[0.98]">Dismiss report</button>
						</div>
					</div>
				{/each}
			{/if}

		{:else if activetab === 'flagged'}
			{#if items.length === 0}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]"
					style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
					Inbox zero. Nothing flagged.
				</div>
			{:else}
				{#each items as it (it.id)}
					<div
						class="mb-3 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] p-[22px]"
						style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);"
					>
						<div class="mb-3 flex items-center gap-3 text-[12px] text-[var(--fg-3)]">
							<span
								class="rounded px-2 py-[2px] text-[11px] text-[var(--danger)] border border-[rgba(217,138,138,0.2)] bg-[var(--danger-bg)]"
								style="font-family: var(--mono);"
							>{it.reason}</span>
							{#if it.review_type === 'course' && it.course_code}
								<span
									class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] tracking-[0.02em] text-[var(--fg-2)]"
									style="font-family: var(--mono);"
								>{it.course_code}</span>
								{#if it.offering_code}
									<span style="font-family: var(--mono); color: var(--fg-3);">{it.offering_code}</span>
								{/if}
							{/if}
							{#if it.review_type === 'advisor' && it.faculty_name}
								<span>advisor review · <span class="text-[var(--fg-2)]">{it.faculty_name}</span></span>
							{/if}
							<span class="ml-auto text-[var(--fg-4)]" style="font-family: var(--mono); font-size: 11px;">
								flagged {reltime(it.created_at)}
							</span>
						</div>
						<div class="mb-3 text-[12px] text-[var(--fg-3)]">
							reported by <span class="text-[var(--fg-2)]">{it.reporter.display_name}</span>
						</div>
						<div class="text-[14px] leading-[1.65] text-[var(--fg)]">{it.review_body}</div>
						<div class="mt-4 flex gap-2">
							<button
								type="button"
								class="inline-flex items-center gap-2 rounded-[7px] border px-[14px] py-2 text-[13px] whitespace-nowrap transition-[background-color,border-color,transform] duration-[120ms] border-[rgba(217,138,138,0.2)] bg-[var(--danger-bg)] text-[var(--danger)] hover:bg-[rgba(217,138,138,0.14)] hover:border-[rgba(217,138,138,0.32)] active:scale-[0.98]"
								onclick={() => withconfirm('Delete this review? This action can be undone from the audit log.', () => deleteReview(it.id), 'Delete review', true)}
							>Delete review</button>
							<button
								type="button"
								class="inline-flex items-center gap-2 rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] whitespace-nowrap text-[var(--fg)] transition-[background-color,border-color,transform] duration-[120ms] hover:bg-[var(--bg-3)] hover:border-[var(--border-strong)] active:scale-[0.98]"
								onclick={() => withconfirm('Dismiss this flag?', () => dismiss(it.id), 'Dismiss flag')}
							>Dismiss flag</button>
						</div>
					</div>
				{/each}
			{/if}

		{:else if activetab === 'proposed'}
			{#if proposed.length === 0}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]"
					style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
					Inbox zero. No proposed offerings.
				</div>
			{:else}
				{#each proposed as p (p.id)}
					<div
						class="mb-3 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] p-[22px]"
						style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);"
					>
						<div class="mb-3 flex items-center gap-3 text-[12px] text-[var(--fg-3)]">
							<span
								class="rounded px-2 py-[2px] text-[11px] text-[var(--accent-2)] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)]"
								style="font-family: var(--mono);"
							>PROPOSED</span>
							<span
								class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] tracking-[0.02em] text-[var(--fg-2)]"
								style="font-family: var(--mono);"
							>{p.course_code}</span>
							<span style="font-family: var(--mono); color: var(--fg-2);">{p.season === 'M' ? 'Monsoon' : 'Spring'} 20{p.year}</span>
							{#if p.faculty && p.faculty.length > 0}
								<span class="text-[var(--fg-4)]">·</span>
								<span class="text-[var(--fg-3)]">Taught by {p.faculty.join(', ')}</span>
							{/if}
						</div>
						<div class="mb-3 text-[13px] font-semibold text-[var(--fg)]">{p.course_name}</div>
						{#if p.reviews.length === 0}
							<div class="text-[13px] italic text-[var(--fg-4)] mb-3">Proposed directly (no review content)</div>
						{:else}
							{#each p.reviews as r (r.id)}
								<div class="border-t border-[var(--border)] pt-3 mt-3">
									<div class="flex items-center gap-2 mb-2 text-[11px] text-[var(--fg-3)]">
										<span>Review by {r.author_name ?? 'Anonymous'}</span>
										<span>·</span>
										<span class="text-[11px] text-[var(--fg-4)]">{reltime(r.created_at)}</span>
									</div>
									<div class="grid grid-cols-5 gap-2 text-[11px] text-[var(--fg-2)] mb-2" style="font-family: var(--mono);">
										<div>diff: {r.difficulty}/5</div>
										<div>teach: {r.teaching}/5</div>
										<div>grad: {r.grading}/5</div>
										<div>cont: {r.content}/5</div>
										<div>work: {r.workload}/5</div>
									</div>
									<div class="text-[13px] leading-[1.6] text-[var(--fg-2)]">{r.body}</div>
								</div>
							{/each}
						{/if}
						<div class="mt-4 flex gap-2">
							<button
								type="button"
								class="inline-flex items-center gap-2 rounded-[7px] border px-[14px] py-2 text-[13px] whitespace-nowrap transition-[background-color,border-color,transform] duration-[120ms] border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] text-[var(--accent-2)] hover:bg-[rgba(107,143,111,0.14)] hover:border-[rgba(107,143,111,0.32)] active:scale-[0.98]"
								onclick={() => approveProp(p.id)}
							>Approve</button>
							<button
								type="button"
								class="inline-flex items-center gap-2 rounded-[7px] border border-[var(--border-2)] bg-[var(--bg-2)] px-[14px] py-2 text-[13px] whitespace-nowrap text-[var(--fg)] transition-[background-color,border-color,transform] duration-[120ms] hover:bg-[var(--bg-3)] hover:border-[var(--border-strong)] active:scale-[0.98]"
								onclick={() => withconfirm(`Reject this proposed offering for ${p.course_code}?`, () => rejectProp(p.id), 'Reject', true)}
							>Reject</button>
						</div>
					</div>
				{/each}
			{/if}

		{:else if activetab === 'deleted'}
			{#if totaldeleted === 0}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]"
					style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
					Nothing deleted.
				</div>
			{:else}
				{#if deleted.length > 0}
					<div class="mb-2 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">Courses</div>
					{#each deleted as d (d.code)}
						<div class="mb-3 flex items-center gap-4 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[22px] py-[16px]"
							style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
							<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{d.code}</span>
							<span class="flex-1 text-[14px] text-[var(--fg)]">{d.name}</span>
							<span class="text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">{reltime(d.deleted_at)}</span>
							<button type="button" onclick={() => doRestore(d.code)}
								class="inline-flex items-center rounded-[7px] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] px-[12px] py-[6px] text-[12px] font-medium text-[var(--accent-2)] transition-[background-color,transform] duration-[120ms] hover:bg-[rgba(107,143,111,0.14)] active:scale-[0.98]"
							>Restore</button>
						</div>
					{/each}
				{/if}
				{#if deletedfaculty.length > 0}
					<div class="mb-2 mt-4 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">Faculty</div>
					{#each deletedfaculty as d (d.slug)}
						<div class="mb-3 flex items-center gap-4 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[22px] py-[16px]"
							style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
							<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{d.slug}</span>
							<span class="flex-1 text-[14px] text-[var(--fg)]">{d.name}</span>
							<span class="text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">{reltime(d.deleted_at)}</span>
							<button type="button" onclick={() => doRestoreFaculty(d.slug)}
								class="inline-flex items-center rounded-[7px] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] px-[12px] py-[6px] text-[12px] font-medium text-[var(--accent-2)] transition-[background-color,transform] duration-[120ms] hover:bg-[rgba(107,143,111,0.14)] active:scale-[0.98]"
							>Restore</button>
						</div>
					{/each}
				{/if}
				{#if deletedlabs.length > 0}
					<div class="mb-2 mt-4 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">Labs</div>
					{#each deletedlabs as d (d.shortname)}
						<div class="mb-3 flex items-center gap-4 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[22px] py-[16px]"
							style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
							<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{d.shortname}</span>
							<span class="flex-1 text-[14px] text-[var(--fg)]">{d.name}</span>
							<span class="text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">{reltime(d.deleted_at)}</span>
							<button type="button" onclick={() => doRestoreLab(d.shortname)}
								class="inline-flex items-center rounded-[7px] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] px-[12px] py-[6px] text-[12px] font-medium text-[var(--accent-2)] transition-[background-color,transform] duration-[120ms] hover:bg-[rgba(107,143,111,0.14)] active:scale-[0.98]"
							>Restore</button>
						</div>
					{/each}
				{/if}
				{#if deletedofferings.length > 0}
					<div class="mb-2 mt-4 text-[11px] uppercase tracking-[0.08em] text-[var(--fg-4)]" style="font-family: var(--mono);">Offerings</div>
					{#each deletedofferings as d (d.id)}
						<div class="mb-3 flex items-center gap-4 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[22px] py-[16px]"
							style="background-image: linear-gradient(180deg, rgba(107,143,111,0.035), transparent 42%);">
							<span class="rounded-[5px] border border-[var(--border-strong)] px-2 py-[3px] text-[12px] text-[var(--fg-2)]" style="font-family: var(--mono);">{d.course_code} · {d.season === 'M' ? 'Monsoon' : 'Spring'} 20{d.year}</span>
							<span class="flex-1 text-[14px] text-[var(--fg)]">{d.course_name}</span>
							<span class="text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">{reltime(d.deleted_at)}</span>
							<button type="button" onclick={() => doRestoreOffering(d.id)}
								class="inline-flex items-center rounded-[7px] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] px-[12px] py-[6px] text-[12px] font-medium text-[var(--accent-2)] transition-[background-color,transform] duration-[120ms] hover:bg-[rgba(107,143,111,0.14)] active:scale-[0.98]"
							>Restore</button>
						</div>
					{/each}
				{/if}
			{/if}
		{:else if activetab === 'moderators'}
			<div class="mb-6 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] p-[22px]">
				<div class="mb-4 text-[13px] font-medium text-[var(--fg)]">Add moderator</div>
				<div class="mb-4 flex gap-2">
					<button
						type="button"
						onclick={() => { modtype = 'cas'; }}
						class="rounded-[6px] border px-3 py-[5px] text-[12px] transition-colors {modtype === 'cas' ? 'border-[rgba(107,143,111,0.3)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border)] bg-transparent text-[var(--fg-3)] hover:text-[var(--fg)]'}"
					>CAS account</button>
					<button
						type="button"
						onclick={() => { modtype = 'local'; }}
						class="rounded-[6px] border px-3 py-[5px] text-[12px] transition-colors {modtype === 'local' ? 'border-[rgba(107,143,111,0.3)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border)] bg-transparent text-[var(--fg-3)] hover:text-[var(--fg)]'}"
					>Local account</button>
				</div>
				{#if modtype === 'cas'}
					<input
						type="email"
						placeholder="CAS email"
						bind:value={modinput.casid}
						class="mb-3 w-full rounded-[7px] border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-[13px] text-[var(--fg)] placeholder-[var(--fg-4)] outline-none focus:border-[var(--border-strong)]"
					/>
				{:else}
					<input
						type="text"
						placeholder="Username"
						bind:value={modinput.username}
						class="mb-3 w-full rounded-[7px] border border-[var(--border)] bg-[var(--bg)] px-3 py-2 text-[13px] text-[var(--fg)] placeholder-[var(--fg-4)] outline-none focus:border-[var(--border-strong)]"
					/>
				{/if}
				<button
					type="button"
					onclick={doAddMod}
					disabled={modsubmitting}
					class="inline-flex items-center rounded-[7px] border border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] px-[14px] py-2 text-[13px] font-medium text-[var(--accent-2)] transition-colors hover:bg-[rgba(107,143,111,0.14)] disabled:opacity-50"
				>{modsubmitting ? 'Adding…' : 'Add moderator'}</button>
			</div>

			{#if moderators.length === 0}
				<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[60px] text-center text-[13px] text-[var(--fg-3)]">
					No moderators yet.
				</div>
			{:else}
				{#each moderators as mod (mod.id)}
					<div class="mb-2 flex items-center gap-4 rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-[18px] py-[14px]">
						<div class="flex-1 min-w-0">
							<div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
								<span class="text-[14px] font-medium text-[var(--fg)]">{mod.display_name}</span>
								<span class="rounded-[4px] border px-[6px] py-[1px] text-[11px] {mod.cas_id ? 'border-[rgba(107,143,111,0.2)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border-strong)] text-[var(--fg-3)]'}">{mod.cas_id ? 'CAS' : 'Local'}</span>
								<span class="text-[12px] text-[var(--fg-3)]" style="font-family: var(--mono);">{mod.cas_id ?? mod.username ?? ''}</span>
							</div>
						</div>
						<span class="shrink-0 text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">{reltime(mod.created_at)}</span>
						{#if mod.id !== $currentUser?.id}
							<button
								type="button"
								onclick={() => withconfirm(`Demote ${mod.display_name} from moderator?`, () => doDemote(mod.id), 'Demote', true)}
								aria-label="Demote {mod.display_name}"
								class="shrink-0 inline-flex items-center rounded-[6px] border border-[rgba(217,138,138,0.2)] bg-[var(--danger-bg)] px-[10px] py-[4px] text-[11px] font-medium text-[var(--danger)] transition-colors hover:bg-[rgba(217,138,138,0.14)]"
							>Demote</button>
						{/if}
					</div>
				{/each}
			{/if}

		{/if}
			</div><!-- flex-1 content -->
		</div><!-- sidebar+content flex -->
	{/if}
</div>

{#if dialog}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={() => dialog = null}
	>
		<div
			class="mx-4 w-full max-w-[360px] rounded-[12px] border border-[var(--border)] bg-[var(--bg-2)] p-6 shadow-2xl"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
		>
			<p class="mb-5 text-[14px] leading-[1.6] text-[var(--fg)]">{dialog.msg}</p>
			<div class="flex justify-end gap-2">
				<button
					type="button"
					onclick={() => dialog = null}
					class="rounded-[7px] border border-[var(--border)] bg-transparent px-4 py-[7px] text-[13px] text-[var(--fg-2)] transition-colors hover:bg-[var(--bg-3)]"
				>Cancel</button>
				<button
					type="button"
					onclick={() => { dialog!.fn(); dialog = null; }}
					class="rounded-[7px] border px-4 py-[7px] text-[13px] font-medium transition-colors {dialog.danger ? 'border-[rgba(217,138,138,0.3)] bg-[var(--danger-bg)] text-[var(--danger)] hover:bg-[rgba(217,138,138,0.14)]' : 'border-[var(--border-strong)] bg-[var(--bg-3)] text-[var(--fg)] hover:bg-[var(--bg-4,var(--bg-3))]'}"
				>{dialog.label}</button>
			</div>
		</div>
	</div>
{/if}
