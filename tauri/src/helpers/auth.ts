import { invoke } from '@tauri-apps/api/core';

export function currentUser(): string | null {
  return localStorage.getItem('currentUser');
}

export function setCurrentUser(user: string) {
  localStorage.setItem('currentUser', user);
}

export async function login(clientId: string, tenantId: string): Promise<any> {
  const token: any = await invoke('login', { clientId, tenantId });
  // token includes id_token etc. Extract user from id_token on Rust side or store returned user.
  // If your Rust `login` returns TokenResponse, adapt accordingly.
  if (token && token.id_token) {
    // optionally extract preferred_username in JS or rely on Rust to return user
    // For simplicity assume Rust saved the token and we can derive user from token in Rust
  }
  return token;
}

export async function getAccessToken(user: string, clientId: string, tenantId: string): Promise<string> {
  return await invoke('get_access_token', { user, clientId, tenantId });
}

export async function fetchProtected(apiUrl: string, user: string, clientId: string, tenantId: string): Promise<string> {
  return await invoke('fetch_protected', { apiUrl, user, clientId, tenantId });
}

export async function logout(user: string): Promise<void> {
  await invoke('logout', { user });
  localStorage.removeItem('currentUser');
}
