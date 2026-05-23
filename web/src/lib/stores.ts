import { writable } from 'svelte/store';
import type { AuthUser } from '$lib/types';

export const currentUser = writable<AuthUser | null>(null);
