import snarkdown from 'snarkdown';
import DOMPurify from 'dompurify';

const BLOCK = /^<(h[1-6]|ul|ol|blockquote|pre|hr)/;

export function rendermarkdown(text: string): string {
	const parts = text.split(/\n{2,}/).map(p => snarkdown(p).replace(/\n/g, '<br>'));
	let html = '';
	for (let i = 0; i < parts.length; i++) {
		if (i > 0 && !BLOCK.test(parts[i - 1]) && !BLOCK.test(parts[i])) {
			html += '<br><br>';
		}
		html += parts[i];
	}
	return DOMPurify.sanitize(html, {
		ALLOWED_TAGS: ['p', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'strong', 'em', 'a', 'code', 'pre', 'ul', 'ol', 'li', 'br', 'blockquote'],
		ALLOWED_ATTR: ['href'],
	});
}
