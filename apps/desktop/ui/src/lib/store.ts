import { writable } from 'svelte/store';
export const events = writable<any[]>([]);
export const selected = writable<string|null>(null);
export const alerts = writable<any[]>([]);
export const viewport = writable<[number, number, number, number] | null>(null);
