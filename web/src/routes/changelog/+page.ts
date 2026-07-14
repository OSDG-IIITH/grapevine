import changelogRaw from '../../lib/changelog.md?raw';

interface Feature {
	title: string;
	desc: string;
	items: string[];
}

export interface Entry {
	date: string;
	label: string;
	features: Feature[];
}

function fmtdate(s: string): string {
	const ymd = s.match(/^(\d{4})-(\d{2})-(\d{2})$/);
	if (ymd) {
		const d = new Date(+ymd[1], +ymd[2] - 1, +ymd[3]);
		return d.toLocaleDateString('en-US', { day: 'numeric', month: 'short', year: 'numeric' });
	}
	const ym = s.match(/^(\d{4})-(\d{2})$/);
	if (ym) {
		const d = new Date(+ym[1], +ym[2] - 1, 1);
		return d.toLocaleDateString('en-US', { month: 'long', year: 'numeric' });
	}
	return s;
}

function parse(md: string): Entry[] {
	const entries: Entry[] = [];
	let curEntry: Entry | null = null;
	let curFeature: Feature | null = null;
	for (const raw of md.split('\n')) {
		const line = raw.trim();
		if (!line) continue;
		const h2 = line.match(/^##\s+(.+)$/);
		if (h2) {
			curFeature = null;
			curEntry = { date: h2[1].trim(), label: fmtdate(h2[1].trim()), features: [] };
			entries.push(curEntry);
			continue;
		}
		const h3 = line.match(/^###\s+(.+)$/);
		if (h3 && curEntry) {
			curFeature = { title: h3[1].trim(), desc: '', items: [] };
			curEntry.features.push(curFeature);
			continue;
		}
		const item = line.match(/^[-*]\s+(.+)$/);
		if (item && curFeature) { curFeature.items.push(item[1]); continue; }
		if (curFeature && !line.startsWith('#')) {
			curFeature.desc += (curFeature.desc ? ' ' : '') + line;
		}
	}
	return entries;
}

export function load() {
	return { entries: parse(changelogRaw) };
}
