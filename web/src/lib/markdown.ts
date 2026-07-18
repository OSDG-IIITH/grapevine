import snarkdown from 'snarkdown';
import DOMPurify from 'dompurify';

export function rendermarkdown(text: string): string {
	return DOMPurify.sanitize(snarkdown(text), {
		ALLOWED_TAGS: ['p', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'strong', 'em', 'a', 'code', 'pre', 'ul', 'ol', 'li', 'br', 'blockquote'],
		ALLOWED_ATTR: ['href'],
	});
}
