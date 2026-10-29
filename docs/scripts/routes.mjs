// SPDX-License-Identifier: GPL-3.0-or-later
const prefixes = Object.freeze({ en: '', fr: '/fr' });
export const defaultSiteOrigin = 'https://slint-kit.dev';

export function localePrefix(locale) {
  if (!Object.hasOwn(prefixes, locale)) throw new Error(`Unsupported documentation language: ${locale}`);
  return prefixes[locale];
}

export function localeUrl(route = '', locale = 'en') {
  return `${localePrefix(locale)}/${route.replace(/^\/+/, '')}`;
}

export function siteOrigin() {
  return (process.env.SITE_URL || defaultSiteOrigin).replace(/\/+$/, '');
}
