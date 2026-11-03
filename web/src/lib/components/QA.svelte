<script lang="ts">
	import type { Question, Answer, QaTarget, QaSort } from '$lib/types';
	import {
		getQuestions, createQuestion, editQuestion, deleteQuestion,
		voteQuestion, unvoteQuestion, flagQuestion,
		createAnswer, editAnswer, deleteAnswer,
		voteAnswer, unvoteAnswer, flagAnswer
	} from '$lib/api';
	import { currentUser } from '$lib/stores';
	import { rendermarkdown } from '$lib/markdown';
	import { toast } from 'svelte-sonner';
	import { tick } from 'svelte';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import FlagModal from './FlagModal.svelte';
	import IconFlag from '@tabler/icons-svelte/icons/flag';
	import IconTrash from '@tabler/icons-svelte/icons/trash';
	import IconPencil from '@tabler/icons-svelte/icons/pencil';
	import IconSearch from '@tabler/icons-svelte/icons/search';
	import IconPlus from '@tabler/icons-svelte/icons/plus';
	import IconChevronUp from '@tabler/icons-svelte/icons/chevron-up';

	interface Props {
		target: QaTarget;
		offerings?: { id: string; code: string }[];
		oncount?: (n: number) => void;
	}

	let { target, offerings = [], oncount }: Props = $props();

	const iscourse = $derived('course' in target);
	const ismod = $derived(!!$currentUser?.is_admin);
	const offeringmap = $derived(Object.fromEntries(offerings.map((o) => [o.id, o.code])));

	let questions = $state<Question[]>([]);
	let loading = $state(true);
	let query = $state('');
	let sort = $state<QaSort>('top');
	let sortopen = $state(false);
	let composing = $state(false);
	let newq = $state('');
	let anon = $state(true);
	let asking = $state(false);
	let similar = $state<Question[]>([]);
	let seq = 0;

	let open = $state<Record<string, boolean>>({});
	let drafts = $state<Record<string, { body: string; anonymous: boolean; offering: string; active: boolean }>>({});
	let editq = $state<{ id: string; body: string } | null>(null);
	let edita = $state<{ id: string; body: string; offering: string } | null>(null);
	let flagging = $state<{ kind: 'question' | 'answer'; id: string } | null>(null);
	let flagsending = $state(false);
	let flagged = $state<Record<string, boolean>>({});
	let confirm = $state<{ kind: 'question' | 'answer'; id: string } | null>(null);

	const trimmed = $derived(query.trim());
	const newtrim = $derived(newq.trim());
	const sorts: { id: QaSort; label: string }[] = [
		{ id: 'top', label: 'Top' },
		{ id: 'new', label: 'New' },
		{ id: 'unanswered', label: 'Unanswered' }
	];

	/** fetch questions for the current search and sort */
	async function load() {
		const n = ++seq;
		const res = await getQuestions(target, trimmed, sort);
		if (n !== seq) return;
		loading = false;
		if (!res) return;
		questions = res;
		if (!trimmed && sort !== 'unanswered') oncount?.(res.length);
	}

	$effect(() => {
		void trimmed; void sort; void target;
		const t = setTimeout(load, trimmed ? 250 : 0);
		return () => clearTimeout(t);
	});

	$effect(() => {
		const text = newtrim;
		if (!composing || text.length < 10) {
			similar = [];
			return;
		}
		const t = setTimeout(async () => {
			const res = await getQuestions(target, text, 'top');
			if (res && newq.trim() === text) similar = res.slice(0, 3);
		}, 300);
		return () => clearTimeout(t);
	});

	function focus(node: HTMLElement) {
		node.focus();
	}

	function fmtdate(s: string) {
		return new Date(s).toLocaleDateString('en-US', { month: 'short', year: 'numeric' });
	}

	function startask(text = '') {
		newq = text;
		anon = true;
		composing = true;
	}

	function cancelask() {
		composing = false;
		newq = '';
	}

	async function ask() {
		if (asking || newtrim.length < 10) return;
		asking = true;
		const res = await createQuestion(target, newtrim, anon);
		asking = false;
		if (!res) return;
		cancelask();
		query = '';
		sort = 'new';
		toggle(res.id);
	}

	/** close the composer and open an existing question instead */
	async function jumpto(id: string) {
		cancelask();
		query = '';
		open[id] = true;
		drafts[id] ??= blank();
		await load();
		await tick();
		document.getElementById(`q-${id}`)?.scrollIntoView({ block: 'center', behavior: 'smooth' });
	}

	async function togglevote(q: Question) {
		const was = q.voted;
		q.voted = !was;
		q.votes += was ? -1 : 1;
		if (!(await (was ? unvoteQuestion(q.id) : voteQuestion(q.id)))) load();
	}

	async function voteanswer(a: Answer, value: 1 | -1) {
		const prev = a.user_vote;
		const next = prev === value ? null : value;
		if (prev === 1) a.upvotes--;
		if (prev === -1) a.downvotes--;
		if (next === 1) a.upvotes++;
		if (next === -1) a.downvotes++;
		a.user_vote = next;
		if (!(await (next ? voteAnswer(a.id, next) : unvoteAnswer(a.id)))) load();
	}

	function blank() {
		return { body: '', anonymous: true, offering: '', active: false };
	}

	function toggle(id: string) {
		open[id] = !open[id];
		drafts[id] ??= blank();
	}

	async function postanswer(q: Question) {
		const d = drafts[q.id];
		if (!d || d.body.trim().length < 2) return;
		const res = await createAnswer(q.id, d.body.trim(), d.anonymous, d.offering || null);
		if (!res) return;
		drafts[q.id] = blank();
		await load();
	}

	async function saveq() {
		if (!editq) return;
		if (await editQuestion(editq.id, editq.body.trim())) {
			editq = null;
			await load();
		}
	}

	async function savea() {
		if (!edita) return;
		if (await editAnswer(edita.id, edita.body.trim(), edita.offering || null)) {
			edita = null;
			await load();
		}
	}

	async function dodelete() {
		if (!confirm) return;
		const { kind, id } = confirm;
		confirm = null;
		if (await (kind === 'question' ? deleteQuestion(id) : deleteAnswer(id))) await load();
	}

	async function sendflag(reason: string) {
		if (!flagging) return;
		flagsending = true;
		const { kind, id } = flagging;
		const ok = await (kind === 'question' ? flagQuestion(id, reason) : flagAnswer(id, reason));
		flagsending = false;
		if (!ok) return;
		flagged[id] = true;
		flagging = null;
		toast.success('Flag sent to the moderators.');
	}

	const primary = 'background: linear-gradient(180deg,#7ea583 0%,#6b8f6f 100%); border: 1px solid #4d6e51; color: #0f1612; box-shadow: inset 0 1px 0 rgba(255,255,255,0.15);';
	const field = 'w-full resize-none rounded-[7px] border border-[var(--border-2)] bg-transparent px-3 py-2 text-[13px] leading-[1.6] text-[var(--fg)] outline-none transition-[border-color] duration-[120ms] placeholder:text-[var(--fg-4)] hover:border-[var(--border-strong)] focus:border-[var(--accent)]';
	const iconbtn = 'inline-flex h-7 w-7 items-center justify-center rounded-[5px] text-[var(--fg-4)] transition-colors duration-[120ms]';
	const hoveronly = 'opacity-0 transition-opacity duration-[120ms] group-hover:opacity-100 focus-within:opacity-100 max-md:opacity-100';
	const ghost = 'px-2 py-1 text-[12px] text-[var(--fg-3)] transition-colors hover:text-[var(--fg)]';
	const save = 'inline-flex items-center rounded-[6px] px-[12px] py-[5px] text-[12px] font-medium disabled:cursor-not-allowed disabled:opacity-50';
