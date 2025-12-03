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
  // If Rust returned an 'id_token' or 'user' field, persist a currentUser entry
  if (token && token.id_token) {
    // try to decode preferred_username from id_token payload in JS as a fallback
    try {
      const parts = token.id_token.split('.');
      if (parts.length >= 2) {
        const payload = JSON.parse(atob(parts[1].replace(/-/g, '+').replace(/_/g, '/')));
        const u = payload.preferred_username || payload.upn || payload.email;
        if (u) {
          setCurrentUser(u);
        }
      }
    } catch (e) {
      // ignore
    }
  }

  if (token && token.user) {
    setCurrentUser(token.user);
  }
  return token;
}

export async function probeAccessToken(user: string | null, clientId: string, tenantId: string): Promise<boolean> {
  if (!user) return false;
  try {
    // Try to get an access token; if succeed token is valid or refreshed
    const access = await invoke('get_access_token', { user, clientId, tenantId });
    return !!access;
  } catch (e) {
    return false;
  }
}

export async function getAccessToken(user: string, clientId: string, tenantId: string): Promise<string> {
  return await invoke('get_access_token', { user, clientId, tenantId });
}

export async function whoami(clientId: string, tenantId: string): Promise<{ user: string; authenticated: boolean }> {
  console.log('helpers.whoami: invoking whoami', { clientId, tenantId });
  const res: any = await invoke('whoami', { clientId, tenantId });
  console.log('helpers.whoami: got response', res);
  return { user: res.user || '', authenticated: !!res.authenticated };
}

export async function fetchProtected(apiUrl: string, user: string, clientId: string, tenantId: string): Promise<string> {
  return await invoke('fetch_protected', { apiUrl, user, clientId, tenantId });
}

export async function logout(user: string): Promise<void> {
  await invoke('logout', { user });
  try {
    await invoke('clear_last_user', {});
  } catch (e) {
    // ignore if not available
  }
  localStorage.removeItem('currentUser');
}
