// SPDX-License-Identifier: GPL-3.0-or-later
const aliases = new Set(['slint-kit.damiengoehrig.ca', 'www.slint-kit.dev']);

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (aliases.has(url.hostname)) {
      url.protocol = 'https:';
      url.host = 'slint-kit.dev';
      // Resolve retired English prefixes and guide URLs in the same hop.
      if (url.pathname === '/en') url.pathname = '/';
      else if (url.pathname.startsWith('/en/')) url.pathname = url.pathname.slice(3);
      if (url.pathname === '/guides/source-updates/') url.pathname = '/guides/getting-started/';
      else if (url.pathname === '/fr/guides/source-updates/') url.pathname = '/fr/guides/getting-started/';
      return Response.redirect(url.href, 301);
    }
    return env.ASSETS.fetch(request);
  },
};