</script>

{#snippet anontoggle(checked: boolean, onchange: (v: boolean) => void)}
	<label class="inline-flex cursor-pointer select-none items-center gap-[10px] text-[13px] text-[var(--fg-2)]">
		<input type="checkbox" class="peer sr-only" {checked} onchange={(e) => onchange(e.currentTarget.checked)} />
		<span
			class="relative flex h-[18px] w-[18px] items-center justify-center rounded-[5px] border transition-[background,border-color,box-shadow] duration-[120ms] {checked ? 'border-[var(--accent)] bg-[var(--accent-bg)]' : 'border-[var(--border)] bg-[var(--bg-inset)]'}"
			style={checked ? 'box-shadow: inset 0 0 0 1px rgba(0,0,0,0.08)' : ''}
		>
			<svg viewBox="0 0 16 16" class="h-[11px] w-[11px] transition-[opacity,transform] duration-[120ms] {checked ? 'opacity-100 scale-100' : 'opacity-0 scale-90'}" fill="none" aria-hidden="true">
				<path d="M3.5 8.5l2.6 2.6L12.5 4.9" stroke="var(--accent)" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
			</svg>
		</span>
		<span>Post anonymously</span>
	</label>
{/snippet}

{#snippet offeringselect(value: string, onchange: (v: string) => void)}
	{#if iscourse && offerings.length > 0}
		<select
			{value}
			onchange={(e) => onchange(e.currentTarget.value)}
			aria-label="Semester"
			class="rounded-[5px] border border-[var(--border)] bg-transparent px-[8px] py-[4px] text-[11px] text-[var(--fg-3)] outline-none transition-colors hover:text-[var(--fg)] focus:border-[var(--accent-2)]"
			style="font-family: var(--mono);"
		>
			<option value="">Any semester</option>
			{#each offerings as o (o.id)}
				<option value={o.id}>{o.code}</option>
			{/each}
		</select>
	{/if}
{/snippet}

{#snippet author(a: { display_name: string } | null)}
	{#if a}
		<span class="text-[var(--fg-2)]">{a.display_name}</span>
	{:else}
		<span>anonymous</span>
	{/if}
{/snippet}

{#snippet answerrow(a: Answer)}
	<div class="group py-[10px]">
		<div class="flex flex-wrap items-center gap-[5px] text-[12px] text-[var(--fg-3)]">
			{@render author(a.author)}
			{#if a.reviewer}
				<span class="text-[var(--fg-4)]">·</span>
				<span class="text-[var(--accent-2)]">took this {iscourse ? 'course' : 'advisor'}</span>
			{/if}
			{#if a.offering_id && offeringmap[a.offering_id]}
				<span class="text-[var(--fg-4)]">·</span>
				<span class="text-[11px]" style="font-family: var(--mono);">{offeringmap[a.offering_id]}</span>
			{/if}
			<span class="text-[var(--fg-4)]">·</span>
			<span class="text-[11px]" style="font-family: var(--mono);">{fmtdate(a.created_at)}</span>
			{#if a.edited_at}<span class="text-[var(--fg-4)]">· edited</span>{/if}
		</div>

		{#if edita?.id === a.id}
			<textarea bind:value={edita.body} rows={4} class="mt-2 {field}"></textarea>
			<div class="mt-2 flex flex-wrap items-center gap-2">
				{@render offeringselect(edita.offering, (v) => { if (edita) edita.offering = v; })}
				<button type="button" onclick={() => (edita = null)} class="ml-auto {ghost}">Cancel</button>
				<button type="button" onclick={savea} class={save} style={primary}>Save</button>
			</div>
		{:else}
			<div class="mt-[4px] text-[13px] leading-[1.6] text-[var(--fg-2)]" style="text-wrap: pretty;"><div class="review-prose">{@html rendermarkdown(a.body)}</div></div>
			<div class="mt-[4px] flex items-center gap-[2px]">
				<button type="button" aria-label="Upvote answer" onclick={() => voteanswer(a, 1)}
					class="-ml-2 inline-flex items-center gap-[5px] rounded-[5px] px-2 py-1 text-[12px] font-semibold transition-colors duration-[120ms] {a.user_vote === 1 ? 'text-[var(--accent-2)]' : 'text-[var(--fg-3)] hover:text-[var(--fg)]'}"
					style="font-family: var(--mono);">
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 19V5M5 12l7-7 7 7"/></svg>
					<span class="min-w-[14px] text-left">{a.upvotes}</span>
				</button>
				<button type="button" aria-label="Downvote answer" onclick={() => voteanswer(a, -1)}
					class="inline-flex items-center gap-[5px] rounded-[5px] px-2 py-1 text-[12px] font-semibold transition-colors duration-[120ms] {a.user_vote === -1 ? 'text-[var(--danger)]' : 'text-[var(--fg-3)] hover:text-[var(--fg)]'}"
					style="font-family: var(--mono);">
					<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 5v14M19 12l-7 7-7-7"/></svg>
					<span class="min-w-[14px] text-left">{a.downvotes}</span>
				</button>
				<div class="ml-auto flex items-center {hoveronly}">
					{#if !a.mine}
						<button type="button" aria-label={flagged[a.id] ? 'Flagged' : 'Flag answer'} onclick={() => { if (!flagged[a.id]) flagging = { kind: 'answer', id: a.id }; }}
							class="{iconbtn} {flagged[a.id] ? 'text-[var(--danger)]' : 'hover:text-[var(--danger)]'}">
							<IconFlag size={13} stroke={1.7} fill={flagged[a.id] ? 'currentColor' : 'none'} />
						</button>
					{:else}
						<button type="button" aria-label="Edit answer" onclick={() => (edita = { id: a.id, body: a.body, offering: a.offering_id ?? '' })} class="{iconbtn} hover:text-[var(--fg)]">
							<IconPencil size={13} stroke={1.7} />
						</button>
					{/if}
					{#if a.mine || ismod}
						<button type="button" aria-label="Delete answer" onclick={() => (confirm = { kind: 'answer', id: a.id })} class="{iconbtn} hover:text-[var(--danger)]">
							<IconTrash size={13} stroke={1.7} />
						</button>
					{/if}
				</div>
			</div>
		{/if}
	</div>
{/snippet}

{#snippet composer(q: Question)}
	{@const d = drafts[q.id]}
	<div class="pt-[6px] pb-[4px]">
		<textarea
			bind:value={d.body}
			onfocus={() => (d.active = true)}
			rows={d.active || d.body ? 4 : 1}
			maxlength={5000}
			placeholder="Write an answer…"
			aria-label="Write an answer"
			class={field}
		></textarea>
		{#if d.active || d.body}
			<div class="mt-2 flex flex-wrap items-center gap-3">
				{@render anontoggle(d.anonymous, (v) => (d.anonymous = v))}
				{@render offeringselect(d.offering, (v) => (d.offering = v))}
				<button type="button" onclick={() => (drafts[q.id] = blank())} class="ml-auto {ghost}">Cancel</button>
				<button type="button" onclick={() => postanswer(q)} disabled={d.body.trim().length < 2} class={save} style={primary}>Post answer</button>
			</div>
		{/if}
	</div>
{/snippet}

<div class="mb-[6px] flex items-center gap-2">
	<div class="flex min-w-0 flex-1 items-center gap-[10px] rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-2)] px-3 py-[9px] transition-[border-color] duration-[120ms] focus-within:border-[var(--accent)] hover:border-[var(--border-strong)]">
		<IconSearch size={15} stroke={2} class="shrink-0 text-[var(--fg-4)]" />
		<input
			bind:value={query}
			maxlength={500}
			placeholder="Search questions"
			aria-label="Search questions"
			autocomplete="off"
			class="min-w-0 flex-1 border-0 bg-transparent text-[13px] text-[var(--fg)] outline-none placeholder:text-[var(--fg-4)]"
			onkeydown={(e) => { if (e.key === 'Escape') query = ''; }}
		/>
		{#if trimmed}
			<span class="shrink-0 text-[11px] text-[var(--fg-4)]" style="font-family: var(--mono);">
				{questions.length} {questions.length === 1 ? 'match' : 'matches'}
			</span>
		{/if}
	</div>
	<Popover.Root bind:open={sortopen}>
		<Popover.Trigger
			class="inline-flex shrink-0 items-center gap-[6px] rounded-[5px] border px-[10px] py-[9px] text-[11px] tracking-[0.04em] transition-[color,background,border-color] duration-[120ms] {sort !== 'top' ? 'border-[var(--accent-dim)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border)] text-[var(--fg-3)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]'}"
			style="font-family: var(--mono);"
			aria-label="Sort questions"
		>
			{sorts.find((s) => s.id === sort)?.label}
			<svg class="opacity-50" width="9" height="6" viewBox="0 0 9 6" fill="none" aria-hidden="true"><path d="M1 1l3.5 3.5L8 1" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" /></svg>
		</Popover.Trigger>
		<Popover.Content class="w-[160px] rounded-[8px] border border-[var(--border-2)] bg-[var(--bg-inset)] p-1 shadow-lg" sideOffset={5} align="end">
			{#each sorts as s (s.id)}
				<button
					type="button"
					onclick={() => { sort = s.id; sortopen = false; }}
					class="flex w-full items-center justify-between rounded-[5px] px-3 py-[7px] text-left text-[12px] transition-colors duration-[80ms] {s.id === sort ? 'text-[var(--accent-2)]' : 'text-[var(--fg-2)] hover:bg-[var(--bg-3)] hover:text-[var(--fg)]'}"
					style="font-family: var(--mono);"
				>
					{s.label}
					{#if s.id === sort}
						<svg width="11" height="9" viewBox="0 0 11 9" fill="none" aria-hidden="true"><path d="M1 4l3.5 3.5L10 1" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" /></svg>
					{/if}
				</button>
			{/each}
		</Popover.Content>
	</Popover.Root>
	{#if !composing}
		<button type="button" onclick={() => startask()} aria-label="Ask a question"
			class="inline-flex shrink-0 items-center gap-[6px] rounded-[7px] px-[12px] py-[8px] text-[12px] font-medium" style={primary}>
			<IconPlus size={13} stroke={2.2} />
			<span class="max-sm:hidden">Ask a question</span>
			<span class="sm:hidden">Ask</span>
		</button>
	{/if}
</div>

{#if composing}
	<div class="mt-3 rounded-[10px] border border-[var(--border-2)] bg-[var(--bg-2)] p-[14px]" style="animation: fadeUp 160ms cubic-bezier(.2,.6,.2,1) both;">
		<textarea
			use:focus
			bind:value={newq}
			rows={3}
			maxlength={500}
			placeholder={iscourse ? 'What do you want to know about this course?' : 'What do you want to know about this professor?'}
			aria-label="Your question"
			class="w-full resize-none border-0 bg-transparent text-[14px] leading-[1.6] text-[var(--fg)] outline-none placeholder:text-[var(--fg-4)]"
			onkeydown={(e) => { if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) ask(); if (e.key === 'Escape') cancelask(); }}
		></textarea>

		{#if similar.length > 0}
			<div class="mt-2 border-t border-[var(--border)] pt-[10px]">
				<div class="mb-[6px] text-[10px] font-medium uppercase tracking-[0.07em] text-[var(--fg-4)]" style="font-family: var(--mono);">Already asked</div>
				{#each similar as q (q.id)}
					<button type="button" onclick={() => jumpto(q.id)}
						class="-mx-2 flex w-[calc(100%+16px)] items-baseline gap-3 rounded-[5px] px-2 py-[6px] text-left transition-colors hover:bg-[var(--bg-3)]">
						<span class="min-w-0 flex-1 truncate text-[13px] text-[var(--fg-2)]">{q.body}</span>
						<span class="shrink-0 text-[11px] {q.answers.length ? 'text-[var(--accent-2)]' : 'text-[var(--fg-4)]'}">
							{q.answers.length} {q.answers.length === 1 ? 'answer' : 'answers'}
						</span>
					</button>
				{/each}
			</div>
		{/if}

		<div class="mt-[10px] flex flex-wrap items-center gap-3 border-t border-[var(--border)] pt-[10px]">
			{@render anontoggle(anon, (v) => (anon = v))}
			{#if newtrim.length > 0 && newtrim.length < 10}
				<span class="text-[12px] text-[var(--fg-4)]">a little more detail</span>
			{/if}
			<button type="button" onclick={cancelask} class="ml-auto {ghost}">Cancel</button>
			<button type="button" onclick={ask} disabled={asking || newtrim.length < 10} class={save} style={primary}>
				{asking ? 'Posting…' : 'Post question'}
			</button>
		</div>
	</div>
{/if}

<div class="mt-4">
	{#if loading}
		<div class="text-[13px] text-[var(--fg-3)]">Loading…</div>
	{:else if questions.length === 0}
		<div class="rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)] px-5 py-[52px] text-center text-[13px] text-[var(--fg-3)]">
			{#if trimmed}
				No questions match “{trimmed}”.
				{#if !composing}
					<button type="button" onclick={() => startask(trimmed)} class="ml-1 text-[var(--accent-2)] transition-colors hover:text-[var(--fg)]">Ask it instead</button>
				{/if}
			{:else if sort === 'unanswered'}
				Every question has an answer.
			{:else}
				No questions yet.
				{#if !composing}
					<button type="button" onclick={() => startask()} class="ml-1 text-[var(--accent-2)] transition-colors hover:text-[var(--fg)]">Ask the first one</button>
				{/if}
			{/if}
		</div>
	{:else}
		<div class="overflow-hidden rounded-[10px] border border-[var(--border)] bg-[var(--bg-2)]">
			{#each questions as q, i (q.id)}
				{@const answered = q.answers.some((a) => a.mine)}
				<div id="q-{q.id}" class="group flex gap-[14px] px-4 py-[14px] {i > 0 ? 'border-t border-[var(--border)]' : ''}">
					<button
						type="button"
						onclick={() => togglevote(q)}
						aria-label={q.voted ? 'Remove me too' : 'Me too'}
						title="I want to know this too"
						class="flex h-fit w-[36px] shrink-0 flex-col items-center rounded-[7px] border py-[4px] text-[12px] transition-colors {q.voted ? 'border-[var(--accent-line)] bg-[var(--accent-bg)] text-[var(--accent-2)]' : 'border-[var(--border-2)] text-[var(--fg-3)] hover:border-[var(--border-strong)] hover:text-[var(--fg)]'}"
						style="font-family: var(--mono);"
					>
						<IconChevronUp size={14} stroke={2} />
						{q.votes}
					</button>

					<div class="min-w-0 flex-1">
						{#if editq?.id === q.id}
							<input bind:value={editq.body} maxlength={500} aria-label="Edit question" class={field} onkeydown={(e) => { if (e.key === 'Enter') saveq(); if (e.key === 'Escape') editq = null; }} />
							<div class="mt-2 flex justify-end gap-2">
								<button type="button" onclick={() => (editq = null)} class={ghost}>Cancel</button>
								<button type="button" onclick={saveq} class={save} style={primary}>Save</button>
							</div>
						{:else}
							<div class="flex items-start gap-2">
								<button type="button" onclick={() => toggle(q.id)} aria-expanded={!!open[q.id]}
									class="min-w-0 flex-1 text-left text-[14px] leading-[1.5] text-[var(--fg)] transition-colors hover:text-[var(--accent-2)]" style="text-wrap: pretty;">
									{q.body}
								</button>
								<div class="flex shrink-0 items-center {hoveronly}">
									{#if !q.mine}
										<button type="button" aria-label={flagged[q.id] ? 'Flagged' : 'Flag question'} onclick={() => { if (!flagged[q.id]) flagging = { kind: 'question', id: q.id }; }}
											class="{iconbtn} {flagged[q.id] ? 'text-[var(--danger)]' : 'hover:text-[var(--danger)]'}">
											<IconFlag size={13} stroke={1.7} fill={flagged[q.id] ? 'currentColor' : 'none'} />
										</button>
									{:else}
										<button type="button" aria-label="Edit question" onclick={() => (editq = { id: q.id, body: q.body })} class="{iconbtn} hover:text-[var(--fg)]">
											<IconPencil size={13} stroke={1.7} />
										</button>
									{/if}
									{#if ismod || (q.mine && q.answers.length === 0)}
										<button type="button" aria-label="Delete question" onclick={() => (confirm = { kind: 'question', id: q.id })} class="{iconbtn} hover:text-[var(--danger)]">
											<IconTrash size={13} stroke={1.7} />
										</button>
									{/if}
								</div>
							</div>
						{/if}

						<div class="mt-[3px] flex flex-wrap items-center gap-[5px] text-[12px] text-[var(--fg-3)]">
							<button type="button" onclick={() => toggle(q.id)} class="transition-colors hover:text-[var(--fg)] {q.answers.length ? 'text-[var(--accent-2)]' : ''}">
								{q.answers.length === 0 ? 'No answers yet' : `${q.answers.length} ${q.answers.length === 1 ? 'answer' : 'answers'}`}
							</button>
							<span class="text-[var(--fg-4)]">·</span>
							{@render author(q.author)}
							<span class="text-[var(--fg-4)]">·</span>
							<span class="text-[11px]" style="font-family: var(--mono);">{fmtdate(q.created_at)}</span>
							{#if q.edited_at}<span class="text-[var(--fg-4)]">· edited</span>{/if}
						</div>

						{#if open[q.id]}
							<div class="mt-[10px] border-l border-[var(--border-2)] pl-[14px]" style="animation: fadeUp 160ms cubic-bezier(.2,.6,.2,1) both;">
								{#each q.answers as a (a.id)}
									{@render answerrow(a)}
								{/each}
								{#if !answered && drafts[q.id]}
									{@render composer(q)}
								{/if}
							</div>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

{#if flagging}
	<FlagModal what={flagging.kind} submitting={flagsending} onsubmit={sendflag} onclose={() => (flagging = null)} />
{/if}

{#if confirm}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="dialog" aria-modal="true">
		<div class="w-full max-w-[340px] rounded-[12px] border border-[var(--border)] bg-[var(--bg-2)] p-6 shadow-xl">
			<p class="mb-1 text-[15px] font-medium text-[var(--fg)]">Delete {confirm.kind}?</p>
			<p class="mb-5 text-[13px] text-[var(--fg-3)]">
				{ismod ? 'It can be restored from the audit log.' : 'This action cannot be undone.'}
			</p>
			<div class="flex justify-end gap-2">
				<button type="button" onclick={() => (confirm = null)} class="rounded-[7px] border border-[var(--border-strong)] bg-[var(--bg-3)] px-[14px] py-2 text-[13px] font-medium text-[var(--fg-2)] transition-colors hover:bg-[var(--bg-4)]">Cancel</button>
				<button type="button" onclick={dodelete} class="rounded-[7px] px-[14px] py-2 text-[13px] font-medium text-white" style="background: var(--danger);">Delete</button>
			</div>
		</div>
	</div>
{/if}
